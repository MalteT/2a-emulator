{
  pkgs,
  perSystem,
  ...
}:
pkgs.mkShell {
  inputsFrom = [perSystem.self."2a-emulator"];
  packages = [
    perSystem.self.formatter
    pkgs.cargo-workspaces
    # Brings clippy, rustfmt and rust-analyzer of the pinned version.
    perSystem.self."2a-emulator".toolchain
    pkgs.nil
  ];
}
