
# Emulator for Minirechner 2a microcomputer

## Installation

Download and install [rustup](https://rustup.rs/) for your operating system and follow the instructions on how to install [Rust](https://www.rust-lang.org/) using rustup.

Use one of the [prebuild binaries](https://v4.git.tammena.rocks/2a-emulator/2a-emulator/releases).

Or clone this repository and compile and run your own binary using [Cargo](https://github.com/rust-lang/cargo) (which should have already been installed by rustup):
```
cargo run --release --locked
```

# Compilation flags

Specify the `utf8` feature during compilation to enable UTF8 output (if your terminal supports it).
```
cargo run --release --locked --feature utf8
```
