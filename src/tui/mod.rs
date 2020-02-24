//! Everything necessary to run the Terminal User Interface.

use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers as Mod},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::error;
use log::trace;
use log::warn;
use tui::backend::CrosstermBackend;
use tui::Terminal;

use std::io::{Error as IOError, Stdout, Write};
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

pub mod display;
pub mod events;
pub mod input;
pub mod interface;

use crate::error::Error;
use crate::helpers::Configuration;
use crate::supervisor::Supervisor;
use events::Events;
use input::{Command, Input, InputRegister};
use interface::Interface;

pub type Backend = CrosstermBackend<Stdout>;
type AbortEmulation = bool;

const DURATION_BETWEEN_FRAMES: Duration = Duration::from_micros(1_000_000 / 60);
//const ONE_MICROSECOND: Duration = Duration::from_micros(1);
const ONE_MILLISECOND: Duration = Duration::from_millis(1);
//const DEFAULT_CLK_PERIOD: Duration = Duration::from_nanos((1_000.0 / 7.3728) as u64);

/// The Terminal User Interface (TUI)
pub struct Tui {
    /// The machine's supervisor.
    supervisor: Supervisor,
    /// Event iterator.
    events: Events,
    /// The input field at the bottom of the TUI.
    input_field: Input,
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
        let last_reset_press = None;
        let last_clk_press = None;
        let last_int_press = None;
        let last_continue_press = None;
        Ok(Tui {
            supervisor,
            events,
            input_field,
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
        enable_raw_mode().map_err(Error::crossterm_init)?;

        let mut stdout = ::std::io::stdout();
        execute!(stdout, EnterAlternateScreen).map_err(Error::crossterm_init)?;

        let crossterm_backend: Backend = CrosstermBackend::new(stdout);
        let mut backend = Terminal::new(crossterm_backend).map_err(Error::tui_init)?;

        // Initialize interface.
        let mut interface = Interface::new();
        // Clear the terminal and hide the cursor
        backend.clear()?;
        backend.hide_cursor()?;
        // Run program if given.
        if let Some(path) = path {
            self.supervisor.load_program(path)?;
        }
        // Prepare for main loop
        let mut last_draw;
        // Loop until exit is requested
        'outer: loop {
            // Next draw of the machine
            backend.draw(|mut f| {
                interface.draw(&mut self, &mut f);
            })?;
            last_draw = Instant::now();
            // Loop until the next draw is necessary
            while last_draw.elapsed() < DURATION_BETWEEN_FRAMES {
                // Let the supervisor do some work
                self.supervisor.tick();
                // Handle event
                if self.handle_event() {
                    // Quit
                    break 'outer;
                }
                if !self.supervisor.is_auto_run_mode() {
                    thread::sleep(10 * ONE_MILLISECOND);
                }
            }
        }
        backend.clear()?;
        backend.show_cursor()?;
        execute!(backend.backend_mut(), LeaveAlternateScreen).map_err(Error::crossterm_exit)?;
        disable_raw_mode().map_err(Error::crossterm_exit)?;
        Ok(())
    }
    /// Get a reference to the underlying supervisor.
    pub const fn supervisor(&self) -> &Supervisor {
        &self.supervisor
    }
    /// Handle one single event in the queue.
    /// Returns whether to abort emulation or not.
    fn handle_event(&mut self) -> AbortEmulation {
        if let Some(event) = self.events.next_key() {
            use KeyCode::*;
            trace!("{:?}", event);
            if event.modifiers == Mod::CONTROL {
                match event.code {
                    Char('c') => return true,
                    Char('a') => {
                        self.supervisor.toggle_auto_run_mode();
                    }
                    Char('w') => {
                        self.supervisor.toggle_asm_step_mode();
                    }
                    Char('e') => {
                        self.supervisor.key_edge_int();
                        self.last_int_press = Some(Instant::now());
                    }
                    Char('r') => {
                        self.supervisor.reset();
                        self.last_reset_press = Some(Instant::now());
                    }
                    Char('l') => {
                        self.supervisor.continue_from_stop();
                        self.last_continue_press = Some(Instant::now());
                    }
                    _ => warn!("TUI cannot handle event {:?}", event),
                }
            } else if event.modifiers == Mod::empty() {
                match event.code {
                    Enter => {
                        if self.input_field.is_empty() {
                            self.supervisor.next_clk();
                            self.last_clk_press = Some(Instant::now());
                        } else {
                            self.handle_input();
                        }
                    }
                    Home | End | Tab | BackTab | Backspace | Left | Right | Up | Down | Delete
                    | Char(_) => {
                        self.input_field.handle(event);
                    }
                    _ => warn!("TUI cannot handle event {:?}", event),
                }
            }
        }
        false
    }
    /// Handle the input field after an 'Enter'.
    fn handle_input(&mut self) -> AbortEmulation {
        self.input_field.handle(KeyEvent {
            code: KeyCode::Enter,
            modifiers: Mod::empty(),
        });
        if let Some(cmd) = self.input_field.last_cmd() {
            trace!("Command entered: {:?}", cmd);
            use Command::*;
            match cmd {
                LoadProgram(path) => match self.supervisor.load_program(path) {
                    Ok(()) => {}
                    Err(e) => error!("Failed to run program: {}", e),
                },
                SetInputReg(InputRegister::FC, val) => self.supervisor.input_fc(val),
                SetInputReg(InputRegister::FD, val) => self.supervisor.input_fd(val),
                SetInputReg(InputRegister::FE, val) => self.supervisor.input_fe(val),
                SetInputReg(InputRegister::FF, val) => self.supervisor.input_ff(val),
                SetIRG(val) => self.supervisor.set_irg(val),
                SetTEMP(val) => self.supervisor.set_temp(val),
                SetI1(val) => self.supervisor.set_i1(val),
                SetI2(val) => self.supervisor.set_i2(val),
                SetJ1(val) => self.supervisor.set_j1(val),
                SetJ2(val) => self.supervisor.set_j2(val),
                SetUIO1(val) => self.supervisor.set_uio1(val),
                SetUIO2(val) => self.supervisor.set_uio2(val),
                SetUIO3(val) => self.supervisor.set_uio3(val),
                Show(part) => self.supervisor.show(part),
                Quit => return true,
            }
        } else {
            warn!("Invalid input: {:?}", self.input_field.last());
        }
        false
    }
    pub const fn input_field(&self) -> &Input {
        &self.input_field
    }
}
