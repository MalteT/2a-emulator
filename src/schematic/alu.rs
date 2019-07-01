/// The arithmetic logical unit!
/// Stateless.
#[derive(Debug, Clone)]
pub struct Alu;

/// A list containing all ALU functions.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AluFn {
    ADDH,
    A,
    NOR,
    ZERO,
    ADD,
    ADDS,
    ADC,
    ADCS,
    LSR,
    RR,
    RRC,
    ASR,
    B,
    SETC,
    BH,
    INVC,
}

impl Alu {
    /// Get the output of the ALU.
    pub fn output(cin: bool, a: u8, b: u8, function: AluFn) -> u8 {
        Alu::get_outputs(cin, a, b, function).0
    }
    /// Get the carry out of the ALU.
    pub fn co(cin: bool, a: u8, b: u8, function: AluFn) -> bool {
        Alu::get_outputs(cin, a, b, function).1
    }
    /// Get the zero out of the ALU.
    pub fn zo(cin: bool, a: u8, b: u8, function: AluFn) -> bool {
        Alu::get_outputs(cin, a, b, function).2
    }
    /// Get the negative out of the ALU.
    pub fn no(cin: bool, a: u8, b: u8, function: AluFn) -> bool {
        Alu::get_outputs(cin, a, b, function).3
    }
    /// Calculate all outputs of the ALU from the given inputs.
    ///
    /// # Returns
    /// A 4-tuple with
    ///
    /// - output,
    /// - carry out,
    /// - zero out,
    /// - negative out.
    fn get_outputs(cin: bool, a: u8, b: u8, function: AluFn) -> (u8, bool, bool, bool) {
        use AluFn::*;
        let (out, cout) = match function {
            ADDH => a.overflowing_add(b),
            A => (a, false),
            NOR => (!(a | b), false),
            ZERO => (0, false),
            ADD => a.overflowing_add(b),
            ADDS => {
                let (o, c1) = a.overflowing_add(b);
                let (o, c2) = o.overflowing_add(1);
                (o, !(c1 || c2))
            }
            ADC => {
                let (o, c1) = a.overflowing_add(b);
                let (o, c2) = o.overflowing_add(cin.into());
                (o, c1 || c2)
            }
            ADCS => {
                let (o, c1) = a.overflowing_add(b);
                let (o, c2) = o.overflowing_add((!cin).into());
                (o, !(c1 || c2))
            }
            LSR => {
                let cout = (a & 0b0000_0001) != 0;
                (a >> 1, cout)
            }
            RR => {
                let cout = (a & 0b0000_0001) != 0;
                let (o, c) = a.overflowing_shr(1);
                (o | (c as u8) << 7, cout)
            }
            RRC => {
                let cout = (a & 0b0000_0001) != 0;
                let (o, _) = a.overflowing_shr(1);
                (o | (cin as u8) << 7, cout)
            }
            ASR => {
                let cout = (a & 0b0000_0001) != 0;
                let (o, _) = a.overflowing_shr(1);
                (o | a & 0b10000000, cout)
            }
            B => (b, false),
            SETC => (b, true),
            BH => (b, cin),
            INVC => (b, !cin),
        };

        (out, cout, out == 0, (out & 0b1000_0000) != 0)
    }
}

impl From<(bool, bool, bool, bool)> for AluFn {
    fn from((malus3, malus2, malus1, malus0): (bool, bool, bool, bool)) -> Self {
        use AluFn::*;
        let selection =
            ((malus3 as u8) << 3) + ((malus2 as u8) << 2) + ((malus1 as u8) << 1) + (malus0 as u8);
        match selection {
            0b0000 => ADDH,
            0b0001 => A,
            0b0010 => NOR,
            0b0011 => ZERO,
            0b0100 => ADD,
            0b0101 => ADDS,
            0b0110 => ADC,
            0b0111 => ADCS,
            0b1000 => LSR,
            0b1001 => RR,
            0b1010 => RRC,
            0b1011 => ASR,
            0b1100 => B,
            0b1101 => SETC,
            0b1110 => BH,
            0b1111 => INVC,
            _ => unreachable!(),
        }
    }
}
