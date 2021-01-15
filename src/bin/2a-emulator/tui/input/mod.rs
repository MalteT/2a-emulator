//! Everything related to the input field of the TUI.
use crossterm::event::{KeyCode, KeyEvent};
use rustyline::completion::FilenameCompleter;

use log::warn;
use nom::{error::ErrorKind as NomErrorKind, Err as NomErr};
use tui::{buffer::Buffer, layout::Rect, style::Color, style::Style, widgets::StatefulWidget};

mod parser;

use crate::{helpers, tui::Part};
use parser::parse_cmd;

/// An Input field widget.
pub struct InputWidget;

/// State needed to draw the input widget.
/// This also keeps track of the input history and
/// handles completions.
pub struct InputState {
    /// Current value of the input box.
    input: Vec<char>,
    /// Cursor position inside the input field.
    input_index: usize,
    /// History of recorded messages.
    history: Vec<String>,
    /// Current position in the history.
    /// This is necessary for arrow key usage.
    history_index: Option<usize>,
    /// Current completions and current index in that list.
    curr_completions: Option<(Vec<Vec<char>>, usize)>,
}

/// Possible input registers
#[derive(Debug, Clone, PartialEq, Hash, Copy, Eq)]
pub enum InputRegister {
    FC,
    FD,
    FE,
    FF,
}

/// Possible commands to enter in the input
#[derive(Debug, Clone, PartialEq)]
pub enum Command<'a> {
    /// Load a program from the path .0.
    LoadProgram(&'a str),
    /// Set the input register .0 to the value .1.
    SetInputReg(InputRegister, u8),
    /// Set the IRG to value .0.
    SetIRG(u8),
    /// Set the TEMP value to value .0.
    SetTEMP(f32),
    /// Set the I1 to value .0.
    SetI1(f32),
    /// Set the I2 to value .0.
    SetI2(f32),
    /// Set the J1 to value .0.
    SetJ1(bool),
    /// Set the J2 to value .0.
    SetJ2(bool),
    /// Set the UIO1 to value .0.
    SetUIO1(bool),
    /// Set the UIO2 to value .0.
    SetUIO2(bool),
    /// Set the UIO3 to value .0.
    SetUIO3(bool),
    /// Show the machine part .0.
    Show(Part),
    /// Quit the program.
    Quit,
}

impl InputState {
    /// Create an new Input widget.
    pub const fn new() -> Self {
        InputState {
            input: Vec::new(),
            input_index: 0,
            history: Vec::new(),
            history_index: None,
            curr_completions: None,
        }
    }
    /// Let the Input widget handle the given event.
    pub fn handle(&mut self, event: KeyEvent) {
        use KeyCode::*;
        match (event.modifiers, event.code) {
            (_, Enter) => {
                if !self.input.is_empty() {
                    self.history.push(self.input.drain(..).collect());
                }
                self.input_index = 0;
                self.history_index = None;
            }
            (_, Tab) => {
                self.next_completion();
            }
            (_, BackTab) => {
                self.previous_completion();
            }
            (_, Char(c)) => {
                self.input.insert(self.input_index, c);
                self.input_index += 1;
            }
            (_, Backspace) => {
                if self.input_index > 0 {
                    self.input_index -= 1;
                    self.input.remove(self.input_index);
                }
            }
            (_, Home) => {
                self.input_index = 0;
            }
            (_, End) => {
                self.input_index = self.input.len();
            }
            (_, Left) => {
                if self.input_index > 0 {
                    self.input_index -= 1;
                }
            }
            (_, Right) => {
                if self.input_index < self.input.len() {
                    self.input_index += 1;
                }
            }
            (_, Up) => match self.history_index {
                Some(index) if index > 0 => {
                    self.history_index = Some(index - 1);
                    self.input = self.history[index - 1].chars().collect();
                    self.input_index = self.input.len();
                }
                None if !self.history.is_empty() => {
                    self.history_index = Some(self.history.len() - 1);
                    self.input = self.history.last().expect("infallible").chars().collect();
                    self.input_index = self.input.len();
                }
                _ => {}
            },
            (_, Down) => match self.history_index {
                Some(index) if index < self.history.len() - 1 => {
                    self.history_index = Some(index + 1);
                    self.input = self.history[index + 1].chars().collect();
                    self.input_index = self.input.len();
                }
                Some(index) if index == self.history.len() - 1 => {
                    self.history_index = None;
                    self.input = vec![];
                    self.input_index = self.input.len();
                }
                _ => {}
            },
            (_, Delete) => {
                if self.input_index < self.input.len() {
                    self.input.remove(self.input_index);
                }
            }
            _ => unreachable!("The input field should not have received a {:?}", event),
        }
        if event.code != Tab && event.code != BackTab {
            self.curr_completions = None;
        }
    }
    /// Check if the input is empty.
    pub fn is_empty(&self) -> bool {
        self.input.len() == 0
    }
    /// Get the last input from the history.
    ///
    /// This excludes the current input of the input field.
    pub fn last(&self) -> Option<String> {
        self.history.last().cloned()
    }
    /// Get the last input as [`Command`].
    ///
    /// This excludes the current input of the input field.
    pub fn last_cmd(&self) -> Option<Command<'_>> {
        self.history.last().and_then(|s| Command::parse(s).ok())
    }
    /// Get the current input.
    pub const fn current(&self) -> &Vec<char> {
        &self.input
    }
    /// Switch to the next completion.
    fn next_completion(&mut self) {
        match &mut self.curr_completions {
            Some((comps, idx)) => {
                *idx = (*idx + 1) % comps.len();
                self.input = comps[*idx].clone();
                self.input_index = self.input.len();
            }
            None => self.complete(),
        }
    }
    /// Switch to the previous completion.
    fn previous_completion(&mut self) {
        match &mut self.curr_completions {
            Some((comps, idx)) => {
                *idx = (*idx as isize - 1) as usize % comps.len();
                self.input = comps[*idx].clone();
                self.input_index = self.input.len();
            }
            None => self.complete(),
        }
    }
    /// Try to complete the current input.
    fn complete(&mut self) {
        let s: String = self.input.iter().collect();
        if let Some(s) = s.strip_prefix("load ") {
            let file_comp = FilenameCompleter::new();
            let s = &s[5..];
            let pos = if self.input_index > 5 {
                self.input_index - 5
            } else {
                0
            };
            let comps = file_comp.complete_path(s, pos);
            match comps {
                Ok((_, comps)) => {
                    let start: String = "load ".into();
                    let comps: Vec<Vec<char>> = comps
                        .iter()
                        .map(|p| &p.replacement)
                        .map(|s| start.clone() + s)
                        .map(|s| s.chars().collect())
                        .collect();
                    self.curr_completions = Some((comps, 0));
                }
                Err(e) => {
                    warn!("Error during completion: {}", e);
                }
            }
        } else if s.starts_with('l') {
            self.curr_completions = Some((vec!["load ".chars().collect()], 0));
        } else if s.starts_with('s') {
            self.curr_completions = Some((vec!["set ".chars().collect()], 0));
        } else if s.starts_with('F') && self.input_index > 1 && self.input_index <= 4 {
            let comp = match &s[1..2] {
                "C" => "FC = ",
                "D" => "FD = ",
                "E" => "FE = ",
                "F" => "FF = ",
                _ => return,
            };
            self.curr_completions = Some((vec![comp.chars().collect()], 0));
        }
        if let Some((ref mut comps, idx)) = self.curr_completions {
            // Add current input to completions
            comps.push(self.input.clone());
            // Select first completions
            self.input = comps[idx].clone();
            self.input_index = self.input.len();
        }
    }
}

impl<'a> Command<'a> {
    /// Try to parse a string into a Command.
    pub fn parse(input: &'a str) -> Result<Self, NomErr<(&str, NomErrorKind)>> {
        parse_cmd(input).map(|(_, out)| out)
    }
}

impl StatefulWidget for InputWidget {
    type State = InputState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let max_string_width = area.width as usize - 3;
        let mut string: String = state.input.iter().collect();
        let mut start = string.len().saturating_sub(max_string_width);
        // Move start to the left to include the cursor
        if start > 0 && start + 5 > state.input_index {
            start = state.input_index.saturating_sub(5);
        }
        // Replace start with dots
        if start > 0 {
            string = String::from("...") + &string[start + 3..];
        }
        // Replace end with dots
        if string.len() > area.width as usize - 3 {
            string.truncate(max_string_width - 3);
            string += "...";
        }
        // Draw prompt
        buf.set_stringn(area.x, area.y, "> ", area.width as usize, *helpers::YELLOW);
        // Draw input chars
        for (i, c) in string.chars().enumerate() {
            buf.set_stringn(
                area.x + 2 + i as u16,
                area.y,
                &format!("{}", c),
                area.width as usize - 2 - i,
                if i == state.input_index - start {
                    Style::default().bg(Color::Yellow)
                } else {
                    Style::default()
                },
            );
        }
        // Draw the cursor if necessary
        if state.input_index == state.input.len() {
            buf.set_stringn(
                area.x + state.input_index as u16 - start as u16 + 2,
                area.y,
                "█",
                1,
                *helpers::YELLOW,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers as Mod;

    macro_rules! key {
        ($key:expr) => {
            KeyEvent {
                modifiers: Mod::empty(),
                code: $key,
            }
        };
    }

    #[test]
    fn is_empty() {
        use KeyCode::*;
        let mut i = InputState::new();
        assert!(i.is_empty());
        i.handle(key!(Char('x')));
        assert!(!i.is_empty());
        i.handle(key!(Backspace));
        assert!(i.is_empty());
    }

    #[test]
    fn basics() {
        use KeyCode::*;
        let mut i = InputState::new();
        assert_eq!(i.history.len(), 0);
        i.handle(key!(Enter));
        assert_eq!(i.history.len(), 0);
        i.handle(key!(Char('x')));
        i.handle(key!(Enter));
        assert_eq!(i.history.len(), 1);
        assert_eq!(i.last(), Some(String::from("x")));
        assert_eq!(i.input_index, 0);
        assert_eq!(i.input.len(), 0);

        i.handle(key!(Backspace));
        i.handle(key!(Char('a')));
        i.handle(key!(Char('b')));
        i.handle(key!(Char('c')));
        assert_eq!(i.input, vec!['a', 'b', 'c']);
        assert_eq!(i.input_index, 3);

        i.handle(key!(Left));
        assert_eq!(i.input_index, 2);

        i.handle(key!(Backspace));
        assert_eq!(i.input, vec!['a', 'c']);

        i.handle(key!(Char('d')));
        i.handle(key!(Right));
        i.handle(key!(Char('d')));
        assert_eq!(i.input, vec!['a', 'd', 'c', 'd']);
    }
}
