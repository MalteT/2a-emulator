//! # Schematic functionality
//!
//! All necessary functions for building the schematic for the
//! Minirechner 2a schematic.

use bitflags::bitflags;
use log::{debug, trace};

use std::ops::{Index, IndexMut};

use super::mp_ram::MP28BitWord;

/// Create a logical `HIGH` function.
pub fn make_high() -> impl FnMut() -> bool {
    || true
}

/// Create a logical `LOW` function.
pub fn make_low() -> impl FnMut() -> bool {
    || false
}

/// Create a logical binary `and` function.
pub fn make_and2() -> impl FnMut(&bool, &bool) -> bool {
    |&in0, &in1| {
        let out = in0 && in1;
        debug!("AND2: {} && {} -> {}", in0, in1, out);
        out
    }
}

/// Create a logical `and` function with four inputs.
pub fn make_and4() -> impl FnMut(&bool, &bool, &bool, &bool) -> bool {
    |&in0, &in1, &in2, &in3| {
        let out = in0 && in1 && in2 && in3;
        debug!("AND4: {} && {} && {} && {}", in0, in1, in2, in3);
        out
    }
}

/// Create a logical binary `or` function.
pub fn make_or2() -> impl FnMut(&bool, &bool) -> bool {
    |&in0, &in1| {
        let out = in0 || in1;
        debug!("OR2 : {} || {} -> {}", in0, in1, out);
        out
    }
}

/// Create a logical binary `xor` function.
pub fn make_xor2() -> impl FnMut(&bool, &bool) -> bool {
    |&in0, &in1| {
        let out = (in0 || in1) && !(in0 && in1);
        debug!("XOR2: {} || {} -> {}", in0, in1, out);
        out
    }
}

/// Create a 2x1 multiplexer function.
///
/// Inputs: 3
/// - `in0`..`in1`: MUX inputs
/// - `select0`: MUX input selector
pub fn make_mux2x1() -> impl FnMut(&bool, &bool, &bool) -> bool {
    |&in0, &in1, &select0| {
        let out = if select0 { in1 } else { in0 };
        debug!(
            "MUX2x1: 0: {}, 1: {}, select: {}, out: {}",
            in0, in1, select0, out
        );
        out
    }
}

/// Create a 4x1 multiplexer function.
///
/// Inputs: 6
/// - `in0`..`in3`: MUX inputs
/// - `select0`..`select1`: MUX input selectors
pub fn make_mux4x1() -> impl FnMut(&bool, &bool, &bool, &bool, &bool, &bool) -> bool {
    |&in0, &in1, &in2, &in3, &select0, &select1| {
        let out = match (select1, select0) {
            (false, false) => in0,
            (false, true) => in1,
            (true, false) => in2,
            (true, true) => in3,
        };
        debug!(
            "MUX4x1: 0: {}, 1: {}, 2: {}, 3: {}, select0: {}, select1: {}, out: {}",
            in0, in1, in2, in3, select0, select1, out
        );
        out
    }
}

/// Create a 8x1 multiplexer function.
///
/// Inputs: 11
/// - `in0`..`in7`: MUX inputs
/// - `select0`..`select2`: MUX input selectors
pub fn make_mux8x1(
) -> impl FnMut(&bool, &bool, &bool, &bool, &bool, &bool, &bool, &bool, &bool, &bool, &bool) -> bool
{
    |&in0, &in1, &in2, &in3, &in4, &in5, &in6, &in7, &select0, &select1, &select2| {
        let out = match (select2, select1, select0) {
            (false, false, false) => in0,
            (false, false, true) => in1,
            (false, true, false) => in2,
            (false, true, true) => in3,
            (true, false, false) => in4,
            (true, false, true) => in5,
            (true, true, false) => in6,
            (true, true, true) => in7,
        };
        debug!( "MUX8x1: 0: {}, 1: {}, 2: {}, 3: {}, 4: {}, 5: {}, 6: {}, 7: {}, select0: {}, select1: {}, select2: {}, out: {}", in0, in1, in2, in3, in4, in5, in6, in7, select0, select1, select2, out);
        out
    }
}

/// Create a D-FlipFlop function
///
/// Inputs: 2
/// - `input`: Data input
/// - `clk`: Clock
pub fn make_dflipflop() -> impl FnMut(&bool, &bool) -> bool {
    let mut state = false;
    move |&input, &clk| {
        debug!("DFLIPFLOP: input: {}, clk: {}", input, clk);
        if clk {
            state = input
        }
        state
    }
}

/// Create a D-FlipFlop with clear function
///
/// Inputs: 2
/// - `input`: Data input
/// - `clk`: Clock
/// - `clear`: Reset flipflop
pub fn make_dflipflopc() -> impl FnMut(&bool, &bool, &bool) -> bool {
    let mut state = false;
    move |&input, &clk, &clear| {
        debug!(
            "DFLIPFLOP: input: {}, clk: {}, clear: {}",
            input, clk, clear
        );
        if clear {
            state = false
        } else if clk {
            state = input
        }
        state
    }
}

/// Register block with 8x8 bits.
struct Register {
    state: [u8; 8],
}

bitflags! {
    pub struct Flags: u8 {
        const CARRY      = 0b00000001;
        const ZERO       = 0b00000010;
        const NEGATIVE   = 0b00000100;
        const INT_ENABLE = 0b00001000;
    }
}

impl Register {
    /// Create a new register filled with zeros.
    pub fn new() -> Register {
        Register { state: [0; 8] }
    }
    /// Reset registers to initial state
    pub fn reset(&mut self) {
        *self = Register::new();
    }
    /// Get the flags in register 4.
    pub fn flags(&self) -> Flags {
        Flags::from_bits_truncate(self[4])
    }
    /// Set the flags in register 4.
    pub fn set_flags(&mut self, flags: Flags) {
        self[4] = flags.bits();
    }
}

impl Index<usize> for Register {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.state[index]
    }
}

impl IndexMut<usize> for Register {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.state[index]
    }
}

/// Create Register function
///
/// - Inputs: `15`
///   - `AA2`..`AA0`: Register selection for `DOA`
///   - `WE`: Enable write for selected registers
///   - `WS`: Select which register (A=0/B=1) to use for write operation
///   - `FWE`: Enable write for flag register
///   - `F0I`..`F2I`: Input flags (CF/ZF/NF)
///   - `DI`: Data input to write to selected register
///   - `CLK`: Clock
///   - `CLR`: Reset registers
///   - `AB2`..`AB0`: Register selection for `DOB`
/// - Outputs: `6`
///   - `DOA`: Data output from register selection A
///   - `DOB`: Data output from register selection B
///   - `F0O`..`F3O`: Flags output (CF/ZF/NF/IEF)
pub fn make_register() -> impl FnMut(
    &bool,
    &bool,
    &bool,
    &bool,
    &bool,
    &bool,
    &bool,
    &bool,
    &bool,
    &u8,
    &bool,
    &bool,
    &bool,
    &bool,
    &bool,
) -> (u8, bool, bool, bool, bool, u8) {
    // Internal state
    // - R0: free
    // - R1: free
    // - R2: free
    // - R3: program counter
    // - R4: flag register
    // - R5: stackpointer
    // - R6: temp
    // - R7: temp
    let mut reg = Register::new();
    let mut last_index_a = 0;
    let mut last_index_b = 0;
    // Return function
    move |&aa0, &aa1, &aa2, &we, &ws, &fwe, &f0i, &f1i, &f2i, &di, &clk, &clr, &ab2, &ab1, &ab0| {
        if clr {
            reg.reset();
            last_index_a = 0;
            last_index_b = 0;
        } else if clk {
            debug!("Evaluating Register Function");
            // Convert address bits to usable indices
            let address_a = (aa2 as u8) << 2 + (aa1 as u8) << 1 + (aa0 as u8);
            let address_b = (ab2 as u8) << 2 + (ab1 as u8) << 1 + (ab0 as u8);
            trace!("REGISTER: ADDR A: {}", address_a);
            trace!("REGISTER: ADDR B: {}", address_b);
            // If write enable, overwrite
            if we {
                let write_index = match ws {
                    false => address_a,
                    true => address_a,
                } as usize;
                trace!("REGISTER: Writting {} to {}", di, write_index);
                reg[write_index] = di;
            }
            // If flag should be written
            if fwe {
                let mut flags = Flags::empty();
                flags.set(Flags::CARRY, f0i);
                flags.set(Flags::ZERO, f1i);
                flags.set(Flags::NEGATIVE, f2i);
                reg.set_flags(flags);
            }
        }
        let flags = reg.flags();
        (
            reg[last_index_a],
            flags.contains(Flags::CARRY),
            flags.contains(Flags::ZERO),
            flags.contains(Flags::NEGATIVE),
            flags.contains(Flags::INT_ENABLE),
            reg[last_index_b],
        )
    }
}

pub fn make_instruction_register() -> impl FnMut(&u8, &bool, &bool) -> u8 {
    let mut state = 0;
    move |&memdi, &enable, &clear| {
        debug!(
            "Instruction Register: input: {}, enable: {}, clear: {}",
            memdi, enable, clear
        );
        if clear {
            state = 0;
        } else if enable {
            state = memdi;
        }
        state
    }
}

pub fn make_mpff() -> impl FnMut(
    &MP28BitWord,
    &bool,
    &bool,
) -> (
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
) {
    let mut state = MP28BitWord::empty();
    move |&word, &enable, &clear| {
        if clear {
            state = MP28BitWord::empty();
        } else if enable {
            state = word;
        }
        (
            state.contains(MP28BitWord::MRGAA3),
            state.contains(MP28BitWord::MRGAA2),
            state.contains(MP28BitWord::MRGAA1),
            state.contains(MP28BitWord::MRGAA0),
            state.contains(MP28BitWord::MRGAB3),
            state.contains(MP28BitWord::MRGAB2),
            state.contains(MP28BitWord::MRGAB1),
            state.contains(MP28BitWord::MRGAB0),
            state.contains(MP28BitWord::MCHFLG),
            state.contains(MP28BitWord::MALUS3),
            state.contains(MP28BitWord::MALUS2),
            state.contains(MP28BitWord::MALUS1),
            state.contains(MP28BitWord::MALUS0),
            state.contains(MP28BitWord::MRGWE),
            state.contains(MP28BitWord::MRGWS),
            state.contains(MP28BitWord::MALUIA),
            state.contains(MP28BitWord::MALUIB),
            state.contains(MP28BitWord::MAC3),
            state.contains(MP28BitWord::MAC2),
            state.contains(MP28BitWord::MAC1),
            state.contains(MP28BitWord::MAC0),
            state.contains(MP28BitWord::NA4),
            state.contains(MP28BitWord::NA3),
            state.contains(MP28BitWord::NA2),
            state.contains(MP28BitWord::NA1),
            state.contains(MP28BitWord::NA0),
            state.contains(MP28BitWord::BUSEN),
            state.contains(MP28BitWord::BUSWR),
        )
    }
}

pub fn make_memory_controller() -> impl FnMut(&bool, &bool, &bool) -> (bool, bool, bool, bool) {
    // TODO: How does the memory controller work?
    let mut wait = false;
    move |&enable, &write, &clk| {
        if !enable && wait && clk {
            wait = false;
        } else if enable && clk {
            wait = true;
        }
        (enable, enable, write, wait)
    }
}

pub fn make_arithmetic_logical_unit(
) -> impl FnMut(&bool, &u8, &u8, &bool, &bool, &bool, &bool) -> (bool, bool, bool, u8) {
    move |&cin, &a, &b, &malus3, &malus2, &malus1, &malus0| {
        let selection =
            (malus3 as u8) << 3 + (malus2 as u8) << 2 + (malus1 as u8) << 1 + malus0 as u8;
        let mut cout = None;
        let zout = None;
        let nout = None;
        let out;

        match selection {
            0b0000 => {
                let (o, c) = a.overflowing_add(b);
                out = o;
                cout = Some(c || cin);
            }
            0b0001 => {
                out = a;
            }
            0b0010 => {
                out = !(a | b);
            }
            0b0011 => {
                out = 0;
            }
            0b0100 => {
                let (o, c) = a.overflowing_add(b);
                out = o;
                cout = Some(c);
            }
            0b0101 => {
                let (o, c1) = a.overflowing_add(b);
                let (o, c2) = o.overflowing_add(1);
                out = o;
                cout = Some(!(c1 || c2));
            }
            0b0110 => {
                let (o, c1) = a.overflowing_add(b);
                let (o, c2) = o.overflowing_add(cin.into());
                out = o;
                cout = Some(c1 || c2);
            }
            0b0111 => {
                let (o, c1) = a.overflowing_add(b);
                let (o, c2) = o.overflowing_add((!cin).into());
                out = o;
                cout = Some(!(c1 || c2));
            }
            0b1000 => {
                out = a >> 1;
                cout = Some(a & 1 == 1);
            }
            0b1001 => {
                let (o, c) = a.overflowing_shr(1);
                out = o | (c as u8) << 7;
                cout = Some(c);
            }
            0b1010 => {
                let (o, c) = a.overflowing_shr(1);
                out = o | (cin as u8) << 7;
                cout = Some(c);
            }
            0b1011 => {
                let (o, c) = a.overflowing_shr(1);
                out = o | a & 0b10000000;
                cout = Some(c);
            }
            0b1100 => {
                out = b;
                cout = Some(false);
            }
            0b1101 => {
                out = b;
                cout = Some(true);
            }
            0b1110 => {
                out = b;
                cout = Some(cin);
            }
            0b1111 => {
                out = b;
                cout = Some(!cin);
            }
            16..=255 => unreachable!(),
        }

        (
            cout.unwrap_or(false),
            zout.unwrap_or(out == 0),
            nout.unwrap_or(out & 0b10000000 != 0),
            out,
        )
    }
}
