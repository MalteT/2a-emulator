//! TUI I/O events
use crossterm::event::{self, Event, KeyEvent};

use std::time::Duration;

/// An asynchronous event receiver.
pub struct Events;

impl Events {
    /// Create a new async Event reader.
    pub fn new() -> Events {
        Events
    }
    /// Get the next [`Event`].
    pub fn next(&mut self) -> Option<Event> {
        match event::poll(Duration::from_secs(0)) {
            Ok(true) => event::read().ok(),
            _ => None,
        }
    }
    /// Get the next [`KeyEvent`].
    pub fn next_key(&mut self) -> Option<KeyEvent> {
        self.next().and_then(|ie| match ie {
            Event::Key(ke) => Some(ke),
            _ => None,
        })
    }
}
