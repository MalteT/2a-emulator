{
  pkgs,
  inputs,
  flake,
  ...
}: let
  treefmt = inputs.treefmt-nix.lib.evalModule pkgs {
    projectRootFile = "flake.nix";
    programs.alejandra.enable = true;
    programs.rustfmt.enable = true;
    # Without this rustfmt formats as the newest edition, but the crates are 2018.
    programs.rustfmt.edition = "2018";
    # The RAM content has syntax issues.
    settings.global.excludes = ["emulator-2a-lib/src/machine/microprogram_ram_content.rs"];
  };
in
  treefmt.config.build.wrapper
  // {
    passthru =
      treefmt.config.build.wrapper.passthru
      // {
        tests.check = treefmt.config.build.check flake;
      };
  }
