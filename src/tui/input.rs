//! Simple input field for the TUI.

use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Style;
use tui::widgets::Widget;
use tui::style::Color;
use log::error;

use super::events::Event;
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
    pub fn handle(&mut self, event: Event) {
        match event {
            Event::Char('\n') => {
                if self.input.len() > 0 {
                    self.history.push(self.input.drain(..).collect());
                }
                self.input_index = 0;
                self.history_index = None;
            }
            Event::Char('\t') => {
                error!("No completion implemented yet!");
            }
            Event::Char(c) => {
                self.input.insert(self.input_index, c);
                self.input_index += 1;
            }
            Event::Backspace => {
                if self.input_index > 0 {
                    self.input_index -= 1;
                    self.input.remove(self.input_index);
                }
            }
            Event::Left => {
                if self.input_index > 0 {
                    self.input_index -= 1;
                }
            }
            Event::Right => {
                if self.input_index < self.input.len() {
                    self.input_index += 1;
                }
            }
            Event::Delete => {
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
        let mut i = Input::new();
        assert!(i.is_empty());
        i.handle(Event::Char('x'));
        assert!(!i.is_empty());
        i.handle(Event::Backspace);
        assert!(i.is_empty());
    }

    #[test]
    fn basics() {
        let mut i = Input::new();
        assert_eq!(i.history.len(), 0);
        i.handle(Event::Char('\n'));
        assert_eq!(i.history.len(), 0);
        i.handle(Event::Char('x'));
        i.handle(Event::Char('\n'));
        assert_eq!(i.history.len(), 1);
        assert_eq!(i.last(), Some(String::from("x")));
        assert_eq!(i.input_index, 0);
        assert_eq!(i.input.len(), 0);

        i.handle(Event::Backspace);
        i.handle(Event::Char('a'));
        i.handle(Event::Char('b'));
        i.handle(Event::Char('c'));
        assert_eq!(i.input, vec!['a', 'b', 'c']);
        assert_eq!(i.input_index, 3);

        i.handle(Event::Left);
        assert_eq!(i.input_index, 2);

        i.handle(Event::Backspace);
        assert_eq!(i.input, vec!['a', 'c']);

        i.handle(Event::Char('d'));
        i.handle(Event::Right);
        i.handle(Event::Char('d'));
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
