
# Emulator for Minirechner 2a microcomputer

This is an emulator for the Minirechner 2a, that is used during the Hardwarepraktikum at the university of Leipzig. This emulator was created for my bachelor thesis and is published under the GNU GPLv3. Please report all bugs, or no one will fix them. Just create an issue or write a message to me.

![Demo image](/static/demo.png)

**Warning**: There are still some unimplemented features, like the temperature sensor. Expect some bugs.


## Installation

### Prebuild binaries

Use one of the [prebuild binaries](https://v4.git.tammena.rocks/2a-emulator/2a-emulator/releases).

### Manually install

Download and install [rustup](https://rustup.rs/) for your operating system and follow the instructions on how to install [Rust](https://www.rust-lang.org/) using rustup.

clone this repository and compile and run your own binary using [Cargo](https://github.com/rust-lang/cargo) (which should have already been installed by rustup):
```console
$ cargo run --release --locked
```

You can also install the binary using:
```console
$ cargo install --git https://v4.git.tammena.rocks/2a-emulator/2a-emulator
```
See the [Cargo Manual](https://doc.rust-lang.org/cargo/commands/cargo-install.html?highlight=install#cargo-install) about where the binaries is installed to.

## Usage

Simply run `2a-emulator YOUR_PROGRAM_FILE.asm` to start the interactive session. More usage information can be found with `2a-emulator --help`:
```text
2a-emulator 0.10.0
Malte Tammena <malte.tammena@gmx.de>
Emulator for the Minirechner 2a microcomputer

USAGE:
    2a-emulator [FLAGS] [OPTIONS] [--] [PROGRAM]

FLAGS:
    -c, --check          Validate the given source file. If the validation fails, neither tests nor the interactive
                         session will be executed
    -h, --help           Prints help information
    -i, --interactive    Start an interactive session
    -V, --version        Prints version information

OPTIONS:
    -t, --test <TEST>...    Specify a test file

ARGS:
    <PROGRAM>    File to load and verify
```

## Compilation flags

Specify the `utf8` feature during compilation to enable UTF8 output (if your terminal supports it).
```console
$ cargo run --release --locked --feature utf8
```
