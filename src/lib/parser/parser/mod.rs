//! # Minirechner 2a assembly parsing
//!
//! For a complete reference of the assembly syntax
//! see the official resources by Werner Dreher or
//! the Manual by Max Braungardt and Thomas Schmid.
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

use crate::asm::*;

mod error;
#[cfg(test)]
mod tests;

pub use error::ParserError;
type ParseResult<T> = Result<T, ParserError>;

/// Parser for valid Minirechner 2a assembly files.
/// See [module documentation](crate::parser) for more information.
#[derive(Parser)]
#[grammar = "../mrasm.pest"]
pub struct AsmParser;

/// Parse inner elements of a [`Pair`] into a tuple.
///
/// # Example
///
/// ```no_run
/// let (_, label, _, constant) = inner_tuple! { equ;
///     sep_ip      => ignore;
///     raw_label   => parse_raw_label;
///     sep_pp      => ignore;
///     constant    => parse_constant;
/// };
/// ```
macro_rules! inner_tuple {
    ($rule:expr;
     $($($expected:pat )|+ => $function:ident $( | $error:literal)?);* $(;)?) => {
        {
            let mut inner = $rule.into_inner();
            // return tuple
            (
                $(
                    {
                        #[allow(unused_variables)]
                        let e1 = format!("No inner rule. '{:?}' expected '{}'",
                                         $rule,
                                         stringify!($($expected),+));
                        #[allow(unused_variables)]
                        let e2 = format!("Wrong rule found inside '{:?}'. Expected '{:?}'",
                                         $rule,
                                         stringify!($($expected),+));
                        $(let e1: String = $error.into();)?
                        $(let e2: String = $error.into();)?
                        let inner = inner.next().expect(&e1);
                        use Rule::*;
                        #[allow(unreachable_patterns)]
                        match inner.as_rule() {
                            $($expected)|+ => $function(inner),
                            _ => panic!(e2)
                        }
                    }
                ),*
            )
        }
    }
}

/// Helper function for [`inner_tuple`] macro.
/// This function is the identity for all input.
fn id<T>(element: T) -> T {
    element
}

/// Helper function for [`inner_tuple`] macro.
/// This function discards it's input.
fn ignore<T>(_: T) -> () {
    ()
}

impl AsmParser {
    /// Parse a valid Minirechner 2a assembly file.
    ///
    /// # Checks
    /// 1) **Syntax** Is it a valid file?
    /// 2) **Labels** Are all used labels defined?
    ///
    /// # Arguments
    /// - `input`: The [`str`] to parse.
    ///
    /// # Returns
    /// - The parsed [`assembler program`](Asm) or
    /// - a [`ParserError`]
    pub fn parse(input: &str) -> ParseResult<Asm> {
        let mut lines = vec![];
        let mut parsed = <Self as Parser<Rule>>::parse(Rule::file, input)?;
        // Get the header of the asm file
        let header = parsed.next().expect("Infallible: Header must exist");
        // Extract the optional comment from the header file
        let mut comment_after_shebang = None;
        for el in header.into_inner() {
            if el.as_rule() == Rule::comment {
                comment_after_shebang = Some(parse_comment(el));
            }
        }
        // iterate over lines, skipping the header
        for line in parsed {
            if line.as_rule() == Rule::line {
                lines.push(parse_line(line));
            }
        }
        // Do some checks
        validate_lines(&lines)?;
        Ok(Asm {
            lines,
            comment_after_shebang,
        })
    }
}
/// Parse an assembler instruction line into a valid type.
///
/// # Arguments
/// - `line`: The [`Rule`] pair to be pressed into form.
///
/// # Returns
/// - Some tuple of a [`Line`] and a [`Comment`] or
/// - None, if the line is empty or
/// - a [`ParserError`]
fn parse_line(line: Pair<Rule>) -> Line {
    let line = line.into_inner();
    let mut ret = Line::Empty(None);
    // Possible elements in a line:
    // - space
    // - label
    // - instruction
    // - comment
    // Mutate ret accordingly
    for element in line {
        ret = match element.as_rule() {
            Rule::space => ret,
            // The label or instruction rule comes first and they occur
            // exclusive so replacing is just fine.
            Rule::label => Line::Label(parse_label(element), None),
            Rule::instruction => Line::Instruction(parse_instruction(element), None),
            // comment can only occur once.
            // So it has to be THE comment.
            Rule::comment => {
                let c = Some(parse_comment(element));
                match ret {
                    Line::Empty(_) => Line::Empty(c),
                    Line::Instruction(i, _) => Line::Instruction(i, c),
                    Line::Label(l, _) => Line::Label(l, c),
                }
            }
            _ => unreachable!(),
        }
    }
    ret
}
/// Do some validity checking on the given lines.
///
/// # Checks
/// - Undefined Labels
/// - Too many Labels
fn validate_lines(lines: &Vec<Line>) -> Result<(), ParserError> {
    // Collect labels
    let mut labels = vec![];
    for line in lines {
        match line {
            Line::Label(label, _) => labels.push(label.to_lowercase()),
            Line::Instruction(inst, _) => match inst {
                Instruction::AsmEquals(label, _) => labels.push(label.to_lowercase()),
                _ => {}
            },
            _ => {}
        }
    }
    // Check for undefined labels
    let mut undefined_labels: Vec<String> = vec![];
    // Function to map a Constant to a vec of labels
    let const_to_vec = |c: &Constant| match c {
        Constant::Label(label) => vec![label.clone()],
        Constant::Constant(_) => vec![],
    };
    // Function to map a Memory to a vec of labels
    let mem_to_vec = |c: &MemAddress| match c {
        MemAddress::Constant(c) => const_to_vec(&c),
        MemAddress::Register(_) => vec![],
    };
    // Function to map a Source to a vec of labels
    let src_to_vec = |src: &Source| match src {
        Source::MemAddress(mem) => mem_to_vec(mem),
        Source::Constant(c) => const_to_vec(c),
        _ => vec![],
    };
    // Function to map a Destination to a vec of labels
    let dst_to_vec = |dst: &Destination| match dst {
        Destination::MemAddress(mem) => mem_to_vec(mem),
        _ => vec![],
    };
    for line in lines {
        let mut refs = match line {
            Line::Instruction(inst, _) => match inst {
                Instruction::Jmp(label)
                | Instruction::Jcs(label)
                | Instruction::Jcc(label)
                | Instruction::Jzs(label)
                | Instruction::Jzc(label)
                | Instruction::Jns(label)
                | Instruction::Jnc(label)
                | Instruction::Jr(label)
                | Instruction::Call(label) => vec![label.clone()],
                Instruction::LdConstant(_, c) => const_to_vec(c),
                Instruction::AsmDefineBytes(constants) => {
                    constants.iter().map(const_to_vec).flatten().collect()
                }
                Instruction::LdMemAddress(_, mem) | Instruction::St(mem, _) => mem_to_vec(mem),
                Instruction::Dec(src) | Instruction::Ldsp(src) | Instruction::Ldfr(src) => {
                    src_to_vec(src)
                }
                Instruction::Bits(dst, src)
                | Instruction::Bitc(dst, src)
                | Instruction::Cmp(dst, src)
                | Instruction::Bitt(dst, src)
                | Instruction::Mov(dst, src) => {
                    let mut labels = dst_to_vec(dst);
                    labels.append(&mut src_to_vec(src));
                    labels
                }
                _ => vec![],
            },
            _ => vec![],
        };
        // Check if labels exist and add missing ones to the list of undefined labels
        let mut refs = refs
            .drain(..)
            .filter(|label| !labels.contains(&label.to_lowercase()))
            .collect();
        undefined_labels.append(&mut refs)
    }
    if labels.len() > 40 {
        Err(ParserError::TooManyLabels)
    } else if undefined_labels.is_empty() {
        Ok(())
    } else {
        Err(ParserError::UndefinedLabels(undefined_labels))
    }
}
/// Parse a `label` rule into a [`Label`].
fn parse_label(label: Pair<Rule>) -> Label {
    let (label, _) = inner_tuple! { label;
        raw_label => parse_raw_label;
        colon     => ignore
    };
    label
}
/// Parse a `raw_label` rule into a [`Label`].
fn parse_raw_label(label: Pair<Rule>) -> Label {
    label.as_str().into()
}
/// Parse an `instruction` rule into an [`Instruction`].
fn parse_instruction(instruction: Pair<Rule>) -> Instruction {
    let instruction = instruction
        .into_inner()
        .next()
        .expect("an instruction rule should have an actual instruction");
    let instruction = match instruction.as_rule() {
        Rule::org => parse_instruction_org(instruction),
        Rule::byte => parse_instruction_byte(instruction),
        Rule::db => parse_instruction_db(instruction),
        Rule::equ => parse_instruction_equ(instruction),
        Rule::stacksize => parse_instruction_stacksize(instruction),
        Rule::clr => parse_instruction_clr(instruction),
        Rule::add => parse_instruction_add(instruction),
        Rule::adc => parse_instruction_adc(instruction),
        Rule::sub => parse_instruction_sub(instruction),
        Rule::mul => parse_instruction_mul(instruction),
        Rule::div => parse_instruction_div(instruction),
        Rule::inc => parse_instruction_inc(instruction),
        Rule::dec => parse_instruction_dec(instruction),
        Rule::neg => parse_instruction_neg(instruction),
        Rule::and => parse_instruction_and(instruction),
        Rule::or => parse_instruction_or(instruction),
        Rule::xor => parse_instruction_xor(instruction),
        Rule::com => parse_instruction_com(instruction),
        Rule::bits => parse_instruction_bits(instruction),
        Rule::bitc => parse_instruction_bitc(instruction),
        Rule::tst => parse_instruction_tst(instruction),
        Rule::cmp => parse_instruction_cmp(instruction),
        Rule::bitt => parse_instruction_bitt(instruction),
        Rule::lsr => parse_instruction_lsr(instruction),
        Rule::asr => parse_instruction_asr(instruction),
        Rule::lsl => parse_instruction_lsl(instruction),
        Rule::rrc => parse_instruction_rrc(instruction),
        Rule::rlc => parse_instruction_rlc(instruction),
        Rule::mov => parse_instruction_mov(instruction),
        Rule::ld_const => parse_instruction_ld_const(instruction),
        Rule::ld_memory => parse_instruction_ld_memory(instruction),
        Rule::st => parse_instruction_st(instruction),
        Rule::push => parse_instruction_push(instruction),
        Rule::pop => parse_instruction_pop(instruction),
        Rule::pushf => parse_instruction_pushf(),
        Rule::popf => parse_instruction_popf(),
        Rule::ldsp => parse_instruction_ldsp(instruction),
        Rule::ldfr => parse_instruction_ldfr(instruction),
        Rule::jmp => parse_instruction_jmp(instruction),
        Rule::jcs => parse_instruction_jcs(instruction),
        Rule::jcc => parse_instruction_jcc(instruction),
        Rule::jzs => parse_instruction_jzs(instruction),
        Rule::jzc => parse_instruction_jzc(instruction),
        Rule::jns => parse_instruction_jns(instruction),
        Rule::jnc => parse_instruction_jnc(instruction),
        Rule::jr => parse_instruction_jr(instruction),
        Rule::call => parse_instruction_call(instruction),
        Rule::ret => parse_instruction_ret(),
        Rule::reti => parse_instruction_reti(),
        Rule::stop => parse_instruction_stop(),
        Rule::nop => parse_instruction_nop(),
        Rule::ei => parse_instruction_ei(),
        Rule::di => parse_instruction_di(),
        _ => unreachable!(),
    };
    instruction
}
/// Parse an `org` rule into an [`Instruction`].
fn parse_instruction_org(org: Pair<Rule>) -> Instruction {
    let (_, number) = inner_tuple! { org;
        sep_ip => ignore;
        constant_bin | constant_hex | constant_dec => id;
    };
    let number = match number.as_rule() {
        Rule::constant_bin => u8::from_str_radix(&number.as_str()[2..], 2).unwrap(),
        Rule::constant_hex => u8::from_str_radix(&number.as_str()[2..], 16).unwrap(),
        Rule::constant_dec => u8::from_str_radix(&number.as_str(), 10).unwrap(),
        _ => unreachable!(),
    };
    Instruction::AsmOrigin(number)
}
/// Parse a `constant` rule into a [`Constant`].
fn parse_constant(constant: Pair<Rule>) -> Constant {
    let inner = inner_tuple! { constant;
        constant_bin | constant_hex | constant_dec | raw_label => id;
    };
    match inner.as_rule() {
        Rule::constant_bin => u8::from_str_radix(&inner.as_str()[2..], 2)
            .map(|nr| Constant::Constant(nr))
            .unwrap(),
        Rule::constant_hex => u8::from_str_radix(&inner.as_str()[2..], 16)
            .map(|nr| Constant::Constant(nr))
            .unwrap(),
        Rule::constant_dec => u8::from_str_radix(&inner.as_str(), 10)
            .map(|nr| Constant::Constant(nr))
            .unwrap(),
        Rule::raw_label => Constant::Label(parse_raw_label(inner)),
        _ => unreachable!(),
    }
}
/// Parse a `byte` rule into an [`Instruction`].
fn parse_instruction_byte(byte: Pair<Rule>) -> Instruction {
    let (_, number) = inner_tuple! { byte;
        sep_ip => ignore;
        constant_bin | constant_hex | constant_dec => id;
    };
    let number = match number.as_rule() {
        Rule::constant_bin => u8::from_str_radix(&number.as_str()[2..], 2).unwrap(),
        Rule::constant_hex => u8::from_str_radix(&number.as_str()[2..], 16).unwrap(),
        Rule::constant_dec => u8::from_str_radix(&number.as_str(), 10).unwrap(),
        _ => unreachable!(),
    };
    Instruction::AsmByte(number)
}
/// Parse a `db` rule into an [`Instruction`].
fn parse_instruction_db(db: Pair<Rule>) -> Instruction {
    let results = db
        .into_inner()
        .filter(|pair| pair.as_rule() == Rule::constant)
        .map(|constant| parse_constant(constant));
    let constants = results.collect();
    Instruction::AsmDefineBytes(constants)
}
/// Parse an `equ` rule into an [`Instruction`].
fn parse_instruction_equ(equ: Pair<Rule>) -> Instruction {
    let (_, label, _, constant) = inner_tuple! { equ;
        sep_ip      => ignore;
        raw_label   => parse_raw_label;
        sep_pp      => ignore;
        constant    => parse_constant;
    };
    Instruction::AsmEquals(label, constant)
}
/// Parse a `stacksize` rule into an [`Instruction`].
fn parse_instruction_stacksize(instruction: Pair<Rule>) -> Instruction {
    let (_, stacksize) = inner_tuple! { instruction;
        sep_ip          => ignore;
        raw_stacksize   => parse_raw_stacksize;
    };
    Instruction::AsmStacksize(stacksize)
}
/// Parse a `raw_stacksize` rule into a [`Stacksize`].
fn parse_raw_stacksize(stacksize: Pair<Rule>) -> Stacksize {
    let stacksize = stacksize.as_str().to_lowercase();
    match stacksize.as_str() {
        "16" => Stacksize::_16,
        "32" => Stacksize::_32,
        "48" => Stacksize::_48,
        "64" => Stacksize::_64,
        "noset" => Stacksize::NotSet,
        _ => unreachable!(),
    }
}
/// Parse a `clr` rule into an [`Instruction`].
fn parse_instruction_clr(instruction: Pair<Rule>) -> Instruction {
    let (_, register) = inner_tuple! { instruction;
        sep_ip      => ignore;
        register    => parse_register;
    };
    Instruction::Clr(register)
}
/// Parse a `register` rule into a [`Register`].
fn parse_register(register: Pair<Rule>) -> Register {
    let reg = register.as_str().to_lowercase();
    match reg.as_str() {
        "r0" => Register::R0,
        "r1" => Register::R1,
        "r2" => Register::R2,
        "r3" => Register::R3,
        "pc" => Register::R3,
        _ => unreachable!(),
    }
}
/// Parse an `add` rule into an [`Instruction`].
fn parse_instruction_add(add: Pair<Rule>) -> Instruction {
    let (_, reg1, _, reg2) = inner_tuple! { add;
        sep_ip      => ignore;
        register    => parse_register;
        sep_pp      => ignore;
        register    => parse_register;
    };
    Instruction::Add(reg1, reg2)
}
/// Parse an `adc` rule into an [`Instruction`].
fn parse_instruction_adc(adc: Pair<Rule>) -> Instruction {
    let (_, reg1, _, reg2) = inner_tuple! { adc;
        sep_ip      => ignore;
        register    => parse_register;
        sep_pp      => ignore;
        register    => parse_register;
    };
    Instruction::Adc(reg1, reg2)
}
/// Parse a `sub` rule into an [`Instruction`].
fn parse_instruction_sub(sub: Pair<Rule>) -> Instruction {
    let (_, reg1, _, reg2) = inner_tuple! { sub;
        sep_ip      => ignore;
        register    => parse_register;
        sep_pp      => ignore;
        register    => parse_register;
    };
    Instruction::Sub(reg1, reg2)
}
/// Parse a `mul` rule into an [`Instruction`].
fn parse_instruction_mul(mul: Pair<Rule>) -> Instruction {
    let (_, reg1, _, reg2) = inner_tuple! { mul;
        sep_ip      => ignore;
        register    => parse_register;
        sep_pp      => ignore;
        register    => parse_register;
    };
    Instruction::Mul(reg1, reg2)
}
/// Parse a `div` rule into an [`Instruction`].
fn parse_instruction_div(div: Pair<Rule>) -> Instruction {
    let (_, reg1, _, reg2) = inner_tuple! { div;
        sep_ip      => ignore;
        register    => parse_register;
        sep_pp      => ignore;
        register    => parse_register;
    };
    Instruction::Div(reg1, reg2)
}
/// Parse an `inc` rule into an [`Instruction`].
fn parse_instruction_inc(inc: Pair<Rule>) -> Instruction {
    let (_, reg) = inner_tuple! { inc;
        sep_ip      => ignore;
        register    => parse_register;
    };
    Instruction::Inc(reg)
}
/// Parse a `dec` rule into an [`Instruction`].
fn parse_instruction_dec(dec: Pair<Rule>) -> Instruction {
    let (_, source) = inner_tuple! { dec;
        sep_ip     => ignore;
        source     => parse_source;
    };
    Instruction::Dec(source)
}
/// Parse a `source` rule into a [`Source`].
fn parse_source(source: Pair<Rule>) -> Source {
    let inner = source
        .into_inner()
        .next()
        .expect("source needs an inner element");
    match inner.as_rule() {
        Rule::register => parse_register(inner).into(),
        Rule::registerdi => parse_register_di(inner).into(),
        Rule::registerddi => parse_register_ddi(inner).into(),
        Rule::memory => parse_memory(inner).into(),
        Rule::constant => parse_constant(inner).into(),
        _ => unreachable!(),
    }
}
/// Parse a `registerdi` rule into a [`RegisterDI`].
fn parse_register_di(registerdi: Pair<Rule>) -> RegisterDI {
    let (_, register, _, _) = inner_tuple! { registerdi;
        oparen      => ignore;
        register    => parse_register;
        plus        => ignore;
        cparen      => ignore;
    };
    register.into()
}
/// Parse a `registerddi` rule into a [`RegisterDDI`].
fn parse_register_ddi(registerddi: Pair<Rule>) -> RegisterDDI {
    let (_, register, _) = inner_tuple! { registerddi;
        oparen      => ignore;
        registerdi  => parse_register_di;
        cparen      => ignore;
    };
    register.into()
}
/// Parse a `memory` rule into a [`MemAddress`].
fn parse_memory(memory: Pair<Rule>) -> MemAddress {
    let (_, inner, _) = inner_tuple! { memory;
        oparen                                                  => ignore;
        register | registerdi | registerddi | memory | constant => id;
        cparen                                                  => ignore;
    };
    let memory = match inner.as_rule() {
        Rule::constant => parse_constant(inner).into(),
        Rule::register => parse_register(inner).into(),
        Rule::raw_label => {
            let constant: Constant = parse_raw_label(inner).into();
            constant.into()
        }
        _ => unreachable!(),
    };
    memory
}
/// Parse a `neg` rule into an [`Instruction`].
fn parse_instruction_neg(neg: Pair<Rule>) -> Instruction {
    let (_, reg) = inner_tuple! { neg;
        sep_ip      => ignore;
        register    => parse_register;
    };
    Instruction::Neg(reg)
}
/// Parse an `and` rule into an [`Instruction`].
fn parse_instruction_and(and: Pair<Rule>) -> Instruction {
    let (_, reg1, _, reg2) = inner_tuple! { and;
        sep_ip      => ignore;
        register    => parse_register;
        sep_pp      => ignore;
        register    => parse_register;
    };
    Instruction::And(reg1, reg2)
}
/// Parse an `or` rule into an [`Instruction`].
fn parse_instruction_or(or: Pair<Rule>) -> Instruction {
    let (_, reg1, _, reg2) = inner_tuple! { or;
        sep_ip      => ignore;
        register    => parse_register;
        sep_pp      => ignore;
        register    => parse_register;
    };
    Instruction::Or(reg1, reg2)
}
/// Parse an `xor` rule into an [`Instruction`].
fn parse_instruction_xor(xor: Pair<Rule>) -> Instruction {
    let (_, reg1, _, reg2) = inner_tuple! { xor;
        sep_ip      => ignore;
        register    => parse_register;
        sep_pp      => ignore;
        register    => parse_register;
    };
    Instruction::Xor(reg1, reg2)
}
/// Parse a `com` rule into an [`Instruction`].
fn parse_instruction_com(com: Pair<Rule>) -> Instruction {
    let (_, reg) = inner_tuple! { com;
        sep_ip      => ignore;
        register    => parse_register;
    };
    Instruction::Com(reg)
}
/// Parse a `bits` rule into an [`Instruction`].
fn parse_instruction_bits(bits: Pair<Rule>) -> Instruction {
    let (_, dst, _, src) = inner_tuple! { bits;
        sep_ip      => ignore;
        destination => parse_destination;
        sep_pp      => ignore;
        source      => parse_source;
    };
    Instruction::Bits(dst, src)
}
/// Parse a `destination` rule into a [`Destination`].
fn parse_destination(destination: Pair<Rule>) -> Destination {
    let inner = destination
        .into_inner()
        .next()
        .expect("source needs an inner element");
    match inner.as_rule() {
        Rule::register => parse_register(inner).into(),
        Rule::registerdi => parse_register_di(inner).into(),
        Rule::registerddi => parse_register_ddi(inner).into(),
        Rule::memory => parse_memory(inner).into(),
        _ => unreachable!(),
    }
}
/// Parse a `bitc` rule into an [`Instruction`].
fn parse_instruction_bitc(bitc: Pair<Rule>) -> Instruction {
    let (_, dst, _, src) = inner_tuple! { bitc;
        sep_ip      => ignore;
        destination => parse_destination;
        sep_pp      => ignore;
        source      => parse_source;
    };
    Instruction::Bitc(dst, src)
}
/// Parse a `tst` rule into an [`Instruction`].
fn parse_instruction_tst(tst: Pair<Rule>) -> Instruction {
    let (_, reg) = inner_tuple! { tst;
        sep_ip      => ignore;
        register    => parse_register;
    };
    Instruction::Tst(reg)
}
/// Parse a `cmp` rule into an [`Instruction`].
fn parse_instruction_cmp(cmp: Pair<Rule>) -> Instruction {
    let (_, dst, _, src) = inner_tuple! { cmp;
        sep_ip      => ignore;
        destination => parse_destination;
        sep_pp      => ignore;
        source      => parse_source;
    };
    Instruction::Cmp(dst, src)
}
/// Parse a `bitt` rule into an [`Instruction`].
fn parse_instruction_bitt(bitt: Pair<Rule>) -> Instruction {
    let (_, dst, _, src) = inner_tuple! { bitt;
        sep_ip      => ignore;
        destination => parse_destination;
        sep_pp      => ignore;
        source      => parse_source;
    };
    Instruction::Bitt(dst, src)
}
/// Parse a `lsr` rule into an [`Instruction`].
fn parse_instruction_lsr(lsr: Pair<Rule>) -> Instruction {
    let (_, reg) = inner_tuple! { lsr;
        sep_ip      => ignore;
        register    => parse_register;
    };
    Instruction::Lsr(reg)
}
/// Parse an `asr` rule into an [`Instruction`].
fn parse_instruction_asr(asr: Pair<Rule>) -> Instruction {
    let (_, reg) = inner_tuple! { asr;
        sep_ip      => ignore;
        register    => parse_register;
    };
    Instruction::Asr(reg)
}
/// Parse a `lsl` rule into an [`Instruction`].
fn parse_instruction_lsl(lsl: Pair<Rule>) -> Instruction {
    let (_, reg) = inner_tuple! { lsl;
        sep_ip      => ignore;
        register    => parse_register;
    };
    Instruction::Lsl(reg)
}
/// Parse an `rrc` rule into an [`Instruction`].
fn parse_instruction_rrc(rrc: Pair<Rule>) -> Instruction {
    let (_, reg) = inner_tuple! { rrc;
        sep_ip      => ignore;
        register    => parse_register;
    };
    Instruction::Rrc(reg)
}
/// Parse an `rlc` rule into an [`Instruction`].
fn parse_instruction_rlc(rlc: Pair<Rule>) -> Instruction {
    let (_, reg) = inner_tuple! { rlc;
        sep_ip      => ignore;
        register    => parse_register;
    };
    Instruction::Rlc(reg)
}
/// Parse a `mov` rule into an [`Instruction`].
fn parse_instruction_mov(mov: Pair<Rule>) -> Instruction {
    let (_, dst, _, src) = inner_tuple! { mov;
        sep_ip      => ignore;
        destination => parse_destination;
        sep_pp      => ignore;
        source      => parse_source;
    };
    Instruction::Mov(dst, src)
}
/// Parse an `ld_const` rule into an [`Instruction`].
fn parse_instruction_ld_const(ld_const: Pair<Rule>) -> Instruction {
    let (_, reg, _, constant) = inner_tuple! { ld_const;
        sep_ip      => ignore;
        register    => parse_register;
        sep_pp      => ignore;
        constant    => parse_constant;
    };
    Instruction::LdConstant(reg, constant)
}
/// Parse an `ld_memory` rule into an [`Instruction`].
fn parse_instruction_ld_memory(ld_memory: Pair<Rule>) -> Instruction {
    let (_, reg, _, mem) = inner_tuple! { ld_memory;
        sep_ip      => ignore;
        register    => parse_register;
        sep_pp      => ignore;
        memory      => parse_memory;
    };
    Instruction::LdMemAddress(reg, mem)
}
/// Parse an `st` rule into an [`Instruction`].
fn parse_instruction_st(st: Pair<Rule>) -> Instruction {
    let (_, mem, _, reg) = inner_tuple! { st;
        sep_ip      => ignore;
        memory      => parse_memory;
        sep_pp      => ignore;
        register    => parse_register;
    };
    Instruction::St(mem, reg)
}
/// Parse a `push` rule into an [`Instruction`].
fn parse_instruction_push(push: Pair<Rule>) -> Instruction {
    let (_, reg) = inner_tuple! { push;
        sep_ip         => ignore;
        register    => parse_register;
    };
    Instruction::Push(reg)
}
/// Parse a `pop` rule into an [`Instruction`].
fn parse_instruction_pop(pop: Pair<Rule>) -> Instruction {
    let (_, reg) = inner_tuple! { pop;
        sep_ip         => ignore;
        register    => parse_register;
    };
    Instruction::Pop(reg)
}
/// Parse a `pushf` rule into an [`Instruction`].
fn parse_instruction_pushf() -> Instruction {
    Instruction::PushF
}
/// Parse a `popf` rule into an [`Instruction`].
fn parse_instruction_popf() -> Instruction {
    Instruction::PopF
}
/// Parse a `ldsp` rule into an [`Instruction`].
fn parse_instruction_ldsp(ldsp: Pair<Rule>) -> Instruction {
    let (_, src) = inner_tuple! { ldsp;
        sep_ip     => ignore;
        source  => parse_source;
    };
    Instruction::Ldsp(src)
}
/// Parse a `ldfr` rule into an [`Instruction`].
fn parse_instruction_ldfr(ldfr: Pair<Rule>) -> Instruction {
    let (_, src) = inner_tuple! { ldfr;
        sep_ip     => ignore;
        source  => parse_source;
    };
    Instruction::Ldfr(src)
}
/// Parse a `jmp` rule into an [`Instruction`].
fn parse_instruction_jmp(jmp: Pair<Rule>) -> Instruction {
    let (_, label) = inner_tuple! { jmp;
        sep_ip       => ignore;
        raw_label => parse_raw_label;
    };
    Instruction::Jmp(label)
}
/// Parse a `jcs` rule into an [`Instruction`].
fn parse_instruction_jcs(jcs: Pair<Rule>) -> Instruction {
    let (_, label) = inner_tuple! { jcs;
        sep_ip     => ignore;
        raw_label   => parse_raw_label;
    };
    Instruction::Jcs(label)
}
/// Parse a `jcc` rule into an [`Instruction`].
fn parse_instruction_jcc(jcc: Pair<Rule>) -> Instruction {
    let (_, label) = inner_tuple! { jcc;
        sep_ip     => ignore;
        raw_label   => parse_raw_label;
    };
    Instruction::Jcc(label)
}
/// Parse a `jzs` rule into an [`Instruction`].
fn parse_instruction_jzs(jzs: Pair<Rule>) -> Instruction {
    let (_, label) = inner_tuple! { jzs;
        sep_ip     => ignore;
        raw_label   => parse_raw_label;
    };
    Instruction::Jzs(label)
}
/// Parse a `jzc` rule into an [`Instruction`].
fn parse_instruction_jzc(jzc: Pair<Rule>) -> Instruction {
    let (_, label) = inner_tuple! { jzc;
        sep_ip     => ignore;
        raw_label   => parse_raw_label;
    };
    Instruction::Jzc(label)
}
/// Parse a `jns` rule into an [`Instruction`].
fn parse_instruction_jns(jns: Pair<Rule>) -> Instruction {
    let (_, label) = inner_tuple! { jns;
        sep_ip     => ignore;
        raw_label   => parse_raw_label;
    };
    Instruction::Jns(label)
}
/// Parse a `jnc` rule into an [`Instruction`].
fn parse_instruction_jnc(jnc: Pair<Rule>) -> Instruction {
    let (_, label) = inner_tuple! { jnc;
        sep_ip     => ignore;
        raw_label   => parse_raw_label;
    };
    Instruction::Jnc(label)
}
/// Parse a `jr` rule into an [`Instruction`].
fn parse_instruction_jr(jr: Pair<Rule>) -> Instruction {
    let (_, label) = inner_tuple! { jr;
        sep_ip     => ignore;
        raw_label   => parse_raw_label;
    };
    Instruction::Jr(label)
}
/// Parse a `call` rule into an [`Instruction`].
fn parse_instruction_call(call: Pair<Rule>) -> Instruction {
    let (_, label) = inner_tuple! { call;
        sep_ip         => ignore;
        raw_label   => parse_raw_label;
    };
    Instruction::Call(label)
}
/// Parse a `ret` rule into an [`Instruction`].
fn parse_instruction_ret() -> Instruction {
    Instruction::Ret
}
/// Parse a `reti` rule into an [`Instruction`].
fn parse_instruction_reti() -> Instruction {
    Instruction::RetI
}
/// Parse a `stop` rule into an [`Instruction`].
fn parse_instruction_stop() -> Instruction {
    Instruction::Stop
}
/// Parse a `nop` rule into an [`Instruction`].
fn parse_instruction_nop() -> Instruction {
    Instruction::Nop
}
/// Parse an `ei` rule into an [`Instruction`].
fn parse_instruction_ei() -> Instruction {
    Instruction::Ei
}
/// Parse a `di` rule into an [`Instruction`].
fn parse_instruction_di() -> Instruction {
    Instruction::Di
}
/// Parse a `comment` rule into a [`Comment`].
fn parse_comment(comment: Pair<Rule>) -> Comment {
    let (_, comment) = inner_tuple! { comment;
        semicolon   => ignore;
        rest        => id;
    };
    comment.as_str().trim_matches(|c| " \t;".contains(c)).into()
}
