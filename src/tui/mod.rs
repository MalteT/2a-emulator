//! Everything necessary to run the Terminal User Interface.

use lazy_static::lazy_static;
use log::error;
use log::trace;
use tui::backend::CrosstermBackend;
use tui::Terminal;

use std::io::Error as IOError;
use std::ops::Deref;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

pub mod display;
pub mod events;
pub mod grid;
pub mod input;
pub mod interface;

use crate::error::Error;
use crate::executor::Executor;
use events::{Event, Events};
use input::Input;
use interface::Interface;

lazy_static! {
    static ref DURATION_BETWEEN_FRAMES: Duration = Duration::from_micros(16_666);
    static ref ONE_NANOSECOND: Duration = Duration::from_nanos(1);
    static ref ONE_MICROSECOND: Duration = Duration::from_micros(1);
    static ref ONE_MILLISECOND: Duration = Duration::from_millis(1);
    static ref DEFAULT_CLK_PERIOD: Duration = Duration::from_nanos((1_000.0 / 7.3728) as u64);
}

/// The Terminal User Interface (TUI)
pub struct Tui {
    /// The machine executor.
    executor: Executor,
    /// Event iterator.
    events: Events,
    /// The input field at the bottom of the TUI.
    input_field: Input,
    time_since_last_draw: Instant,
    /// Time between two clock rising edges.
    is_main_loop_running: bool,
}

impl Tui {
    /// Creates a new Tui and shows it.
    pub fn new() -> Result<Self, IOError> {
        let executor = Executor::new();
        let events = Events::new();
        let input_field = Input::new();
        let time_since_last_draw = Instant::now();
        let is_main_loop_running = false;
        Ok(Tui {
            executor,
            events,
            input_field,
            time_since_last_draw,
            is_main_loop_running,
        })
    }
    /// Run the main loop using the optional asm program.
    pub fn run<P>(mut self, path: Option<P>) -> Result<(), Error>
    where
        P: Into<PathBuf>,
    {
        // Initialize backend.
        let mut backend = Terminal::new(init_backend()?)?;
        // Initialize interface.
        let mut interface = Interface::new();
        // Clear the terminal and hide the cursor
        backend.clear()?;
        backend.hide_cursor()?;
        // Run program if given.
        if let Some(path) = path {
            self.executor.execute(path)?;
        }
        self.is_main_loop_running = true;
        while self.is_main_loop_running {
            // Let the executor do some work
            self.executor.tick();
            // Handle event
            self.handle_event();
            // Next draw of the machine
            let now = Instant::now();
            if now - self.time_since_last_draw >= *DURATION_BETWEEN_FRAMES.deref() {
                self.time_since_last_draw = now;
                backend.draw(|mut f| {
                    interface.draw(&mut self, &mut f);
                })?;
            }
            if !self.executor.is_auto_run_mode() {
                thread::sleep(*ONE_MICROSECOND.deref());
            }
            if !self.executor.is_at_full_capacity() {
                thread::sleep(*ONE_NANOSECOND.deref());
            }
        }
        backend.clear()?;
        Ok(())
    }
    /// Get the currently running programs path.
    pub fn get_program_path(&self) -> &Option<PathBuf> {
        &self.executor.get_program_path()
    }
    /// Handle one single event in the queue.
    fn handle_event(&mut self) {
        if let Some(event) = self.events.next() {
            match event {
                Event::Quit => self.is_main_loop_running = false,
                Event::Clock => {
                    // Only interpret Enter as CLK if no text was input
                    if self.input_field.is_empty() {
                        self.executor.next_clk();
                    } else {
                        self.input_field.handle(Event::Char('\n'));
                        let query = self.input_field.pop();
                        self.handle_input(&query);
                        trace!("Command entered: {}", query);
                    }
                }
                Event::Step => {}
                Event::ToggleAutoRun => self.executor.toggle_auto_run_mode(),
                Event::Interrupt => {
                    self.executor.edge_int();
                }
                Event::Reset => {
                    self.executor.reset();
                }
                Event::Backspace | Event::Char(_) => {
                    self.input_field.handle(event.clone());
                }
                x => unimplemented!("{:#?}", x),
            }
            trace!("{:?}", event);
        }
    }
    /// Handle the given input string.
    fn handle_input(&mut self, query: &String) {
        if query.starts_with("load ") {
            let path: String = query[5..].into();
            match self.executor.execute(path) {
                Ok(()) => {}
                Err(e) => error!("Failed to run program: {}", e),
            }
        } else if query == "quit" {
            self.is_main_loop_running = false;
        }
    }
}

fn init_backend() -> Result<CrosstermBackend, IOError> {
    use crossterm::{AlternateScreen, TerminalOutput};
    let stdout = TerminalOutput::new(true);
    let screen = AlternateScreen::to_alternate_screen(stdout, true)?;
    CrosstermBackend::with_alternate_screen(screen)
}
