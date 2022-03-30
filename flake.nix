{
  inputs.nixCargoIntegration.url = "github:yusdacra/nix-cargo-integration/release-1.0";

  outputs = inputs:
    inputs.nixCargoIntegration.lib.makeOutputs {
      root = ./.;
      renameOutputs = {
        "emulator-2a" = "2a-emulator";
        "emulator-2a-lib" = "2a-emulator-lib";
      };
      defaultOutputs.app = "2a-emulator";
      defaultOutputs.package = "2a-emulator";

      #enablePreCommitHooks = true;

      # TODO: Reenable when updating nix-cargo-integration
      # cachix = {
      #   name = "2a-emulator";
      #   key = "2a-emulator.cachix.org-1:ijJDEqNsMqhamxxWvqOiaCQNoYhWNw7A+gGICgAH1mE=";
      # };
    };
}
