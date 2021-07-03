//! [nom](https://crates.io/crates/nom)-based parser for [`Command`]s.
use nom::{
    branch::alt,
    bytes::complete::{is_a, tag, tag_no_case},
    character::complete::{digit1, hex_digit1},
    combinator::{complete, map, map_res, opt, rest, value},
    number::complete::float,
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

use super::{Command, InputRegister};
use crate::tui::Part;

fn ws(input: &str) -> IResult<&str, &str> {
    is_a(" \t")(input)
}

fn nr_hex(input: &str) -> IResult<&str, u8> {
    map_res(preceded(tag_no_case("0x"), hex_digit1), |nr| {
        u8::from_str_radix(nr, 16)
    })(input)
}

fn nr_bin(input: &str) -> IResult<&str, u8> {
    let bits = is_a("01");
    map_res(preceded(tag_no_case("0b"), bits), |nr| {
        u8::from_str_radix(nr, 2)
    })(input)
}

fn nr_dec(input: &str) -> IResult<&str, u8> {
    map_res(digit1, |nr: &str| nr.parse())(input)
}

fn nr_dec_usize(input: &str) -> IResult<&str, usize> {
    map_res(digit1, |nr: &str| nr.parse())(input)
}

fn ws_opt(input: &str) -> IResult<&str, Option<&str>> {
    opt(ws)(input)
}

fn set_ws(input: &str) -> IResult<&str, &str> {
    terminated(tag_no_case("set"), ws)(input)
}

fn unset_ws(input: &str) -> IResult<&str, &str> {
    terminated(tag_no_case("unset"), ws)(input)
}

fn value_u8(input: &str) -> IResult<&str, u8> {
    alt((nr_hex, nr_bin, nr_dec))(input)
}

fn eq_ws(input: &str) -> IResult<&str, &str> {
    delimited(ws_opt, tag("="), ws_opt)(input)
}

fn parse_part(input: &str) -> IResult<&str, Part> {
    let register = value(Part::RegisterBlock, tag_no_case("register"));
    let memory = value(Part::Memory, tag_no_case("memory"));
    let address_logic = value(Part::AddressLogic, tag_no_case("address logic"));
    alt((register, memory, address_logic))(input)
}

/// `load path/to/program`
fn cmd_load_prgm(input: &str) -> IResult<&str, Command> {
    map(tuple((tag_no_case("load"), ws, rest)), |(_, _, path)| {
        Command::LoadProgram(path)
    })(input)
}

/// `set FC = 99`
fn cmd_set_input_reg(input: &str) -> IResult<&str, Command> {
    let fc = value(InputRegister::Fc, tag_no_case("fc"));
    let fd = value(InputRegister::Fd, tag_no_case("fd"));
    let fe = value(InputRegister::Fe, tag_no_case("fe"));
    let ff = value(InputRegister::Ff, tag_no_case("ff"));
    let input_reg = alt((fc, fd, fe, ff));
    map(
        tuple((opt(set_ws), input_reg, eq_ws, value_u8)),
        |(_, reg, _, val)| Command::SetInputReg(reg, val),
    )(input)
}

/// `set IRG = 0xAB`
fn cmd_set_irg(input: &str) -> IResult<&str, Command> {
    let irg = tag_no_case("IRG");
    map(tuple((set_ws, irg, eq_ws, value_u8)), |(_, _, _, val)| {
        Command::SetIrg(val)
    })(input)
}

/// `set TEMP = 42.0`
fn cmd_set_temp(input: &str) -> IResult<&str, Command> {
    let temp = tag_no_case("TEMP");
    map(tuple((set_ws, temp, eq_ws, float)), |(_, _, _, f)| {
        Command::SetTemp(f)
    })(input)
}

/// `set I1 = 1.1` and `set I2 = 2.2`
fn cmd_set_ix(input: &str) -> IResult<&str, Command> {
    let i1 = map(tuple((tag_no_case("I1"), eq_ws, float)), |(_, _, f)| {
        Command::SetI1(f)
    });
    let i2 = map(tuple((tag_no_case("I2"), eq_ws, float)), |(_, _, f)| {
        Command::SetI2(f)
    });
    preceded(set_ws, alt((i1, i2)))(input)
}

/// `set J1` and `unset J2`
fn cmd_set_jx<'a>(input: &'a str) -> IResult<&str, Command<'a>> {
    let set_j1 = value(Command::SetJ1(true), preceded(set_ws, tag_no_case("J1")));
    let set_j2 = value(Command::SetJ2(true), preceded(set_ws, tag_no_case("J2")));
    let unset_j1 = value(Command::SetJ1(false), preceded(unset_ws, tag_no_case("J1")));
    let unset_j2 = value(Command::SetJ2(false), preceded(unset_ws, tag_no_case("J2")));

    alt((set_j1, set_j2, unset_j1, unset_j2))(input)
}

/// `set UIO1`, `unset UIO2` and same for `UIO3`
fn cmd_set_uiox(input: &str) -> IResult<&str, Command> {
    let set_uio1 = value(
        Command::SetUio1(true),
        preceded(set_ws, tag_no_case("UIO1")),
    );
    let set_uio2 = value(
        Command::SetUio2(true),
        preceded(set_ws, tag_no_case("UIO2")),
    );
    let set_uio3 = value(
        Command::SetUio3(true),
        preceded(set_ws, tag_no_case("UIO3")),
    );
    let unset_uio1 = value(
        Command::SetUio1(false),
        preceded(unset_ws, tag_no_case("UIO1")),
    );
    let unset_uio2 = value(
        Command::SetUio2(false),
        preceded(unset_ws, tag_no_case("UIO2")),
    );
    let unset_uio3 = value(
        Command::SetUio3(false),
        preceded(unset_ws, tag_no_case("UIO3")),
    );
    alt((
        set_uio1, set_uio2, set_uio3, unset_uio1, unset_uio2, unset_uio3,
    ))(input)
}

/// `show blub`
fn cmd_show(input: &str) -> IResult<&str, Command> {
    map(
        tuple((tag_no_case("show"), ws, parse_part)),
        |(_, _, part)| Command::Show(part),
    )(input)
}

/// `quit`
fn cmd_quit(input: &str) -> IResult<&str, Command> {
    let quit = tag_no_case("quit");
    let exit = tag_no_case("exit");
    value(Command::Quit, alt((quit, exit)))(input)
}

/// `next N`
fn cmd_next(input: &str) -> IResult<&str, Command> {
    let next = tag_no_case("next");
    map(preceded(next, opt(preceded(ws, nr_dec_usize))), |nr| {
        Command::Next(nr.unwrap_or(1))
    })(input)
}

pub fn parse_cmd(input: &str) -> IResult<&str, Command> {
    let cmd = alt((
        cmd_load_prgm,
        cmd_set_input_reg,
        cmd_set_irg,
        cmd_set_temp,
        cmd_set_ix,
        cmd_set_jx,
        cmd_set_uiox,
        cmd_show,
        cmd_next,
        cmd_quit,
    ));
    complete(delimited(ws_opt, cmd, ws_opt))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd_load_prgm_test() {
        let parse = cmd_load_prgm;
        use Command::*;

        assert!(parse("loadx/a b c/z").is_err());
        assert!(parse("load").is_err());
        assert_eq!(parse("load x/y/z"), Ok(("", LoadProgram("x/y/z"))));
        assert_eq!(parse("load x/a b c/z"), Ok(("", LoadProgram("x/a b c/z"))));
        assert_eq!(
            parse("load \tx/a b c/z"),
            Ok(("", LoadProgram("x/a b c/z")))
        );
        assert_eq!(
            parse("load \t x/a b c/z"),
            Ok(("", LoadProgram("x/a b c/z")))
        );
        assert_eq!(parse("load\tx/a b c/z"), Ok(("", LoadProgram("x/a b c/z"))));
    }

    #[test]
    fn cmd_set_input_reg_test() {
        let parse = cmd_set_input_reg;
        use Command::*;
        use InputRegister::*;

        assert_eq!(parse("fC =9"), Ok(("", SetInputReg(Fc, 9))));
        assert_eq!(parse("set fC = 0x12"), Ok(("", SetInputReg(Fc, 0x12))));
        assert_eq!(parse("sEt FF = 0b10"), Ok(("", SetInputReg(Ff, 0b10))));
        assert!(parse("set FB = 0x11").is_err());
    }

    #[test]
    fn cmd_set_irg_test() {
        let parse = cmd_set_irg;
        use Command::*;

        assert_eq!(parse("set irg = 0xA3"), Ok(("", SetIrg(0xA3))));
        assert_eq!(parse("set IRG = 0xA3"), Ok(("", SetIrg(0xA3))));
        assert_eq!(parse("set IRG=0x00"), Ok(("", SetIrg(0x00))));
        assert!(parse("IRG=0x00").is_err());
    }

    #[test]
    fn cmd_set_ix_test() {
        let parse = cmd_set_ix;
        use Command::*;

        assert_eq!(parse("set i1 = 1.1"), Ok(("", SetI1(1.1))));
        assert_eq!(parse("set i2 = 2.2"), Ok(("", SetI2(2.2))));
        assert!(parse("i2 = 2.2").is_err());
        assert!(parse("I=0x00").is_err());
    }

    #[test]
    fn cmd_set_jx_test() {
        let parse = cmd_set_jx;
        use Command::*;

        assert_eq!(parse("set J1"), Ok(("", SetJ1(true))));
        assert_eq!(parse("set j2"), Ok(("", SetJ2(true))));
        assert_eq!(parse("unset j1"), Ok(("", SetJ1(false))));
        assert_eq!(parse("unset J2"), Ok(("", SetJ2(false))));
        assert_eq!(parse("set j2 = true"), Ok((" = true", SetJ2(true))));
        assert!(parse("I2").is_err());
    }

    #[test]
    fn cmd_next_works() {
        let parse = cmd_next;
        use Command::*;

        assert_eq!(parse("next"), Ok(("", Next(1))));
        assert_eq!(parse("next "), Ok((" ", Next(1))));
        assert_eq!(parse("next 10"), Ok(("", Next(10))));
        assert_eq!(parse("next\t123456789"), Ok(("", Next(123456789))));
        assert_eq!(parse("next  42x"), Ok(("x", Next(42))));
    }

    #[test]
    fn cmd_set_uiox_test() {
        let parse = cmd_set_uiox;
        use Command::*;

        assert_eq!(parse("set uIO1"), Ok(("", SetUio1(true))));
        assert_eq!(parse("set UiO2"), Ok(("", SetUio2(true))));
        assert_eq!(parse("set UIo3"), Ok(("", SetUio3(true))));
        assert_eq!(parse("UNset uIO1"), Ok(("", SetUio1(false))));
        assert_eq!(parse("UNset UiO2"), Ok(("", SetUio2(false))));
        assert_eq!(parse("UNset UIo3"), Ok(("", SetUio3(false))));
        assert!(parse("UIO1").is_err());
    }

    #[test]
    fn cmd_show_test() {
        let parse = cmd_show;
        use Command::*;

        assert_eq!(parse("show memory"), Ok(("", Show(Part::Memory))));
        assert_eq!(parse("show register"), Ok(("", Show(Part::RegisterBlock))));
        assert!(parse("show foo").is_err());
    }

    #[test]
    fn cmd_quit_test() {
        let parse = cmd_quit;
        use Command::*;

        assert_eq!(parse("quit"), Ok(("", Quit)));
        assert_eq!(parse("exit"), Ok(("", Quit)));
    }

    #[test]
    fn parse_cmd_test() {
        let parse = parse_cmd;
        use Command::*;
        use InputRegister::*;

        assert_eq!(parse("load path"), Ok(("", LoadProgram("path"))));
        assert_eq!(parse("fD = 0xFE"), Ok(("", SetInputReg(Fd, 0xFE))));
        assert_eq!(parse("set IRG = 0b10101101"), Ok(("", SetIrg(0b10101101))));
        assert_eq!(parse("set TEMP = 1.234"), Ok(("", SetTemp(1.234))));
        assert_eq!(parse("set I1 = 5.678"), Ok(("", SetI1(5.678))));
        assert_eq!(parse("set I2 = 8.765"), Ok(("", SetI2(8.765))));
        assert_eq!(parse("set J1\t"), Ok(("", SetJ1(true))));
        assert_eq!(parse("set J2"), Ok(("", SetJ2(true))));
        assert_eq!(parse("unset J1"), Ok(("", SetJ1(false))));
        assert_eq!(parse("unset J2"), Ok(("", SetJ2(false))));
        assert_eq!(parse("set UIO1"), Ok(("", SetUio1(true))));
        assert_eq!(parse("\tset UIO2"), Ok(("", SetUio2(true))));
        assert_eq!(parse("set UIO3"), Ok(("", SetUio3(true))));
        assert_eq!(parse("unset UIO1"), Ok(("", SetUio1(false))));
        assert_eq!(parse("unset UIO2 "), Ok(("", SetUio2(false))));
        assert_eq!(parse("unset UIO3"), Ok(("", SetUio3(false))));
        assert_eq!(parse(" show memory"), Ok(("", Show(Part::Memory))));
        assert_eq!(parse("quit"), Ok(("", Quit)));
    }
}
