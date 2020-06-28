
# Emulator for Minirechner 2a microcomputer ![Build](https://github.com/MalteT/2a-emulator/workflows/Build/badge.svg)

This is an emulator for the Minirechner 2a, that is used during the Hardwarepraktikum at the university of Leipzig. This emulator was created for my bachelor thesis and is published under the GNU GPLv3. Please report all bugs, or no one will fix them. Just create an issue or message me.

![Demo Session](./static/demo.svg)

**Warning**: There are still some unimplemented features. Expect some bugs.

A rendered version of the following can be found [here](./Benutzerhandbuch.pdf). It was assembled for the thesis and may be slightly out of date.


## Installation

### Prebuild binaries

Use one of the [prebuild binaries](https://github.com/MalteT/2a-emulator/releases).

### Manually install

Download and install [rustup](https://rustup.rs/) for your operating system and follow the instructions on how to install [Rust](https://www.rust-lang.org/) using rustup.

clone this repository and compile and run your own binary using [Cargo](https://github.com/rust-lang/cargo) (which should have already been installed by rustup):
```console
$ cargo run --release --locked
```

You can also install the binary using:
```console
$ cargo install --git https://github.com/MalteT/2a-emulator
```
See the [Cargo Manual](https://doc.rust-lang.org/cargo/commands/cargo-install.html?highlight=install#cargo-install) about where the binaries is installed to.

## Usage

Simply run `2a-emulator interactive YOUR_PROGRAM_FILE.asm` to start the interactive session. More usage information can be found with `2a-emulator --help`:
```text
emulator-2a 4.0.0
Malte Tammena <malte.tammena@gmx.de>
Emulator for the Minirechner 2a microcomputer.

If run without arguments an interactive session is started.

USAGE:
    2a-emulator [SUBCOMMAND]

FLAGS:
    -h, --help
            Prints help information

    -V, --version
            Prints version information


SUBCOMMANDS:
    help           Prints this message or the help of the given subcommand(s)
    interactive    Run an interactive session
    run            Run a single emulation
    test           Run tests against a program
    verify         Verify the given program's syntax
```

## Compilation flags

**Warning**: The Ubuntu Terminal in combination with the Ubuntu Mono font has troubles displaying some characters. Thus the `utf8` feature should not be used on machines running Ubuntu.

Specify the `utf8` feature during compilation to enable UTF8 output (if your terminal supports it).
```console
$ cargo run --release --locked --features utf8
```
