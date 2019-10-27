//! Everything necessary to run the Terminal User Interface.

use crossterm::KeyEvent;
use lazy_static::lazy_static;
use log::error;
use log::trace;
use log::warn;
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
use crate::supervisor::Supervisor;
use events::Events;
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
    /// The machine's supervisor.
    supervisor: Supervisor,
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
        let supervisor = Supervisor::new();
        let events = Events::new();
        let input_field = Input::new();
        let time_since_last_draw = Instant::now();
        let is_main_loop_running = false;
        Ok(Tui {
            supervisor,
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
            self.supervisor.load_program(path)?;
        }
        self.is_main_loop_running = true;
        while self.is_main_loop_running {
            // Let the supervisor do some work
            self.supervisor.tick();
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
            if !self.supervisor.is_auto_run_mode() {
                thread::sleep(*ONE_MILLISECOND.deref());
            }
            if !self.supervisor.is_at_full_capacity() {
                thread::sleep(*ONE_NANOSECOND.deref());
            }
        }
        backend.clear()?;
        Ok(())
    }
    /// Get the currently running programs path.
    pub fn get_program_path(&self) -> &Option<PathBuf> {
        &self.supervisor.get_program_path()
    }
    /// Handle one single event in the queue.
    fn handle_event(&mut self) {
        if let Some(event) = self.events.next_key() {
            use KeyEvent::*;
            match event {
                Ctrl('c') => self.is_main_loop_running = false,
                Enter => {
                    if self.input_field.is_empty() {
                        self.supervisor.next_clk();
                    } else {
                        self.handle_input();
                    }
                }
                Ctrl('a') => self.supervisor.toggle_auto_run_mode(),
                Ctrl('w') => self.supervisor.toggle_asm_step_mode(),
                Ctrl('e') => {
                    self.supervisor.key_edge_int();
                }
                Ctrl('r') => {
                    self.supervisor.reset();
                }
                Ctrl('l') => {
                    self.supervisor.continue_from_stop();
                }
                Home | End | Tab | BackTab | Backspace | Left | Right | Up | Down | Delete | Char(_) => {
                    self.input_field.handle(event.clone());
                }
                _ => unimplemented!("TUI cannot handle event {:?}", event),
            }
            trace!("{:?}", event);
        }
    }
    /// Handle the input field after an 'Enter'.
    fn handle_input(&mut self) {
        self.input_field.handle(KeyEvent::Enter);
        let query = self.input_field.last().unwrap_or(String::new());
        trace!("Command entered: {}", query);
        if query.starts_with("load ") {
            let path: String = query[5..].into();
            match self.supervisor.load_program(path) {
                Ok(()) => {}
                Err(e) => error!("Failed to run program: {}", e),
            }
        } else if query.starts_with("FC = ") {
            if let Some(x) = parse_input_reg_to_u8(&query[5..]) {
                self.supervisor.input_fc(x)
            } else {
                warn!("Invalid input: {}", query);
            }
        } else if query.starts_with("FD = ") {
            if let Some(x) = parse_input_reg_to_u8(&query[5..]) {
                self.supervisor.input_fd(x)
            } else {
                warn!("Invalid input: {}", query);
            }
        } else if query.starts_with("FE = ") {
            if let Some(x) = parse_input_reg_to_u8(&query[5..]) {
                self.supervisor.input_fe(x)
            } else {
                warn!("Invalid input: {}", query);
            }
        } else if query.starts_with("FF = ") {
            if let Some(x) = parse_input_reg_to_u8(&query[5..]) {
                self.supervisor.input_ff(x)
            } else {
                warn!("Invalid input: {}", query);
            }
        } else if query == "quit" {
            self.is_main_loop_running = false;
        }
    }
}

/// Parse the given [`str`] to u8 with base 16.
/// The input should be something like 'F8'.
fn parse_input_reg_to_u8(input: &str) -> Option<u8> {
    u8::from_str_radix(input, 16).ok()
}

fn init_backend() -> Result<CrosstermBackend, IOError> {
    use crossterm_tui::AlternateScreen;
    let screen = AlternateScreen::to_alternate(true)?;
    CrosstermBackend::with_alternate_screen(screen)
}
