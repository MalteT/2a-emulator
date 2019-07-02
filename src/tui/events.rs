use crossterm_input::{input, AsyncReader, InputEvent, KeyEvent as KE};
use log::info;

use std::iter::Map;

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub enum Event {
    Quit,
    Clock,
    Step,
    ToggleAutoRun,
    Reset,
    Interrupt,
    Backspace,
    Char(char),
    Unknown,
}

pub struct Events {
    iter: Map<AsyncReader, Box<dyn FnMut(InputEvent) -> Event>>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub key_exit: Vec<KE>,
    pub key_clock: Vec<KE>,
    pub key_step: Vec<KE>,
    pub key_toggle_auto_run: Vec<KE>,
    pub key_interrupt: Vec<KE>,
    pub key_reset: Vec<KE>,
}

impl Default for Config {
    fn default() -> Config {
        let key_exit = vec![KE::Ctrl('c')];
        let key_toggle_auto_run = vec![KE::Ctrl('a')];
        let key_clock = vec![KE::Char('\n')];
        let key_step = vec![KE::Ctrl('.')];
        let key_interrupt = vec![KE::Ctrl('e')];
        let key_reset = vec![KE::Ctrl('r')];
        Config {
            key_exit,
            key_toggle_auto_run,
            key_clock,
            key_step,
            key_interrupt,
            key_reset,
        }
    }
}

impl Events {
    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
        let f = move |e| Event::from(e, &config);
        let f_box: Box<dyn FnMut(InputEvent) -> Event> = Box::from(f);
        let iter = input().read_async().map(f_box);
        Events { iter }
    }

    pub fn iter<'a>(&'a mut self) -> &'a mut (impl Iterator<Item = Event> + 'a) {
        &mut self.iter
    }

    pub fn next(&mut self) -> Option<Event> {
        self.iter().next()
    }
}

impl Event {
    fn from(tev: InputEvent, config: &Config) -> Self {
        info!("Received input: {:?}", tev);
        match tev {
            InputEvent::Keyboard(ke) => {
                if config.key_exit.contains(&ke) {
                    Event::Quit
                } else if config.key_clock.contains(&ke) {
                    Event::Clock
                } else if config.key_step.contains(&ke) {
                    Event::Step
                } else if config.key_toggle_auto_run.contains(&ke) {
                    Event::ToggleAutoRun
                } else if config.key_interrupt.contains(&ke) {
                    Event::Interrupt
                } else if config.key_reset.contains(&ke) {
                    Event::Reset
                } else if ke == KE::Backspace {
                    Event::Backspace
                } else {
                    match ke {
                        KE::Char(char) => Event::Char(char),
                        _ => Event::Unknown,
                    }
                }
            }
            InputEvent::Mouse(_) | InputEvent::Unsupported(_) | InputEvent::Unknown => {
                Event::Unknown
            }
        }
    }
}
