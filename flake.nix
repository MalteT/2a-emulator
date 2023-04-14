{
  inputs.dream2nix.url = "github:nix-community/dream2nix";
  inputs.devshell.url = "github:numtide/devshell";
  inputs.flake-parts.url = "github:hercules-ci/flake-parts";
  inputs.treefmt-nix.url = "github:numtide/treefmt-nix";
  inputs.pre-commit-hooks-nix.url = "github:cachix/pre-commit-hooks.nix";
  inputs.nixpkgs.url = "nixpkgs";

  outputs = inputs @ {
    flake-parts,
    dream2nix,
    devshell,
    treefmt-nix,
    pre-commit-hooks-nix,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        dream2nix.flakeModuleBeta
        devshell.flakeModule
        treefmt-nix.flakeModule
        pre-commit-hooks-nix.flakeModule
      ];
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      perSystem = {
        self',
        pkgs,
        config,
        ...
      }: {
        dream2nix.inputs = {
          emulator-2a = {
            source = ./.;
            projects = fromTOML (builtins.readFile ./projects.toml);
          };
        };
        packages."2a-emulator" = config.dream2nix.outputs.emulator-2a.packages.emulator-2a;
        packages.default = self'.packages."2a-emulator";
        apps."2a-emulator" = {
          type = "app";
          program = "${self'.packages."2a-emulator"}/bin/2a-emulator";
        };
        apps.default = self'.apps."2a-emulator";
        devShells = {
          default = config.dream2nix.outputs.emulator-2a.devShells.default.overrideAttrs (old: {
            buildInputs =
              old.buildInputs
              ++ [
                # Additional packages for the shell
                config.treefmt.package
                pkgs.nil
                pkgs.cargo-workspaces
                pkgs.rust-analyzer
              ];
          });
        };
        treefmt.projectRootFile = "flake.nix";
        # The RAM content has syntax issues
        treefmt.settings.global.excludes = [
          "./emulator-2a-lib/src/machine/microprogram_ram_content.rs"
        ];
        treefmt.programs = {
          rustfmt.enable = true;
          alejandra.enable = true;
        };
      };
    };
}
