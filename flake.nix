{
  inputs = {
    pkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nmattia/naersk";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, utils, naersk, ... }@inputs:
    utils.lib.eachDefaultSystem (system:
      let
        pname = "2a-emulator";

        overlays = [ inputs.rust-overlay.overlay ];
        pkgs = import inputs.pkgs { inherit system overlays; };

        # Get the latest rust nightly
        rust = pkgs.rust-bin.selectLatestNightlyWith (toolchain:
          toolchain.default.override { extensions = [ "rust-src" ]; });

        # Override the version used in naersk
        naersk-lib = naersk.lib."${system}".override {
          cargo = rust;
          rustc = rust;
        };
      in rec {
        # `nix build`
        packages.${pname} = naersk-lib.buildPackage {
          inherit pname;
          root = ./.;
        };
        defaultPackage = packages.${pname};

        # `nix run`
        apps.${pname} = utils.lib.mkApp { drv = packages.${pname}; };
        defaultApp = apps.${pname};

        # `nix develop`
        devShell = pkgs.mkShell {
          # supply the specific rust version
          nativeBuildInputs = [ rust pkgs.cargo-readme ];
          RUST_SRC_PATH = "${rust}";
        };

        # `nix check`
        checks.${pname} = naersk-lib.buildPackage {
          inherit pname;
          root = ./.;
          doCheck = true;
        };
      });
}
