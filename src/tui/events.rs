//! TUI I/O events

use crossterm::{input, AsyncReader, InputEvent, KeyEvent};
use log::info;

pub struct Events {
    iter: AsyncReader,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub key_exit: Vec<KeyEvent>,
    pub key_clock: Vec<KeyEvent>,
    pub key_step: Vec<KeyEvent>,
    pub key_toggle_auto_run: Vec<KeyEvent>,
    pub key_toggle_asm_step: Vec<KeyEvent>,
    pub key_interrupt: Vec<KeyEvent>,
    pub key_reset: Vec<KeyEvent>,
    pub key_continue: Vec<KeyEvent>,
}

impl Default for Config {
    fn default() -> Config {
        let key_exit = vec![KeyEvent::Ctrl('c')];
        let key_toggle_auto_run = vec![KeyEvent::Ctrl('a')];
        let key_toggle_asm_step = vec![KeyEvent::Ctrl('w')];
        let key_clock = vec![KeyEvent::Char('\n')];
        let key_step = vec![KeyEvent::Ctrl('.')];
        let key_interrupt = vec![KeyEvent::Ctrl('e')];
        let key_reset = vec![KeyEvent::Ctrl('r')];
        let key_continue = vec![KeyEvent::Ctrl('l')];
        Config {
            key_exit,
            key_toggle_auto_run,
            key_toggle_asm_step,
            key_clock,
            key_step,
            key_interrupt,
            key_reset,
            key_continue,
        }
    }
}

impl Events {
    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
        Events {
            iter: input().read_async(),
        }
    }

    pub fn iter<'a>(&'a mut self) -> &'a mut AsyncReader {
        &mut self.iter
    }

    pub fn next(&mut self) -> Option<InputEvent> {
        self.iter().next()
    }

    pub fn next_key(&mut self) -> Option<KeyEvent> {
        self.next().and_then(|ie| match ie {
            InputEvent::Keyboard(ke) => Some(ke),
            _ => None,
        })
    }
}
