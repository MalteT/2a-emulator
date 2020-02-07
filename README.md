
# Emulator for Minirechner 2a microcomputer

This is an emulator for the Minirechner 2a, that is used during the Hardwarepraktikum at the university of Leipzig. This emulator was created for my bachelor thesis and is published under the GNU GPLv3. Please report all bugs, or no one will fix them. Just create an issue or message me.

![Demo Session](./static/demo.svg)

**Warning**: There are still some unimplemented features, ~like the temperature sensor~. Expect some bugs.

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

Simply run `2a-emulator YOUR_PROGRAM_FILE.asm` to start the interactive session. More usage information can be found with `2a-emulator --help`:
```text
2a-emulator 3.0.6
Malte Tammena <malte.tammena@gmx.de>
Emulator for the Minirechner 2a microcomputer

USAGE:
    2a-emulator [FLAGS] [OPTIONS] [--] [PROGRAM]

FLAGS:
    -c, --check          Validate the given source file. If the validation fails,
                         neither tests nor the interactive
                         session will be executed
    -h, --help           Prints help information
    -i, --interactive    Start an interactive session
        --j1             Plug jumper J1 into the board
        --no-j2          Unplug jumper J2 from the board
    -V, --version        Prints version information

OPTIONS:
        --i1 <VOLT>         Set analog input port I1 (0.0-5.0) [default: 0.0]
        --i2 <VOLT>         Set analog input port I2 (0.0-5.0) [default: 0.0]
        --irg <BYTE>        Set the value for the 8-bit input port (0-255) [default: 0]
        --temp <VOLT>       Set the value for the temperature sensor (0.0-2.55).
    -t, --test <TEST>...    Specify a test file
        --uio1 <VOLT>       Set universal analog input/output port UIO1 (0.0-5.0)
        --uio2 <VOLT>       Set universal analog input/output port UIO2 (0.0-5.0)
        --uio3 <VOLT>       Set universal analog input/output port UIO3 (0.0-5.0)

ARGS:
    <PROGRAM>    File to load and verify
```

## Compilation flags

**Warning**: The Ubuntu Terminal in combination with the Ubuntu Mono font has troubles displaying some characters. Thus the `utf8` feature should not be used on machines running Ubuntu.

Specify the `utf8` feature during compilation to enable UTF8 output (if your terminal supports it).
```console
$ cargo run --release --locked --features utf8
```
