use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::Widget;

use super::events::Event;

/// An Input widget
pub struct Input {
    /// Current value of the input box
    input: String,
    /// History of recorded messages
    messages: Vec<String>,
}

impl Input {
    /// Create an new Input widget.
    pub fn new() -> Self {
        Self::default()
    }
    /// Let the Input widget handle the given event.
    pub fn handle(&mut self, event: Event) {
        match event {
            Event::Char('\n') => {
                self.messages.push(self.input.drain(..).collect());
            }
            Event::Char(c) => {
                self.input.push(c);
            }
            Event::Backspace => {
                self.input.pop();
            }
            _ => unimplemented!(),
        }
    }
    /// Check if the input is empty.
    pub fn is_empty(&self) -> bool {
        self.input.len() == 0
    }
    /// Get the most recently entered input.
    pub fn pop(&mut self) -> String {
        self.messages.pop().unwrap_or_default()
    }
}

impl Widget for Input {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let s = format!("> {}", &self.input);
        buf.set_stringn(
            area.x,
            area.y,
            &s,
            area.width as usize,
            Style::default().fg(Color::Yellow),
        );
    }
}

impl Default for Input {
    fn default() -> Input {
        Input {
            input: String::new(),
            messages: Vec::new(),
        }
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
