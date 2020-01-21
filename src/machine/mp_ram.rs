//! Microprogram ram stuff

use bitflags::bitflags;
use pest::iterators::Pairs;
use pest::Parser;
use pest_derive::Parser;

use std::fmt;
use std::ops::Index;

use super::Signal;

/// Parser for the microprogram words document.
#[derive(Parser)]
#[grammar = "../static/mr-mpram.pest"]
struct MicroprogramRamParser;

/// The microprogram ram.
/// Containing all microprogram words used by the
/// Minirechner 2a as defined in the documentation.
pub struct MicroprogramRam {
    content: [MP28BitWord; 512],
    current_word: usize,
}

bitflags! {
    /// A Word stored in the microprogram ram
    pub struct MP28BitWord: u32 {
        const MAC3       = 0b00001000000000000000000000000000;
        const MAC2       = 0b00000100000000000000000000000000;
        const MAC1       = 0b00000010000000000000000000000000;
        const MAC0       = 0b00000001000000000000000000000000;
        const NA4        = 0b00000000100000000000000000000000;
        const NA3        = 0b00000000010000000000000000000000;
        const NA2        = 0b00000000001000000000000000000000;
        const NA1        = 0b00000000000100000000000000000000;
        const NA0        = 0b00000000000010000000000000000000;
        const BUSWR      = 0b00000000000001000000000000000000;
        const BUSEN      = 0b00000000000000100000000000000000;
        const MRGAA3     = 0b00000000000000010000000000000000;
        const MRGAA2     = 0b00000000000000001000000000000000;
        const MRGAA1     = 0b00000000000000000100000000000000;
        const MRGAA0     = 0b00000000000000000010000000000000;
        const MRGAB3     = 0b00000000000000000001000000000000;
        const MRGAB2     = 0b00000000000000000000100000000000;
        const MRGAB1     = 0b00000000000000000000010000000000;
        const MRGAB0     = 0b00000000000000000000001000000000;
        const MRGWS      = 0b00000000000000000000000100000000;
        const MRGWE      = 0b00000000000000000000000010000000;
        const MALUIA     = 0b00000000000000000000000001000000;
        const MALUIB     = 0b00000000000000000000000000100000;
        const MALUS3     = 0b00000000000000000000000000010000;
        const MALUS2     = 0b00000000000000000000000000001000;
        const MALUS1     = 0b00000000000000000000000000000100;
        const MALUS0     = 0b00000000000000000000000000000010;
        const MCHFLG     = 0b00000000000000000000000000000001;
    }
}

impl MicroprogramRamParser {
    /// Parse to [`MicroprogramRam`].
    ///
    /// The given file needs to be formatted like this:
    /// `addr | instruction | 28BitWord`
    ///
    /// I.e. `00001 | NOP | 01010100 101001 01001 00...`
    pub fn parse_ram(s: &str) -> [MP28BitWord; 512] {
        // Parse the given string using pest
        let parsed = MicroprogramRamParser::parse(Rule::file, s);
        let mut lines = parsed.unwrap();
        let fold_parse = |it: Pairs<Rule>| {
            let mut x = it.fold(String::new(), |mut s, el| {
                s += el.as_str();
                s
            });
            x.retain(|c| c.is_ascii_digit());
            u32::from_str_radix(&x, 2).expect("Not a binary number")
        };
        let mut line_number = 0;
        let mut words = [MP28BitWord::empty(); 512];
        // Iterate over words
        for index in 0..words.len() {
            let line = lines.next().expect("Less than 512 lines in mpram");
            // TODO Use instructions
            for pair in line.into_inner() {
                match pair.as_rule() {
                    Rule::address => {
                        let found_line_number = fold_parse(pair.into_inner());
                        if line_number != found_line_number {
                            panic!(
                                "Missing line number, found: {}, expected: {}",
                                found_line_number, line_number
                            );
                        }
                        line_number += 1;
                    }
                    Rule::instruction => {
                        //words[index].1 = Some(pair.as_str().trim().to_string());
                    }
                    Rule::word => {
                        let word = fold_parse(pair.into_inner());
                        words[index] = MP28BitWord::from_bits_truncate(word);
                    }
                    Rule::sep | Rule::ws => {}
                    _ => unreachable!(),
                }
            }
        }
        words
    }
}

impl MicroprogramRam {
    /// Create a new MicroprogramRam with the default content.
    pub fn new() -> Self {
        let unparsed_content = include_str!("../../static/mr-mpram");
        let content = MicroprogramRamParser::parse_ram(unparsed_content);
        let current_word = 0;
        MicroprogramRam {
            content,
            current_word,
        }
    }
    /// Get the currently active word.
    pub const fn get(&self) -> &MP28BitWord {
        &self.content[self.current_word]
    }
    /// Get the current address.
    pub const fn current_addr(&self) -> u8 {
        self.current_word as u8
    }
    /// Select the next word according to the given parameters.
    pub fn select(&mut self, next_addr: usize) {
        self.current_word = next_addr;
    }
    /// Calculate the next address from the given Signal.
    pub fn get_addr(sig: &Signal) -> usize {
        let a8 = sig.a8();
        let a7 = sig.a7();
        let a6 = sig.a6();
        let a5 = sig.a5();
        let a4 = sig.na4();
        let a3 = sig.na3();
        let a2 = sig.na2();
        let a1 = if sig.mac2() { sig.op11() } else { sig.na1() };
        let a0 = if sig.mac2() {
            sig.op10()
        } else {
            let select = ((sig.mac1() as u8) << 2) + ((sig.mac0() as u8) << 1) + (sig.na0() as u8);
            match select {
                0b000 => false,
                0b001 => true,
                0b010 => {
                    let select = ((sig.op01() as u8) << 1) + (sig.op00() as u8);
                    let am2 = match select {
                        0b00 => true,
                        0b01 => sig.cf(),
                        0b10 => sig.zf(),
                        0b11 => sig.nf(),
                        _ => unreachable!(),
                    };
                    let op10 = sig.op10();
                    // XOR op10 and am2
                    (am2 || op10) && !(am2 && op10)
                }
                0b011 => sig.cf(),
                0b100 => sig.co(),
                0b101 => sig.zo(),
                0b110 => sig.no(),
                0b111 => sig.ief() && (sig.level_int() || sig.edge_int()),
                _ => unreachable!(),
            }
        };
        ((a8 as usize) << 8)
            + ((a7 as usize) << 7)
            + ((a6 as usize) << 6)
            + ((a5 as usize) << 5)
            + ((a4 as usize) << 4)
            + ((a3 as usize) << 3)
            + ((a2 as usize) << 2)
            + ((a1 as usize) << 1)
            + (a0 as usize)
    }
    /// Reset the current address of the microprogram ram.
    pub fn reset(&mut self) {
        self.current_word = 0;
    }
}

impl Index<u16> for MicroprogramRam {
    type Output = MP28BitWord;
    fn index(&self, index: u16) -> &MP28BitWord {
        &self.content[index as usize]
    }
}

impl Default for MP28BitWord {
    fn default() -> Self {
        MP28BitWord::empty()
    }
}

impl fmt::Debug for MicroprogramRam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MicroprogramRam")
            .field("content", &"[hidden]")
            .field("current_word", &self.current_word)
            .finish()
    }
}
