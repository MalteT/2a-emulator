{
  pkgs,
  perSystem,
  ...
}: let
  manifest = (pkgs.lib.importTOML ../../emulator-2a/Cargo.toml).package;
  # The toolchain file is the single source of truth for the Rust version.
  toolchain = perSystem.fenix.fromToolchainFile {
    file = ../../rust-toolchain.toml;
    sha256 = "sha256-qqF33vNuAdU5vua96VKVIwuc43j4EFeEXbjQ6+l4mO4=";
  };
  rustPlatform = pkgs.makeRustPlatform {
    cargo = toolchain;
    rustc = toolchain;
  };
in
  rustPlatform.buildRustPackage {
    pname = "2a-emulator";
    inherit (manifest) version;

    # Only the Rust sources go into the build, so editing nix files does not rebuild it.
    src = pkgs.lib.fileset.toSource {
      root = ../..;
      fileset = pkgs.lib.fileset.unions [
        ../../Cargo.toml
        ../../Cargo.lock
        ../../emulator-2a
        ../../emulator-2a-lib
        # The tests assemble the example programs.
        ../../testing
      ];
    };
    cargoLock.lockFile = ../../Cargo.lock;

    passthru.toolchain = toolchain;

    meta = {
      description = "Emulator for the Minirechner 2a microcomputer";
      homepage = "https://github.com/MalteT/2a-emulator";
      license = pkgs.lib.licenses.gpl3Only;
      mainProgram = "2a-emulator";
    };
  }
