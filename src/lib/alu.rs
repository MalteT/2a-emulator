/// The arithmetic logical unit! Stateless.
use enum_primitive::{FromPrimitive, enum_from_primitive, enum_from_primitive_impl, enum_from_primitive_impl_ty};

enum_from_primitive! {
    /// A list containing all functions understood by the alu.
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum AluSelect {
        /// Add A and B but keep the carry_in or set it if the sum exceeds 8 bits.
        ADDH  = 0b0000,
        /// Pass A through and set zero out and negative out flags accordingly.
        A     = 0b0001,
        /// Calculate the binary NOR for A and B.
        NOR   = 0b0010,
        /// Output is always zero.
        ZERO  = 0b0011,
        /// Simply add A and B.
        ADD   = 0b0100,
        /// Add A + B + 1 and invert the carry. This is addition for subtraction.
        ADDS  = 0b0101,
        /// Add A + B + carry input.
        ADC   = 0b0110,
        /// Add A + B + inverted carry in. This is ADC for subtraction.
        ADCS  = 0b0111,
        /// Logical shift right. Highest bit will be set to zero.
        LSR   = 0b1000,
        /// Rotate right. Highest bit will be set to the lowest bit of the input.
        RR    = 0b1001,
        /// Rotate right. Highest bit will be set to carry input.
        RRC   = 0b1010,
        /// Arithmetic shift right. The highest bit will remain the same.
        ASR   = 0b1011,
        /// Pass B through and set zero out and negative out flags accordingly.
        B     = 0b1100,
        /// Pass B through but set the carry flag.
        SETC  = 0b1101,
        /// Pass B through but hold the carry input.
        BH    = 0b1110,
        /// Pass B through and invert the carry input.
        INVC  = 0b1111,
    }
}

/// Input values needed for the ALU.
///
/// # Example
///
/// ```
/// # use emulator_2a_lib::AluInput;
/// let input = AluInput::new(103, 20, true);
///
/// assert_eq!(input.input_a(), 103);
/// assert_eq!(input.input_b(), 20);
/// assert_eq!(input.carry_in(), true);
/// ```
#[derive(Debug, Clone)]
pub struct AluInput {
    /// Main input A.
    input_a: u8,
    /// Main input B.
    input_b: u8,
    /// Carry in from the last operation.
    carry_in: bool,
}

impl AluInput {
    /// Create a new AluInput object from:
    ///
    /// - `input_a`, the main input A
    /// - `input_b`, the main input B
    /// - `carry_in`, the carry_out of the last operation
    pub const fn new(input_a: u8, input_b: u8, carry_in: bool) -> Self {
        AluInput {
            input_a,
            input_b,
            carry_in,
        }
    }
    /// Get the main input A.
    pub const fn input_a(&self) -> u8 {
        self.input_a
    }
    /// Get the main input B.
    pub const fn input_b(&self) -> u8 {
        self.input_b
    }
    /// Get the carry_in.
    pub const fn carry_in(&self) -> bool {
        self.carry_in
    }
}

/// Output of the arithmetic logical unit.
///
/// # Example
///
/// ```
/// # use emulator_2a_lib::{AluInput, AluSelect, AluOutput};
/// let input = AluInput::new(40, 2, false);
/// let function = AluSelect::ADD;
///
/// let output = AluOutput::from_input(&input, &function);
/// assert_eq!(output.output(), 42);
/// assert_eq!(output.carry_out(), false);
/// assert_eq!(output.zero_out(), false);
/// assert_eq!(output.negative_out(), false);
/// ```
#[derive(Debug, Clone)]
pub struct AluOutput {
    /// Main output of the last operation.
    output: u8,
    /// Carry out of the operation, usually indicating
    /// some sort of overflow.
    carry_out: bool,
    /// Zero out of the operation, set when the output is zero.
    zero_out: bool,
    /// Negative out of the operation, set when the highest bit of the output is set.
    negative_out: bool,
}

impl AluOutput {
    /// Calculate the output from the input of the alu.
    pub fn from_input(input: &AluInput, function: &AluSelect) -> Self {
        let a = input.input_a;
        let b = input.input_b;
        let carry_in = input.carry_in;
        let (out, carry_out) = match function {
            AluSelect::ADDH => a.overflowing_add(b),
            AluSelect::A => (a, false),
            AluSelect::NOR => (!(a | b), false),
            AluSelect::ZERO => (0, false),
            AluSelect::ADD => a.overflowing_add(b),
            AluSelect::ADDS => {
                let (o, c1) = a.overflowing_add(b);
                let (o, c2) = o.overflowing_add(1);
                (o, !(c1 || c2))
            }
            AluSelect::ADC => {
                let (o, c1) = a.overflowing_add(b);
                let (o, c2) = o.overflowing_add(carry_in.into());
                (o, c1 || c2)
            }
            AluSelect::ADCS => {
                let (o, c1) = a.overflowing_add(b);
                let (o, c2) = o.overflowing_add((!carry_in).into());
                (o, !(c1 || c2))
            }
            AluSelect::LSR => {
                let carry_out = (a & 0b0000_0001) != 0;
                (a >> 1, carry_out)
            }
            AluSelect::RR => {
                let carry_out = (a & 0b0000_0001) != 0;
                let (o, c) = a.overflowing_shr(1);
                (o | (c as u8) << 7, carry_out)
            }
            AluSelect::RRC => {
                let carry_out = (a & 0b0000_0001) != 0;
                let (o, _) = a.overflowing_shr(1);
                (o | (carry_in as u8) << 7, carry_out)
            }
            AluSelect::ASR => {
                let carry_out = (a & 0b0000_0001) != 0;
                let (o, _) = a.overflowing_shr(1);
                (o | a & 0b10000000, carry_out)
            }
            AluSelect::B => (b, false),
            AluSelect::SETC => (b, true),
            AluSelect::BH => (b, carry_in),
            AluSelect::INVC => (b, !carry_in),
        };
        AluOutput {
            output: out,
            carry_out,
            zero_out: out == 0,
            negative_out: out & 0b1000_0000 != 0,
        }
    }
    /// Calculate the output from the default input, i.e. all zeros.
    pub fn from_default_input() -> Self {
        AluOutput::from_input(&AluInput::default(), &AluSelect::default())
    }
    /// Get the main output of the last operation.
    pub const fn output(&self) -> u8 {
        self.output
    }
    /// Get the carry out flag of the last operation. This usually indicates some
    /// sort of overflow.
    pub const fn carry_out(&self) -> bool {
        self.carry_out
    }
    /// Get the zero out flag of the last operation. This is set if the output is zero.
    pub const fn zero_out(&self) -> bool {
        self.zero_out
    }
    /// Get the negative out flag of the last operation. This is set if the highest
    /// bit of the output is set.
    pub const fn negative_out(&self) -> bool {
        self.negative_out
    }
}

impl Default for AluInput {
    fn default() -> Self {
        AluInput {
            input_a: 0,
            input_b: 0,
            carry_in: false,
        }
    }
}

impl Default for AluSelect {
    fn default() -> Self {
        AluSelect::from_u8(0).expect("infallible")
    }
}
