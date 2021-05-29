{
  description = "Emulator for the Minirechner 2a microcomputer";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crate2nix = {
      url = "github:kolloch/crate2nix";
      flake = false;
    };
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, utils, rust-overlay, crate2nix, ... }:
    let
      name = "emulator-2a";
      binName = "2a-emulator";
      rustChannel = "nightly";
    in utils.lib.eachDefaultSystem (system:
      let
        # Imports
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            rust-overlay.overlay
            (self: super: {
              # Because rust-overlay bundles multiple rust packages into one
              # derivation, specify that mega-bundle here, so that crate2nix
              # will use them automatically.
              rustc = self.rust-bin.${rustChannel}.latest.default;
              cargo = self.rust-bin.${rustChannel}.latest.default;
            })
          ];
        };
        inherit (import "${crate2nix}/tools.nix" { inherit pkgs; })
          generatedCargoNix;

        # Create the cargo2nix project
        project = import (generatedCargoNix {
          inherit name;
          src = ./.;
        }) { inherit pkgs; };

        # Configuration for the non-Rust dependencies
        buildInputs = with pkgs; [ ];
        nativeBuildInputs = with pkgs; [ rustc cargo pkgconfig ];
      in rec {
        packages.${name} = project.workspaceMembers.${name}.build;

        # `nix build`
        defaultPackage = packages.${name};

        # `nix run`
        apps.${name} = utils.lib.mkApp {
          name = binName;
          drv = packages.${name};
        };
        defaultApp = apps.${name};

        # `nix develop`
        devShell = pkgs.mkShell {
          inputsFrom = builtins.attrValues self.packages.${system};
          buildInputs = buildInputs ++ (with pkgs;
          # Tools you need for development go here.
            [
              nixpkgs-fmt
              cargo-watch
              pkgs.rust-bin.${rustChannel}.latest.rust-analysis
              pkgs.rust-bin.${rustChannel}.latest.rls
            ]);
          RUST_SRC_PATH = "${
              pkgs.rust-bin.${rustChannel}.latest.rust-src
            }/lib/rustlib/src/rust/library";
        };
      });
}

