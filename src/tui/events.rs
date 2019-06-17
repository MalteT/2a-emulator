use log::info;
use std::io;
use std::sync::mpsc;
use std::thread;

use termion::event::Event as TermEvent;
use termion::event::Key;
use termion::input::TermRead;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Event {
    Quit,
    Clock,
    Step,
    ToggleAutoRun,
    Reset,
    Interrupt,
    Backspace,
    Char(char),
    Unsupported(TermEvent),
}

pub struct Events {
    rx: mpsc::Receiver<Event>,
    _input_handle: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub key_exit: Key,
    pub key_clock: Key,
    pub key_step: Key,
    pub key_toggle_auto_run: Key,
    pub key_interrupt: Key,
    pub key_reset: Key,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            key_exit: Key::Ctrl('c'),
            key_toggle_auto_run: Key::Ctrl('a'),
            key_clock: Key::Char('\n'),
            key_step: Key::Ctrl('.'),
            key_interrupt: Key::Ctrl('e'),
            key_reset: Key::Ctrl('r'),
        }
    }
}

impl Events {
    pub fn new() -> Events {
        Events::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Events {
        let (tx, rx) = mpsc::channel();
        let input_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                let event_iter = stdin
                    .events()
                    .filter(Result::is_ok)
                    .map(Result::unwrap)
                    .map(|term_event| Event::from(term_event, config))
                    .map(|event| tx.send(event));
                for result in event_iter {
                    if result.is_err() {
                        return;
                    }
                }
            })
        };
        Events {
            rx,
            _input_handle: input_handle,
        }
    }

    pub fn iter(&self) -> std::sync::mpsc::Iter<Event> {
        self.rx.iter()
    }

    pub fn try_iter(&self) -> std::sync::mpsc::TryIter<Event> {
        self.rx.try_iter()
    }

    pub fn next(&self) -> Result<Event, mpsc::RecvError> {
        self.rx.recv()
    }
}

impl Event {
    fn from(tev: TermEvent, config: Config) -> Self {
        info!("Received input: {:?}", tev);
        match tev {
            TermEvent::Key(key) => {
                if key == config.key_exit {
                    Event::Quit
                } else if key == config.key_clock {
                    Event::Clock
                } else if key == config.key_step {
                    Event::Step
                } else if key == config.key_toggle_auto_run {
                    Event::ToggleAutoRun
                } else if key == config.key_interrupt {
                    Event::Interrupt
                } else if key == config.key_reset {
                    Event::Reset
                } else if key == Key::Backspace {
                    Event::Backspace
                } else {
                    match key {
                        Key::Char(char) => Event::Char(char),
                        _ => Event::Unsupported(tev),
                    }
                }
            }
            TermEvent::Mouse(_) | TermEvent::Unsupported(_) => Event::Unsupported(tev),
        }
    }
}
