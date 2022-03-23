{
  inputs.nixCargoIntegration.url = "github:yusdacra/nix-cargo-integration";

  outputs = inputs:
    inputs.nixCargoIntegration.lib.makeOutputs {
      root = ./.;
      renameOutputs = {
        "emulator-2a" = "2a-emulator";
        "emulator-2a-lib" = "2a-emulator-lib";
      };
      defaultOutput.app = "2a-emulator";
      defaultOutput.package = "2a-emulator";

      enablePreCommitHooks = true;

      cachix = {
        name = "2a-emulator";
        key = "2a-emulator.cachix.org-1:ijJDEqNsMqhamxxWvqOiaCQNoYhWNw7A+gGICgAH1mE=";
      };
    };
}
