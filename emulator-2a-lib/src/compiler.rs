//! Everything related to translating the AST to bytecode.
//!
//! Simply use the [`Translator`] to compile your [`Assembly`](Asm) into valid [`ByteCode`].
//!
//! # Example
//!
//! ```
//! # use emulator_2a_lib::{parser::AsmParser, compiler::{Translator}};
//! let asm = r#"
//!     #! mrasm
//!
//!     .DB 42
//!     CLR R0
//! "#.trim();
//!
//! let parsed = AsmParser::parse(asm).expect("Parsing went well");
//! let bytecode = Translator::compile(&parsed);
//! let bytes: Vec<u8> = bytecode.bytes().cloned().collect();
//!
//! assert_eq!(bytes, vec![42, 4]);
//!
//! ```
use colored::Colorize;
use log::error;

use std::{collections::HashMap, fmt, ops::Deref, rc::Rc};

use crate::parser::{
    Asm, Comment, Constant, Destination, Instruction, Label, Line, MemAddress, Programsize,
    Register, RegisterDdi, RegisterDi, Source, Stacksize,
};

/// An either type for [`u8`]/[`Label`].
///
/// This is used for label references.
/// During translation all labels will be translated into
/// this type which is, after all Labels are defined, translated into the correct bytes.
#[derive(Clone)]
pub enum ByteOrLabel {
    /// An ordinary byte.
    Byte(u8),
    /// A label that will be replaced by the address of the following byte.
    Label(Label),
    /// A label that will be replaced by the address of the following byte
    /// which will the be transformed by the function.
    LabelFn(Label, Rc<dyn Fn(u8) -> u8>),
}

/// This is the final byte code with additional information from which [`Line`]
/// the byte code originates.
#[derive(Debug, Clone)]
pub struct ByteCode {
    /// Lines with translated byte code.
    pub lines: Vec<(Line, Vec<u8>)>,
    /// Stacksize for limiting.
    pub stacksize: Stacksize,
    /// Programsize for limiting.
    pub programsize: Programsize,
}

/// Translator for [`Asm`] -> [`ByteCode`]
#[derive(Debug, Clone)]
pub struct Translator {
    next_addr: u8,
    known_labels: HashMap<Label, u8>,
    bytes: Vec<(Line, Vec<ByteOrLabel>)>,
    stacksize: Stacksize,
    programsize: Programsize,
}

impl ByteCode {
    /// Get an iterator over the byte code.
    /// This iterator always starts at address zero.
    ///
    /// Thus the resulting bytes can be easily written into the program memory.
    pub fn bytes<'a>(&'a self) -> impl Iterator<Item = &u8> + 'a {
        self.lines.iter().map(|(_, c)| c).flatten()
    }
}

impl Translator {
    /// Compile the given [`Asm`] into [`ByteCode`].
    pub fn compile(asm: &Asm) -> ByteCode {
        let mut tr = Translator::new();
        for line in &asm.lines {
            tr.push(line);
        }
        tr.finish()
    }
    /// Create a new translator, starting at address `0` without any Labels and
    /// no bytes in memory.
    fn new() -> Self {
        Translator {
            bytes: vec![],
            known_labels: HashMap::new(),
            next_addr: 0,
            stacksize: Stacksize::default(),
            programsize: Programsize::default(),
        }
    }
    /// Push a [`Line`] into the translator, adding the translated bytes,
    /// changing address and pushing labels
    fn push(&mut self, line: &Line) {
        match line {
            Line::Empty(_) => {
                self.bytes.push((line.clone(), vec![]));
            }
            Line::Label(label, _) => {
                self.known_labels.insert(label.to_string(), self.next_addr);
                self.bytes.push((line.clone(), vec![]));
            }
            Line::Instruction(inst, comment) => self.push_instruction(inst, comment),
        }
    }
    /// Push an instruction into the translator.
    fn push_instruction(&mut self, inst: &Instruction, comment: &Option<Comment>) {
        use ByteOrLabel::*;
        use Instruction::*;
        let bols = match inst.clone() {
            AsmOrigin(addr) => {
                // XXX: This can only skip bytes atm, no fancy
                // XXX: messing with your programs yet!
                // XXX: Prevent the usage of negativ skips to prevent diverging
                // XXX: from the real machine.
                if addr < self.next_addr {
                    error! {
                        "Compiler detected a problematic .ORG instruction!\nThe instruction: `{}` will cause the real machine to behave differently, since the .ORG instruction would point to an existing byte in the program. This is probably unintentional, please use an address larger than {}, to not overwrite your own program.\n\n -> Example: .ORG {}\n\n", inst, self.next_addr, self.next_addr + 5
                    }
                    panic!("Compilation aborted")
                }
                // Insert blanks
                let mut skips = vec![];
                if addr > self.next_addr {
                    for _skip in 0..(addr - self.next_addr) {
                        skips.push(Byte(0x00));
                    }
                }
                skips
            }
            AsmByte(nr) => {
                self.next_addr += nr;
                let mut ret = vec![];
                for _ in 0..nr {
                    ret.push(Byte(0b0000_0000));
                }
                ret
            }
            AsmDefineBytes(mut cs) => cs.drain(..).map(ByteOrLabel::Byte).collect(),
            AsmDefineWords(mut cs) => cs
                .drain(..)
                .map(|word| {
                    vec![
                        ByteOrLabel::Byte((word >> 8) as u8),
                        ByteOrLabel::Byte(word as u8),
                    ]
                })
                .flatten()
                .collect(),
            AsmEquals(label, constant) => {
                // Push Label!
                self.known_labels.insert(label, constant);
                vec![]
            }
            AsmStacksize(ss) => {
                self.stacksize = ss;
                vec![]
            }
            AsmProgramsize(ps) => {
                self.programsize = ps;
                vec![]
            }
            Clr(reg) => {
                let reg: u8 = reg_to_u8(reg);
                vec![Byte(0b0000_0100 + reg)]
            }
            Add(rd, rs) => from_base_and_two_regs(0b0110_0000, rd, rs),
            Adc(rd, rs) => from_base_and_two_regs(0b0111_0000, rd, rs),
            Sub(rd, rs) => from_base_and_two_regs(0b1000_0000, rd, rs),
            Mul(rd, rs) => from_base_and_two_regs(0b1011_0000, rd, rs),
            Div(rd, rs) => from_base_and_two_regs(0b1100_0000, rd, rs),
            Inc(reg) => from_base_and_reg(0b0100_0100, reg),
            Dec(src) => match src {
                Source::Register(reg) => from_base_and_reg(0b0101_0000, reg),
                _ => unimplemented!("DEC [something other than R*] does not work yet"),
            },
            Neg(reg) => from_base_and_reg(0b0011_0100, reg),
            And(rd, rs) => from_base_and_two_regs(0b1001_0000, rd, rs),
            Or(rd, rs) => from_base_and_two_regs(0b1010_0000, rd, rs),
            Xor(rd, rs) => from_base_and_two_regs(0b1101_0000, rd, rs),
            Com(reg) => from_base_and_reg(0b0011_0000, reg),
            Bits(dst, src) => from_bases_dst_and_src(0b1111_0000, 0b0101_0000, &dst, &src),
            Bitc(dst, src) => from_bases_dst_and_src(0b1111_0000, 0b0110_0000, &dst, &src),
            Tst(reg) => from_base_and_reg(0b0100_1000, reg),
            Cmp(dst, src) => from_bases_dst_and_src(0b1111_0000, 0b0010_0000, &dst, &src),
            Bitt(dst, src) => from_bases_dst_and_src(0b1111_0000, 0b0011_0000, &dst, &src),
            Lsr(reg) => from_base_and_reg(0b0011_1000, reg),
            Asr(reg) => from_base_and_reg(0b0011_1100, reg),
            Lsl(reg) => {
                let reg = reg_to_u8(reg);
                vec![Byte(0b0110_0000 + (reg << 2) + reg)]
            }
            Rrc(reg) => from_base_and_reg(0b0100_0000, reg),
            Rlc(reg) => {
                let reg = reg_to_u8(reg);
                vec![Byte(0b0111_0000 + (reg << 2) + reg)]
            }
            Mov(dst, src) => compile_instruction_mov(dst, src),
            LdConstant(reg, c) => compile_instruction_mov(reg.into(), c.into()),
            LdMemAddress(reg, mem) => compile_instruction_mov(reg.into(), mem.into()),
            St(mem, reg) => compile_instruction_mov(mem.into(), reg.into()),
            Push(reg) => from_base_and_reg(0b0001_0000, reg),
            Pop(reg) => from_base_and_reg(0b0001_0100, reg),
            PushF => vec![Byte(0b0001_1000)],
            PopF => vec![Byte(0b0001_1100)],
            Ldsp(src) => from_bases_and_src(0b1111_0000, 0b0100_0000, &src),
            Ldfr(src) => from_bases_and_src(0b1111_0000, 0b0100_0100, &src),
            Jmp(label) => vec![Byte(0b1111_1011), Label(label), Byte(0b0001_0011)],
            Jcs(label) => relative_jump(0b001, label, self.next_addr),
            Jcc(label) => relative_jump(0b101, label, self.next_addr),
            Jzs(label) => relative_jump(0b010, label, self.next_addr),
            Jzc(label) => relative_jump(0b110, label, self.next_addr),
            Jns(label) => relative_jump(0b011, label, self.next_addr),
            Jnc(label) => relative_jump(0b111, label, self.next_addr),
            Jr(label) => relative_jump(0b000, label, self.next_addr),
            Call(label) => vec![Byte(0b0010_1000), Label(label)],
            Ret => vec![Byte(0b0001_0111)],
            RetI => vec![Byte(0b0010_1100)],
            Stop => vec![Byte(0b0000_0001)],
            Nop => vec![Byte(0b0000_0010)],
            Ei => vec![Byte(0b0000_1000)],
            Di => vec![Byte(0b0000_1100)],
        };
        let line = Line::Instruction(inst.clone(), comment.clone());
        self.next_addr += bols.len() as u8;
        self.bytes.push((line, bols));
    }
    /// Finish the translation.
    /// This replaces all references to labels with the address the
    /// Label was defined at.
    fn finish(mut self) -> ByteCode {
        let labels = self.known_labels;
        let lines = self
            .bytes
            .drain(..)
            .map(|(line, mut bols)| {
                let bytes = bols
                    .drain(..)
                    .map(|bol| match bol {
                        ByteOrLabel::Byte(byte) => vec![byte],
                        ByteOrLabel::Label(label) => vec![*labels
                            .get(&label)
                            .expect("infallible. Labels must be defined")],
                        ByteOrLabel::LabelFn(label, f) => {
                            let b = *labels
                                .get(&label)
                                .expect("infallible. Labels must be defined");
                            vec![f.deref()(b)]
                        }
                    })
                    .flatten()
                    .collect();
                (line, bytes)
            })
            .collect();
        let stacksize = self.stacksize;
        let programsize = self.programsize;
        ByteCode {
            lines,
            stacksize,
            programsize,
        }
    }
}

/// Create the necessary [`ByteOrLabel`]s for a relative jump with the given condition.
fn relative_jump(cond: u8, label: Label, curr_addr: u8) -> Vec<ByteOrLabel> {
    use ByteOrLabel::*;
    debug_assert!(cond <= 0b0000_0111);
    let first = Byte(0b0010_0000 + cond);
    // Calculate relative offset of the target address.
    let second = LabelFn(
        label,
        Rc::from(move |target: u8| {
            let (pre_jump, _) = curr_addr.overflowing_add(2);
            let (diff, _) = target.overflowing_sub(pre_jump);
            diff
        }),
    );
    vec![first, second]
}

/// Compile a `MOV` instruction.
fn compile_instruction_mov(dst: Destination, src: Source) -> Vec<ByteOrLabel> {
    use ByteOrLabel::*;
    // SOURCE
    // Calculate first byte from register and mode
    let source_addr_mode = source_addr_mode(&src) << 2;
    let source_register = source_register(&src);
    let first = 0b1111_0000 + source_addr_mode + source_register;
    let mut ret = vec![Byte(first)];
    // Add another byte if we need a constant or an address
    let second = match src {
        Source::Constant(c) => Some(c.into()),
        Source::MemAddress(ref mem) => match mem {
            MemAddress::Constant(c) => Some(c.clone().into()),
            MemAddress::Register(_) => None,
        },
        _ => None,
    };
    if let Some(second) = second {
        ret.push(second)
    }
    // DESTINATION
    // Calculate first byte from register and mode
    let destination_addr_mode = destination_addr_mode(&dst) << 2;
    let destination_register = destination_register(&dst);
    let third = 0b0001_0000 + destination_addr_mode + destination_register;
    ret.push(Byte(third));
    // Add another byte if we need an address
    let fourth = match dst {
        Destination::MemAddress(ref mem) => match mem {
            MemAddress::Constant(c) => Some(c.clone().into()),
            MemAddress::Register(_) => None,
        },
        _ => None,
    };
    if let Some(fourth) = fourth {
        ret.push(fourth)
    }
    ret
}

/// Convert a [`Register`] to [`u8`]
fn reg_to_u8(reg: Register) -> u8 {
    match reg {
        Register::R0 => 0,
        Register::R1 => 1,
        Register::R2 => 2,
        Register::R3 => 3,
    }
}

/// Get the address mode from a [`Source`].
///
/// | Mode  | Meaning |
/// |-------|---------|
/// | `0 0` | R       |
/// | `0 1` | (R)     |
/// | `1 0` | (R+)    |
/// | `1 1` | ((R+))  |
fn source_addr_mode(src: &Source) -> u8 {
    match src {
        Source::Register(_) => 0b00,
        Source::Constant(_) | Source::RegisterDi(_) => 0b10,
        Source::RegisterDdi(_) => 0b11,
        Source::MemAddress(ref mem) => match mem {
            MemAddress::Register(_) => 0b01,
            MemAddress::Constant(_) => 0b11,
        },
    }
}

/// Get the source register from a [`Source`].
fn source_register(src: &Source) -> u8 {
    match src {
        Source::Register(reg)
        | Source::RegisterDi(RegisterDi(reg))
        | Source::RegisterDdi(RegisterDdi(reg)) => reg_to_u8(*reg),
        Source::Constant(_) => 0b11,
        Source::MemAddress(ref mem) => match mem {
            MemAddress::Register(reg) => reg_to_u8(*reg),
            MemAddress::Constant(_) => 0b11,
        },
    }
}

/// Get the address mode from a [`Destination`]. See [`source_addr_mode`] for more.
fn destination_addr_mode(dst: &Destination) -> u8 {
    match dst {
        Destination::Register(_) => 0b00,
        Destination::RegisterDi(_) => 0b10,
        Destination::RegisterDdi(_) => 0b11,
        Destination::MemAddress(ref mem) => match mem {
            MemAddress::Register(_) => 0b01,
            MemAddress::Constant(_) => 0b11,
        },
    }
}

/// Get the destination register from a [`Destination`].
fn destination_register(dst: &Destination) -> u8 {
    match dst {
        Destination::Register(reg)
        | Destination::RegisterDi(RegisterDi(reg))
        | Destination::RegisterDdi(RegisterDdi(reg)) => reg_to_u8(*reg),
        Destination::MemAddress(ref mem) => match mem {
            MemAddress::Register(reg) => reg_to_u8(*reg),
            MemAddress::Constant(_) => 0b11,
        },
    }
}

/// Create a byte vector from two base bytes, a source and a destination.
/// ```text
/// 0b1101_10_11 [0b10110101] 0b0110_11_00 [0b10111001]
///   -B1- MS RS  addr/const    -B2- MD RD  ---addr---
/// ```
fn from_bases_dst_and_src(b1: u8, b2: u8, dst: &Destination, src: &Source) -> Vec<ByteOrLabel> {
    use ByteOrLabel::*;
    // Calculate first byte from register and mode
    let source_addr_mode = source_addr_mode(&src) << 2;
    let source_register = source_register(&src);
    let first = b1 + source_addr_mode + source_register;
    let mut ret = vec![Byte(first)];
    // Add another byte if we need a constant or an address
    let second = match src {
        Source::Constant(c) => Some(c.clone().into()),
        Source::MemAddress(ref mem) => match mem {
            MemAddress::Constant(c) => Some(c.clone().into()),
            MemAddress::Register(_) => None,
        },
        _ => None,
    };
    if let Some(second) = second {
        ret.push(second)
    }
    // DESTINATION
    // Calculate first byte from register and mode
    let destination_addr_mode = destination_addr_mode(&dst) << 2;
    let destination_register = destination_register(&dst);
    let third = b2 + destination_addr_mode + destination_register;
    ret.push(Byte(third));
    // Add another byte if we need an address
    let fourth = match dst {
        Destination::MemAddress(ref mem) => match mem {
            MemAddress::Constant(c) => Some(c.clone().into()),
            MemAddress::Register(_) => None,
        },
        _ => None,
    };
    if let Some(fourth) = fourth {
        ret.push(fourth)
    }
    ret
}

/// Create a Vector of bytes or labels from two bases and a source.
/// ```text
/// 0b1101_10_11 [0b10110101] 0b0110_11_00
///   -B1- MS RS  addr/const    --BASE 2--
/// ```
fn from_bases_and_src(b1: u8, b2: u8, src: &Source) -> Vec<ByteOrLabel> {
    use ByteOrLabel::*;
    // Calculate first byte from register and mode
    let source_addr_mode = source_addr_mode(&src) << 2;
    let source_register = source_register(&src);
    let first = b1 + source_addr_mode + source_register;
    let mut ret = vec![Byte(first)];
    // Add another byte if we need a constant or an address
    let second = match src {
        Source::Constant(c) => Some(c.clone().into()),
        Source::MemAddress(ref mem) => match mem {
            MemAddress::Constant(c) => Some(c.clone().into()),
            MemAddress::Register(_) => None,
        },
        _ => None,
    };
    if let Some(second) = second {
        ret.push(second)
    }
    ret.push(Byte(b2));
    ret
}

/// Create a byte vector from a base and a destination register.
/// ```text
/// 0b0000_01_11
///   -BASE-- RD
/// ```
fn from_base_and_reg(base: u8, reg: Register) -> Vec<ByteOrLabel> {
    let reg = reg_to_u8(reg);
    vec![ByteOrLabel::Byte(base + reg)]
}

/// Create a byte vector from a base, a destination register and a source register.
/// ```text
/// 0b0000_01_11
///   BASE RS RD
/// ```
fn from_base_and_two_regs(base: u8, dst: Register, src: Register) -> Vec<ByteOrLabel> {
    let dst = reg_to_u8(dst);
    let src = reg_to_u8(src) << 2;
    vec![ByteOrLabel::Byte(base + src + dst)]
}

impl From<Constant> for ByteOrLabel {
    fn from(c: Constant) -> Self {
        match c {
            Constant::Constant(c) => ByteOrLabel::Byte(c),
            Constant::Label(label) => ByteOrLabel::Label(label),
        }
    }
}

impl From<MemAddress> for ByteOrLabel {
    fn from(mem: MemAddress) -> Self {
        match mem {
            MemAddress::Constant(c) => c.into(),
            MemAddress::Register(_reg) => unimplemented!("How to make a const from a register"),
        }
    }
}

impl fmt::Display for ByteCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let max_width = self.lines.iter().map(|(_, bs)| bs.len()).max().unwrap_or(0);
        for (line, code) in &self.lines {
            // Skip semicolon for empty lines
            if let Line::Empty(None) = line {
                writeln!(f)?;
                continue;
            }
            let line = format!("; {}", line);
            let code: Vec<_> = code.iter().map(|c| format!("{:>02X}", c)).collect();
            let code_str = format!("{:>width$}", code.join(" "), width = max_width * 3);
            writeln!(f, " {} {}", code_str, line.dimmed())?;
        }
        Ok(())
    }
}

impl fmt::Debug for ByteOrLabel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ByteOrLabel::Byte(b) => write!(f, "Byte({:>02X})", b),
            ByteOrLabel::Label(l) => write!(f, "Label({})", l),
            ByteOrLabel::LabelFn(l, _) => write!(f, "LabelFn({}, [hidden])", l),
        }
    }
}
