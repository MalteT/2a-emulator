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
pub mod input;
pub mod interface;

use crate::error::Error;
use crate::helpers::Configuration;
use crate::machine::Part;
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
    last_reset_press: Option<Instant>,
    last_clk_press: Option<Instant>,
    last_int_press: Option<Instant>,
    last_continue_press: Option<Instant>,
}

impl Tui {
    /// Creates a new Tui and shows it.
    pub fn new(conf: &Configuration) -> Result<Self, IOError> {
        let supervisor = Supervisor::new(conf);
        let events = Events::new();
        let input_field = Input::new();
        let time_since_last_draw = Instant::now();
        let is_main_loop_running = false;
        let last_reset_press = None;
        let last_clk_press = None;
        let last_int_press = None;
        let last_continue_press = None;
        Ok(Tui {
            supervisor,
            events,
            input_field,
            time_since_last_draw,
            is_main_loop_running,
            last_reset_press,
            last_clk_press,
            last_int_press,
            last_continue_press,
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
                        self.last_clk_press = Some(Instant::now());
                    } else {
                        self.handle_input();
                    }
                }
                Ctrl('a') => {
                    self.supervisor.toggle_auto_run_mode();
                }
                Ctrl('w') => {
                    self.supervisor.toggle_asm_step_mode();
                }
                Ctrl('e') => {
                    self.supervisor.key_edge_int();
                    self.last_int_press = Some(Instant::now());
                }
                Ctrl('r') => {
                    self.supervisor.reset();
                    self.last_reset_press = Some(Instant::now());
                }
                Ctrl('l') => {
                    self.supervisor.continue_from_stop();
                    self.last_continue_press = Some(Instant::now());
                }
                Home | End | Tab | BackTab | Backspace | Left | Right | Up | Down | Delete
                | Char(_) => {
                    self.input_field.handle(event.clone());
                }
                _ => warn!("TUI cannot handle event {:?}", event),
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
        } else if query.starts_with("set ") {
            let query = &query[4..];
            if query.starts_with("IRG = ") {
                if let Some(x) = query[6..].parse().ok() {
                    self.supervisor.set_irg(x);
                } else {
                    warn!("Invalid setting: {}", query);
                }
            } else if query.starts_with("TEMP = ") {
                if let Some(x) = query[7..].parse().ok() {
                    self.supervisor.set_temp(x);
                } else {
                    warn!("Invalid setting: {}", query);
                }
            } else if query == "J1" {
                self.supervisor.set_j1(true);
            } else if query == "J2" {
                self.supervisor.set_j2(true);
            } else if query == "J2" {
                self.supervisor.set_j2(true);
            } else if query.starts_with("I1 = ") {
                if let Some(x) = query[5..].parse().ok() {
                    self.supervisor.set_i1(x);
                } else {
                    warn!("Invalid setting: {}", query);
                }
            } else if query.starts_with("I2 = ") {
                if let Some(x) = query[5..].parse().ok() {
                    self.supervisor.set_i2(x);
                } else {
                    warn!("Invalid setting: {}", query);
                }
            } else if query.starts_with("UIO1 = ") {
                if let Some(x) = query[7..].parse().ok() {
                    self.supervisor.set_uio1(x);
                } else {
                    warn!("Invalid setting: {}", query);
                }
            } else if query.starts_with("UIO2 = ") {
                if let Some(x) = query[7..].parse().ok() {
                    self.supervisor.set_uio2(x);
                } else {
                    warn!("Invalid setting: {}", query);
                }
            } else if query.starts_with("UIO3 = ") {
                if let Some(x) = query[7..].parse().ok() {
                    self.supervisor.set_uio3(x);
                } else {
                    warn!("Invalid setting: {}", query);
                }
            } else {
                warn!("Invalid setting: {}", query);
            }
        } else if query.starts_with("unset ") {
            let query = &query[6..];
            if query == "J1" {
                self.supervisor.set_j1(false);
            } else if query == "J2" {
                self.supervisor.set_j2(false);
            } else if query == "J2" {
                self.supervisor.set_j2(false);
            } else {
                warn!("Invalid setting: {}", query);
            }
        } else if query.starts_with("show ") {
            let query = &query[5..];
            if query == "memory" {
                self.supervisor.show(Part::Memory);
            } else {
                self.supervisor.show(Part::RegisterBlock);
            }
        } else if query == "quit" {
            self.is_main_loop_running = false;
        } else {
            warn!("Unrecognized input: {}", query);
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
