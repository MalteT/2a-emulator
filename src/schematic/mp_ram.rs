//! Microprogram ram stuff

use bitflags::bitflags;
use pest::iterators::Pairs;
use pest::Parser;
use pest_derive::Parser;

use std::ops::Index;
use std::fmt;

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
    /// Parse to [`Ram`].
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
        let current_word= 0;
        MicroprogramRam { content, current_word }
    }
    /// Get the currently active word.
    pub fn get(&self) -> &MP28BitWord {
        &self.content[self.current_word]
    }
    // /// Select the next word according to the given parameters.
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
