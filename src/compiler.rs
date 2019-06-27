use mr2a_asm_parser::asm::{Asm, Constant, Instruction, Label, Line, MemAddress, Register};
use colored::{Colorize};

use std::fmt;
use std::collections::HashMap;

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
                    Line::Label(ref label, _) => {
                        labels.insert(label.clone(), addr);
                        vec![]
                    }
                    Line::Instruction(ref inst, _) => {
                        let inst = inst.clone();
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
            Clr(reg) => {
                let reg: u8 = reg_to_u8(reg);
                vec![Byte(0b0000_01_00 + reg)]
            }
            Inc(reg) => {
                let reg: u8 = reg_to_u8(reg);
                vec![Byte(0b0100_01_00 + reg)]
            }
            St(mem, reg) => {
                let reg: u8 = reg_to_u8(reg);
                let mem: ByteOrLabel = mem.into();
                vec![Byte(0b1111_00_00 + reg), Byte(0b0001_11_11), mem]
            }
            Jmp(label) => vec![Byte(0b1111_10_11), Label(label), Byte(0b0001_00_11)],
            _ => unimplemented!(),
        }
    }
}

fn reg_to_u8(reg: Register) -> u8 {
    match reg {
        Register::R0 => 0,
        Register::R1 => 1,
        Register::R2 => 2,
        Register::R3 => 3,
    }
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
        let max_width = self.lines.iter().map(|(_,bs)| bs.len()).max().unwrap_or(0);
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
