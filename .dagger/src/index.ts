import {
  argument,
  Container,
  File,
  dag,
  Directory,
  func,
  object,
  Secret,
} from "@dagger.io/dagger";


@object()
export class MyceliumBuild {
  source: Directory;

  constructor(
    @argument({
      defaultPath: ".",
      ignore: [
        "**/node_modules",
        "**/.git",
        "**/.github",
        "**/.husky",
        "**/.vscode",
        // rust
        "**/target",
        "**/artifact",
        // browser
        "**/.swc",
        "**/.netlify",
        // e2e
        "**/test-results",
        "**/template-tests",
        "**/playwright-report",
        "**/tmp",
        "**/.temp",
        "**/.DS_Store",
        "**/.vscode",
        "**/dist",
        "**/assets_tmp",
        "**/build",
        "**/.env",
        "**/.envrc",
      ],
    }) source: Directory,
  ) {
    this.source = source;
  }

  /**
   * Container for building the Scala backend
   */
  containerBackend(): Container {
    const src = this.source;

    return dag
      .container()
      .from("sbtscala/scala-sbt:eclipse-temurin-alpine-21.0.7_6_1.11.3_2.13.16")
      .withMountedCache("/root/.sbt", dag.cacheVolume("sbt-cache"))
      .withMountedCache("/root/.ivy2", dag.cacheVolume("ivy2-cache"))
      .withMountedCache("/root/.cache/coursier", dag.cacheVolume("scala-coursier-cache"))
      .withDirectory("/workspace", src)
      .withWorkdir("/workspace/backend");
  }

  @func()
  publishBackend(username: string, tag?: string): Promise<string> {
    const builder = this.containerBackend().withExec(["sbt", "assembly"]);

    const image = dag
      .container()
      .from("eclipse-temurin:17.0.7_7-jre")
      .withFile("backend-assembly-1.0.jar", builder.file("target/scala-2.13/backend-assembly-1.0.jar"))
      .withEntrypoint(["java","-cp","backend-assembly-1.0.jar","co.mycelium.Main"]);

    const final_tag = tag ?? "latest";
    const addr = image
      .publish(`${username}/mycelium-backend:${final_tag}`);

    return addr;
  }

  /**
   * Build the Scala backend
   */
  @func()
  async buildBackend(): Promise<string> {
    return this.containerBackend()
      .withExec(["sbt", "compile"])
      .stdout();
  }

  @func()
  createClient(generator: string): Directory {
  
    const openapi = this.containerBackend().withExec(["sbt", 'runMain co.mycelium.OpenApiGenerator']).file("openapi.json");
    const generated = dag.container().from("openapitools/openapi-generator-cli:v7.14.0")
      .withFile("/tmp/openapi.json", openapi)
      .withExec(["mkdir", "-p", "/out"])
      .withExec([
        "/usr/local/bin/docker-entrypoint.sh", "generate",
          "-i", "/tmp/openapi.json",     // input file
          "-g", generator,                
          "-o", "/out"             // output directory inside container
      ]);

    return generated.directory("/out");
  }

  /**
   * Test the Scala backend
   */
  @func()
  async testBackend(): Promise<string> {
    return this.containerBackend()
      .withExec(["sbt", "test"])
      .stdout();
  }

  /**
   * Container for building the Tauri app
   */
  containerApp(): Container {
    const src = this.source;

    let container = dag
      .container()
      .from("rust:1.88-bookworm")
      .withExec(["apt-get", "install", "-y", "curl", "ca-certificates", "gnupg"])
      .withExec(["bash", "-c", "curl -fsSL https://deb.nodesource.com/setup_20.x | bash -"])
      .withExec(["apt-get", "update"])      
      .withExec([
        "apt-get",
        "install",
        "-y",
        "libwebkit2gtk-4.1-dev",
        "build-essential",
        "wget",
        "file",
        "libxdo-dev",
        "libssl-dev",
        "libayatana-appindicator3-dev",
        "librsvg2-dev",
        "xdg-utils",
        "nodejs"
      ])
      .withMountedCache("/root/.cargo", dag.cacheVolume("cargo-tauri"))
      .withMountedCache("/root/.npm", dag.cacheVolume("npm-tauri"))
      .withDirectory("/workspace", src)
      .withWorkdir("/workspace/app");

    return container;
  }

  /**
   * Build the Tauri app for a specific platform
   */
  @func()
  async buildApp(): Promise<string> {
    return this.containerApp()
      .withExec(["bash", "-c", "npm install && npm run tauri build"])
      .stdout();
  }

  /**
   * Container for building the central component with dbus support
   */
  containerCentral(): Container {
    const src = this.source;

    return dag
      .container()
      .from("rust:1.88-bookworm")
      .withExec(["apt-get", "update"])
      .withExec([
        "apt-get",
        "install",
        "-y",
        "libdbus-1-3",
        "libdbus-1-dev",
        "dbus",
        "pkg-config",
      ])
      .withMountedCache("/root/.cargo/registry", dag.cacheVolume("cargo-edge-central-registry"))
      .withMountedCache("/root/.cargo/git", dag.cacheVolume("cargo-edge-central-git"))
      .withDirectory("/workspace", src)
      .withWorkdir("/workspace/edge-central");
  }

  /**
   * Container for building the peripheral component with xtensa toolchain
   */
  containerPeripheral(arch: string): Container {
    const src = this.source;

    return dag
      .container({ platform: arch as any })
      .from("rust:1.88-bookworm")
      .withExec(["apt-get", "update"])
      .withExec([
        "apt-get",
        "install",
        "-y",
        "gcc",
        "build-essential",
        "curl",
        "pkg-config",
      ])
      .withMountedCache("/root", dag.cacheVolume("root-edge-peripheral"))
      .withMountedCache("/usr/local/rustup/toolchains/esp", dag.cacheVolume("firmware-rustup-esp-toolchain"))
      .withExec(["bash", "-c", "cargo install espup --locked"])
      .withExec(["bash", "-c", "espup install"])
      .withDirectory("/workspace", src)
      .withWorkdir("/workspace/edge-peripheral")
  }

  /**
   * Execute a command in the peripheral container with ESP environment
   */
  execPeripheralWithEnv(arch: string, command: string): Container {
    return this.containerPeripheral(arch)
      .withExec(["bash", "-c", `source /root/export-esp.sh && ${command}`]);
  }

  /**
   * Build the central component
   */
  @func()
  buildCentral(): Container {
    return this.containerCentral()
      .withExec(["cargo", "build", "--release"]);
  }

  /**
   * Test the central component
   */
  @func()
  async testCentral(): Promise<string> {
    return this.containerCentral()
      .withExec(["cargo", "test"]).stdout();
  }

  /**
   * Build the peripheral component for ESP32
   */
  @func()
  async buildPeripheral(@argument() arch: string = "linux/arm64"): Promise<string> {
    return this.execPeripheralWithEnv(arch, "cargo build --release").stdout();
  }

  @func()
  async ci(@argument() arch: string = "linux/amd64") {
    await Promise.all([
      this.buildPeripheral(arch),
      this.testCentral(),
      this.buildBackend(),
      this.testBackend(),
      this.buildApp()
    ]);

    return "CI pipeline completed successfully";
  }
}