import {
  argument,
  Container,
  dag,
  Directory,
  func,
  object,
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
      .withMountedCache("/root/.cargo", dag.cacheVolume("cargo-edge-central"))
      .withDirectory("/workspace", src)
      .withWorkdir("/workspace/edge-central");
  }

  /**
   * Container for building the peripheral component with xtensa toolchain
   */
  containerPeripheral(arch: string): Container {
    const src = this.source;

    return dag
      .container({ platform: arch })
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
      this.testCentral()
    ]);

    return "CI pipeline completed successfully";
  }
}
