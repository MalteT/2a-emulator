# Contributing to the Emulator

Contributions to this project are very welcome. If you'd like to help by reporting a bug/glitch, wishing for a feature, creating a pull requests, or in any other way, feel free to do so.

## Pull Requests

Make sure to create an issue first, explaining what you're going to do.

The CI should automatically check your code contributions and make sure, that everything has been formatted with `rustfmt`. It will also run all tests that it can find.

### README.md

The README.md is auto-generated from the documentation of the binary in [emulator-2a/src/main.rs](emulator-2a/src/main.rs) by [`cargo-readme`](https://lib.rs/crates/cargo-readme).Make sure to update `main.rs` and generate the README.md using:
```console
cargo readme -r emulator-2a --template ../README.tpl > README.md
```
The CI should complain if `main.rs` and the `README.md` are out of sync!

:heart: Malte
