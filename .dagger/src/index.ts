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
    const cargoCache = dag.cacheVolume("cargo-central");

    return dag
      .container()
      .from("rust:1.88-bookworm")
      .withMountedCache("/usr/local/cargo/registry", cargoCache)
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
      .withDirectory("/workspace", src)
      .withWorkdir("/workspace/edge-central");
  }

  /**
   * Container for building the peripheral component with xtensa toolchain
   */
  containerPeripheral(): Container {
    const src = this.source;
    const cargoCache = dag.cacheVolume("cargo-peripheral");

    return dag
      .container({ platform: "linux/arm64" })
      .from("rust:1.88-bookworm")
      .withMountedCache("/usr/local/cargo/registry", cargoCache)
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
      .withExec(["bash", "-c", "cargo install espup --locked"])
      .withExec(["bash", "-c", "espup install"])
      .withExec(["bash", "-c", "source /root/export-esp.sh"])
      .withDirectory("/workspace", src)
      .withWorkdir("/workspace/edge-peripheral")
    
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
  testCentral(): Container {
    return this.containerCentral()
      .withExec(["cargo", "test"]);
  }

  /**
   * Build the peripheral component for ESP32
   */
  @func()
  buildPeripheral(): Container {
    return this.containerPeripheral()
      .withExec(["bash", "-c", "cargo build --release"]);
  }

  @func()
  async ci() {
    await Promise.all([
      this.buildPeripheral(),
      this.testCentral()
    ]);

    return "CI pipeline completed successfully";
  }
}
