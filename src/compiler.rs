use colored::Colorize;
use mr2a_asm_parser::asm::{
    Asm, Constant, Destination, Instruction, Label, Line, MemAddress, Register, RegisterDDI,
    RegisterDI, Source,
};

use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ByteOrLabel {
    Byte(u8),
    Label(Label),
}

#[derive(Debug, Clone)]
pub struct ByteCode {
    lines: Vec<(Line, Vec<u8>)>,
}

impl ByteCode {
    /// Compile an [`Asm`] program into an [`Executable`].
    // TODO: .ORG will not work
    pub fn compile(asm: Asm) -> Self {
        let mut addr: u8 = 0;
        let mut labels = HashMap::new();
        let mut lines_with_bols: Vec<_> = asm
            .into_iter()
            .map(|line| {
                let bols = match line {
                    Line::Empty(_) => vec![],
                    // Collect label for later usage
                    Line::Label(ref label, _) => {
                        labels.insert(label.clone(), addr);
                        vec![]
                    }
                    Line::Instruction(ref inst, _) => {
                        let inst = inst.clone();
                        if let Instruction::AsmEquals(label, _) = &inst {
                            // Collect label for later usage
                            labels.insert(label.clone(), addr);
                        }
                        ByteCode::compile_instruction(inst)
                    }
                };
                // Increment address counter
                addr += bols.len() as u8;
                (line, bols)
            })
            .collect();
        let lines = lines_with_bols
            .drain(..)
            .map(|(line, mut bols)| {
                let bytes = bols
                    .drain(..)
                    .map(|bol| match bol {
                        ByteOrLabel::Byte(byte) => byte,
                        ByteOrLabel::Label(label) => *labels
                            .get(&label)
                            .expect("infallible. Labels must be defined"),
                    })
                    .collect();
                (line, bytes)
            })
            .collect();

        ByteCode { lines }
    }
    /// Get an iterator over the byte code.
    /// This iterator always starts at address zero.
    pub fn bytes<'a>(&'a self) -> impl Iterator<Item = &u8> + 'a {
        self.lines.iter().map(|(_, c)| c).flatten()
    }
    /// Compile a single line into its bytes.
    fn compile_instruction(inst: Instruction) -> Vec<ByteOrLabel> {
        use ByteOrLabel::*;
        use Instruction::*;
        match inst {
            AsmOrigin(_orig) => unimplemented!(".ORG is not yet implemented"),
            AsmByte(c) => vec![c.into()],
            AsmDefineBytes(mut cs) => cs.drain(..).map(ByteOrLabel::from).collect(),
            AsmDefineWords(_ws) => unimplemented!(".DW is not yet implemented"),
            // TODO: Research how mcontrol does .EQU translation
            AsmEquals(_, c) => vec![c.into()],
            AsmStacksize(_ss) => unimplemented!(".STACKSIZE is not yet implemented"),
            Clr(reg) => {
                let reg: u8 = reg_to_u8(reg);
                vec![Byte(0b0000_01_00 + reg)]
            }
            Add(rd, rs) => from_base_and_two_regs(0b0110_00_00, rd, rs),
            Adc(rd, rs) => from_base_and_two_regs(0b0111_00_00, rd, rs),
            Sub(rd, rs) => from_base_and_two_regs(0b1000_00_00, rd, rs),
            Mul(rd, rs) => from_base_and_two_regs(0b1011_00_00, rd, rs),
            Div(rd, rs) => from_base_and_two_regs(0b1100_00_00, rd, rs),
            Inc(reg) => from_base_and_reg(0b0100_01_00, reg),
            Dec(src) => match src {
                Source::Register(reg) => from_base_and_reg(0b0101_00_00, reg),
                _ => unimplemented!("DEC [something other than R*] does not work yet"),
            },
            Neg(reg) => from_base_and_reg(0b0011_01_00, reg),
            And(rd, rs) => from_base_and_two_regs(0b1001_00_00, rd, rs),
            Or(rd, rs) => from_base_and_two_regs(0b1010_00_00, rd, rs),
            Xor(rd, rs) => from_base_and_two_regs(0b1101_00_00, rd, rs),
            Com(reg) => from_base_and_reg(0b0011_00_00, reg),
            Bits(_dst, _src) => unimplemented!("BITS is not yet implemented!"),
            Mov(dst, src) => ByteCode::compile_instruction_mov(dst, src),
            St(mem, reg) => ByteCode::compile_instruction_mov(mem.into(), reg.into()),
            LdConstant(reg, c) => ByteCode::compile_instruction_mov(reg.into(), c.into()),
            Jmp(label) => vec![Byte(0b1111_10_11), Label(label), Byte(0b0001_00_11)],
            _ => unimplemented!(),
        }
    }
    /// Compile a `MOV` instruction.
    fn compile_instruction_mov(dst: Destination, src: Source) -> Vec<ByteOrLabel> {
        use ByteOrLabel::*;
        // SOURCE
        // Calculate first byte from register and mode
        let source_addr_mode = source_addr_mode(&src) << 2;
        let source_register = match src {
            Source::Register(reg)
            | Source::RegisterDI(RegisterDI(reg))
            | Source::RegisterDDI(RegisterDDI(reg)) => reg_to_u8(reg),
            Source::Constant(_) => 0b11,
            Source::MemAddress(ref mem) => match mem {
                MemAddress::Register(reg) => reg_to_u8(*reg),
                MemAddress::Label(_) | MemAddress::Constant(_) => 0b11,
            },
        };
        let first = 0b1111_00_00 + source_addr_mode + source_register;
        let mut ret = vec![Byte(first)];
        // Add another byte if we need a constant or an address
        let second = match src {
            Source::Constant(c) => Some(c.into()),
            Source::MemAddress(ref mem) => match mem {
                MemAddress::Label(l) => Some(Label(l.clone())),
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
        let destination_register = match dst {
            Destination::Register(reg)
            | Destination::RegisterDI(RegisterDI(reg))
            | Destination::RegisterDDI(RegisterDDI(reg)) => reg_to_u8(reg),
            Destination::Constant(_) => 0b11,
            Destination::MemAddress(ref mem) => match mem {
                MemAddress::Register(reg) => reg_to_u8(*reg),
                MemAddress::Label(_) | MemAddress::Constant(_) => 0b11,
            },
        };
        let third = 0b0001_00_00 + destination_addr_mode + destination_register;
        ret.push(Byte(third));
        // Add another byte if we need an address
        let fourth = match dst {
            Destination::Constant(c) => Some(c.into()),
            Destination::MemAddress(ref mem) => match mem {
                MemAddress::Label(l) => Some(Label(l.clone())),
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

/// Get the address mode from a [`Source`]
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
            Source::Constant(_) | Source::RegisterDI(_) => 0b10,
            Source::RegisterDDI(_) => 0b11,
            Source::MemAddress(ref mem) => match mem {
                MemAddress::Register(_) => 0b01,
                MemAddress::Label(_) | MemAddress::Constant(_) => 0b11,
            },
        }
}

/// Get the address mode from a [`Destination`]. See [`source_addr_mode`] for more.
fn destination_addr_mode(dst: &Destination) -> u8 {
         match dst {
            Destination::Register(_) => 0b00,
            Destination::Constant(_) | Destination::RegisterDI(_) => 0b10,
            Destination::RegisterDDI(_) => 0b11,
            Destination::MemAddress(ref mem) => match mem {
                MemAddress::Register(_) => 0b01,
                MemAddress::Label(_) | MemAddress::Constant(_) => 0b11,
            },
        }
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
            MemAddress::Label(label) => ByteOrLabel::Label(label),
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
                writeln!(f, "")?;
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
