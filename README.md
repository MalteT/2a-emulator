![Build](https://github.com/MalteT/2a-emulator/workflows/Build/badge.svg)

## Emulator for the Minirechner 2a microcomputer
This is an emulator for the Minirechner 2a microcomputer. The microcomputer is used during a practical course at the university of Leipzig. This emulator was created for my bachelor thesis and is published under the GNU GPLv3. It's main purpose is to aid students in creating correct solutions for the assignments that are part of the course. it was meant to solve the issue of not having the real hardware at hand while coming up with solutions for the given problems. This FOSS should run on most major plattforms and is versatile enough to test solutions.

**Please report all bugs! Just create an issue or message me!**

A rendered version of the following can be found [here](./Benutzerhandbuch.pdf). It was assembled for the thesis and *is heavily* out of date.


### Installation

#### Prebuild binaries

To use one of the prebuild binaries, you can download one of the [release packages](https://github.com/MalteT/2a-emulator/releases). These are always a little outdated, for more up-to-date versions of the emulator download one of the [artifacts created by the CI](https://github.com/MalteT/2a-emulator/actions)!

#### The `cargo` way

Download and install [rustup](https://rustup.rs/) for your operating system and follow the instructions on how to install [Rust](https://www.rust-lang.org/) using rustup.

clone this repository and compile and run your own binary using [Cargo](https://github.com/rust-lang/cargo) (which should have already been installed by rustup):
```console
$ cargo run --release --locked
```

You may also install the binary using:
```console
$ cargo install --git https://github.com/MalteT/2a-emulator
```
See the [Cargo Manual](https://doc.rust-lang.org/cargo/commands/cargo-install.html?highlight=install#cargo-install) on where the binary is installed to.

### Usage

To simply start an interactive session, run `2a-emulator` in the terminal. If you want to adjust some options before starting the interface, supply them on the command line like this:
```console
$ 2a-emulator interactive --ff 42 path_to_a_program.asm
```
The above runs an interactive session with `path_to_a_program.asm` compiled and loaded into main memory. Additionally the input register FF contains 42. For a full list of options, see `2a-emulator interactive --help`.

You may also be interested in the `run` mode which executes a given program for a number of clock cycles before printing the state of the machine. Have a look at `2a-emulator run --help` for more information. To simply verify the syntax of an assembly file run `2a-emulator verify my_faulty_program.asm`.

### Compilation flags

The following feature flags can be used to influence the generated binary. *See [here](https://doc.rust-lang.org/cargo/reference/features.html)*.

- `interactive-tui` (*opt-out*) enables the interactive session. Without it, no interactive session is possible.
- `utf8` (*opt-in*) enables the use of character codes which are supported by fewer terminals. Note, that at the moment the difference is marginal.

  **Warning**: The Ubuntu Terminal in combination with the Ubuntu Mono font has troubles displaying some characters. Thus the `utf8` feature should not be used on machines running Ubuntu.

Current version: 4.1.0

License: GPL-3.0
