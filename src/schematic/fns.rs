//! # Schematic functionality
//!
//! All necessary functions for building the schematic for the
//! Minirechner 2a schematic.

use bitflags::bitflags;
use log::{debug, trace};

use std::ops::{Index, IndexMut};

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
) -> impl FnMut(&bool, &bool, &bool, &bool, &bool, &bool, &bool, &bool, &bool, &bool, &bool) -> bool {
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
        debug!( "DFLIPFLOP: input: {}, clk: {}", input, clk);
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
        debug!( "DFLIPFLOP: input: {}, clk: {}, clear: {}", input, clk, clear);
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
///   - `AB2`..`AB0`: Register selection for `DOB`
///   - `WE`: Enable write for selected registers
///   - `WS`: Select which register (A=0/B=1) to use for write operation
///   - `FWE`: Enable write for flag register
///   - `F0I`..`F2I`: Input flags (CF/ZF/NF)
///   - `DI`: Data input to write to selected register
///   - `CLK`: Clock
///   - `CLR`: Reset registers
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
    &bool,
    &bool,
    &bool,
    &u8,
    &bool,
    &bool,
) -> (u8, u8, bool, bool, bool, bool) {
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
    move |&aa2, &aa1, &aa0, &ab2, &ab1, &ab0, &we, &ws, &fwe, &f0i, &f1i, &f2i, &di, &clk, &clr| {
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
            reg[last_index_b],
            flags.contains(Flags::CARRY),
            flags.contains(Flags::ZERO),
            flags.contains(Flags::NEGATIVE),
            flags.contains(Flags::INT_ENABLE),
        )
    }
}
