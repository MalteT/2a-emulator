//! TUI I/O events

use crossterm::event::{self, Event, KeyEvent};

use std::time::Duration;

pub struct Events;

//#[derive(Debug, Clone)]
//pub struct Config {
//    pub key_exit: Vec<KeyEvent>,
//    pub key_clock: Vec<KeyEvent>,
//    pub key_step: Vec<KeyEvent>,
//    pub key_toggle_auto_run: Vec<KeyEvent>,
//    pub key_toggle_asm_step: Vec<KeyEvent>,
//    pub key_interrupt: Vec<KeyEvent>,
//    pub key_reset: Vec<KeyEvent>,
//    pub key_continue: Vec<KeyEvent>,
//}
//
//impl Default for Config {
//    fn default() -> Config {
//        let key_exit = vec![KeyEvent::Ctrl('c')];
//        let key_toggle_auto_run = vec![KeyEvent::Ctrl('a')];
//        let key_toggle_asm_step = vec![KeyEvent::Ctrl('w')];
//        let key_clock = vec![KeyEvent::Char('\n')];
//        let key_step = vec![KeyEvent::Ctrl('.')];
//        let key_interrupt = vec![KeyEvent::Ctrl('e')];
//        let key_reset = vec![KeyEvent::Ctrl('r')];
//        let key_continue = vec![KeyEvent::Ctrl('l')];
//        Config {
//            key_exit,
//            key_toggle_auto_run,
//            key_toggle_asm_step,
//            key_clock,
//            key_step,
//            key_interrupt,
//            key_reset,
//            key_continue,
//        }
//    }
//}

impl Events {
    /// Create a new async Event reader.
    pub fn new() -> Events {
        Events
    }
    /// Get the next [`InputEvent`].
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
