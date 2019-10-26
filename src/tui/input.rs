//! Simple input field for the TUI.
use crossterm::KeyEvent;

use log::warn;
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Color;
use tui::style::Style;
use tui::widgets::Widget;

use crate::helpers;

/// An Input widget
pub struct Input {
    /// Current value of the input box.
    input: Vec<char>,
    /// Cursor position inside the input field.
    input_index: usize,
    /// History of recorded messages.
    history: Vec<String>,
    /// Current position in the history.
    /// This is necessary for arrow key usage.
    history_index: Option<usize>,
}

impl Input {
    /// Create an new Input widget.
    pub fn new() -> Self {
        Input {
            input: vec![],
            input_index: 0,
            history: Vec::new(),
            history_index: None,
        }
    }
    /// Let the Input widget handle the given event.
    pub fn handle(&mut self, event: KeyEvent) {
        use KeyEvent::*;
        match event {
            Enter => {
                if self.input.len() > 0 {
                    self.history.push(self.input.drain(..).collect());
                }
                self.input_index = 0;
                self.history_index = None;
            }
            Tab => {
                warn!("No completion implemented yet!");
            }
            Char(c) => {
                self.input.insert(self.input_index, c);
                self.input_index += 1;
            }
            Backspace => {
                if self.input_index > 0 {
                    self.input_index -= 1;
                    self.input.remove(self.input_index);
                }
            }
            Left => {
                if self.input_index > 0 {
                    self.input_index -= 1;
                }
            }
            Right => {
                if self.input_index < self.input.len() {
                    self.input_index += 1;
                }
            }
            Up => match self.history_index {
                Some(index) if index > 0 => {
                    self.history_index = Some(index - 1);
                    self.input = self.history[index - 1].chars().collect();
                    self.input_index = self.input.len();
                }
                None if self.history.len() > 0 => {
                    self.history_index = Some(self.history.len() - 1);
                    self.input = self.history.last().expect("infallible").chars().collect();
                    self.input_index = self.input.len();
                }
                _ => {}
            },
            Down => match self.history_index {
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
            Delete => {
                if self.input_index < self.input.len() {
                    self.input.remove(self.input_index);
                }
            }
            _ => unimplemented!(),
        }
    }
    /// Check if the input is empty.
    pub fn is_empty(&self) -> bool {
        self.input.len() == 0
    }
    /// Get the last input from the history.
    pub fn last(&self) -> Option<String> {
        self.history.last().cloned()
    }
}

impl Widget for Input {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        buf.set_stringn(area.x, area.y, "> ", area.width as usize, *helpers::YELLOW);
        for (i, c) in self.input.iter().enumerate() {
            buf.set_stringn(
                area.x + 2 + i as u16,
                area.y,
                &format!("{}", c),
                area.width as usize - 2 - i,
                if i == self.input_index {
                    Style::default().bg(Color::Yellow)
                } else {
                    Style::default()
                },
            );
        }
        // Draw the box if necessary
        if self.input_index == self.input.len() {
            buf.set_stringn(
                area.x + self.input.len() as u16 + 2,
                area.y,
                "█",
                area.width as usize - self.input.len() - 2,
                *helpers::YELLOW,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_empty() {
        use KeyEvent::*;
        let mut i = Input::new();
        assert!(i.is_empty());
        i.handle(Char('x'));
        assert!(!i.is_empty());
        i.handle(Backspace);
        assert!(i.is_empty());
    }

    #[test]
    fn basics() {
        use KeyEvent::*;
        let mut i = Input::new();
        assert_eq!(i.history.len(), 0);
        i.handle(Enter);
        assert_eq!(i.history.len(), 0);
        i.handle(Char('x'));
        i.handle(Enter);
        assert_eq!(i.history.len(), 1);
        assert_eq!(i.last(), Some(String::from("x")));
        assert_eq!(i.input_index, 0);
        assert_eq!(i.input.len(), 0);

        i.handle(Backspace);
        i.handle(Char('a'));
        i.handle(Char('b'));
        i.handle(Char('c'));
        assert_eq!(i.input, vec!['a', 'b', 'c']);
        assert_eq!(i.input_index, 3);

        i.handle(Left);
        assert_eq!(i.input_index, 2);

        i.handle(Backspace);
        assert_eq!(i.input, vec!['a', 'c']);

        i.handle(Char('d'));
        i.handle(Right);
        i.handle(Char('d'));
        assert_eq!(i.input, vec!['a', 'd', 'c', 'd']);
    }
}

// fn main() -> Result<(), failure::Error> {
//     // Terminal initialization
//     let stdout = io::stdout().into_raw_mode()?;
//     let stdout = MouseTerminal::from(stdout);
//     let stdout = AlternateScreen::from(stdout);
//     let backend = TermionBackend::new(stdout);
//     let mut terminal = Terminal::new(backend)?;
//
//     // Setup event handlers
//     let events = Events::new();
//
//     // Create default app state
//     let mut app = App::default();
//
//     loop {
//         // Draw UI
//         terminal.draw(|mut f| {
//             let chunks = Layout::default()
//                 .direction(Direction::Vertical)
//                 .margin(2)
//                 .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
//                 .split(f.size());
//             Paragraph::new([Text::raw(&app.input)].iter())
//                 .style(Style::default().fg(Color::Yellow))
//                 .block(Block::default().borders(Borders::ALL).title("Input"))
//                 .render(&mut f, chunks[0]);
//         })?;
//
//         // Put the cursor back inside the input box
//         write!(
//             terminal.backend_mut(),
//             "{}",
//             Goto(4 + app.input.width() as u16, 4)
//         )?;
//         // stdout is buffered, flush it to see the effect immediately when hitting backspace
//         io::stdout().flush().ok();
//     }
//     Ok(())
// }
