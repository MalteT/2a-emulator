//! Everything necessary to run the Terminal User Interface.

use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers as Mod},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::error;
use log::trace;
use log::warn;
use scopeguard::defer;
use tui::backend::CrosstermBackend;
use tui::Terminal;

use std::io::{Stdout, Write};
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

mod board_info_sidebar;
pub mod display;
pub mod events;
pub mod input;
pub mod interface;
mod program_help_sidebar;
mod supervisor_wrapper;

use crate::args::InteractiveArgs;
use crate::compiler::Translator;
use crate::error::Error;
use crate::helpers;
use crate::machine::State;
pub use board_info_sidebar::BoardInfoSidebarWidget;
use events::Events;
use input::{Command, InputRegister, InputState};
use interface::Interface;
pub use program_help_sidebar::{KeybindingHelpState, ProgramDisplayState, ProgramHelpSidebar};
pub use supervisor_wrapper::{Part, SupervisorWrapper, SupervisorWrapperState};

pub type Backend = CrosstermBackend<Stdout>;
type AbortEmulation = bool;

const DURATION_BETWEEN_FRAMES: Duration = Duration::from_micros(1_000_000 / 60);
//const ONE_MICROSECOND: Duration = Duration::from_micros(1);
const ONE_MILLISECOND: Duration = Duration::from_millis(1);
//const DEFAULT_CLK_PERIOD: Duration = Duration::from_nanos((1_000.0 / 7.3728) as u64);

/// The Terminal User Interface (TUI)
pub struct Tui {
    /// The machine's supervisor.
    supervisor: SupervisorWrapperState,
    /// Event iterator.
    events: Events,
    /// The input field at the bottom of the TUI.
    input_field: InputState,
    /// State for the [`KeybindingHelpWidget`].
    keybinding_state: KeybindingHelpState,
    /// State for the [`ProgramDisplayWidget`].
    program_display_state: ProgramDisplayState,
}

impl Tui {
    /// Creates a new Tui and shows it.
    pub fn new(args: &InteractiveArgs) -> Self {
        let supervisor = SupervisorWrapperState::new(&args.init);
        let events = Events::new();
        let input_field = InputState::new();
        let keybinding_state = KeybindingHelpState::init();
        let program_display_state = ProgramDisplayState::empty();
        Tui {
            program_display_state,
            keybinding_state,
            supervisor,
            events,
            input_field,
        }
    }
    /// Create a new TUI from the given command line arguments
    /// and start it immidiately.
    pub fn run_with_args(args: &InteractiveArgs) -> Result<(), Error> {
        let tui = Tui::new(args);
        tui.run(args.program.as_ref())
    }
    /// Run the main loop using the optional asm program.
    pub fn run<P>(mut self, path: Option<P>) -> Result<(), Error>
    where
        P: Into<PathBuf>,
    {
        // This tries to clean everything even if the program panics.
        defer! {
            disable_raw_mode().map_err(Error::crossterm_exit).ok();
            let mut stdout = ::std::io::stdout();
            execute!(stdout, LeaveAlternateScreen).ok();
        }
        // Initialize backend.
        let mut stdout = ::std::io::stdout();
        execute!(stdout, EnterAlternateScreen).map_err(Error::crossterm_init)?;
        enable_raw_mode().map_err(Error::crossterm_init)?;

        let crossterm_backend: Backend = CrosstermBackend::new(stdout);
        let mut backend = Terminal::new(crossterm_backend).map_err(Error::tui_init)?;

        // Initialize interface.
        let mut interface = Interface::new();
        // Clear the terminal and hide the cursor
        backend.clear()?;
        backend.hide_cursor()?;
        // Run program if given.
        if let Some(path) = path {
            self.load_program(path)?;
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
                // Update interface state
                self.maintain();
                // Handle event
                if self.handle_event() {
                    trace!("Quitting application");
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
        Ok(())
    }
    /// Get a reference to the underlying supervisor.
    pub const fn supervisor(&self) -> &SupervisorWrapperState {
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
                    Char('c') => true,
                    Char('a') => {
                        self.supervisor.toggle_auto_run_mode();
                        false
                    }
                    Char('w') => {
                        self.supervisor.toggle_asm_step_mode();
                        false
                    }
                    Char('e') => {
                        self.supervisor.key_edge_int();
                        self.keybinding_state.int_pressed();
                        false
                    }
                    Char('r') => {
                        self.supervisor.reset();
                        self.keybinding_state.reset_pressed();
                        false
                    }
                    Char('l') => {
                        self.supervisor.continue_from_stop();
                        self.keybinding_state.continue_pressed();
                        false
                    }
                    _ => {
                        warn!("TUI cannot handle event {:?}", event);
                        false
                    }
                }
            } else {
                match event.code {
                    Enter => {
                        if self.input_field.is_empty() {
                            self.supervisor.next_clk();
                            self.keybinding_state.clk_pressed();
                            false
                        } else {
                            self.handle_input()
                        }
                    }
                    Home | End | Tab | BackTab | Backspace | Left | Right | Up | Down | Delete
                    | Char(_) => {
                        self.input_field.handle(event);
                        false
                    }
                    _ => {
                        warn!("TUI cannot handle event {:?}", event);
                        false
                    }
                }
            }
        } else {
            false
        }
    }
    /// Handle the input field after an 'Enter'.
    fn handle_input(&mut self) -> AbortEmulation {
        self.input_field.handle(KeyEvent {
            code: KeyCode::Enter,
            modifiers: Mod::empty(),
        });
        if let Some(cmd) = self.input_field.last_cmd() {
            trace!("Command entered: {:?}", cmd);
            match cmd {
                Command::LoadProgram(path) => match self.supervisor.load_program(path) {
                    Ok(()) => {}
                    Err(e) => error!("Failed to run program: {}", e),
                },
                Command::SetInputReg(InputRegister::FC, val) => self.supervisor.input_fc(val),
                Command::SetInputReg(InputRegister::FD, val) => self.supervisor.input_fd(val),
                Command::SetInputReg(InputRegister::FE, val) => self.supervisor.input_fe(val),
                Command::SetInputReg(InputRegister::FF, val) => self.supervisor.input_ff(val),
                Command::SetIRG(val) => self.supervisor.set_irg(val),
                Command::SetTEMP(val) => self.supervisor.set_temp(val),
                Command::SetI1(val) => self.supervisor.set_i1(val),
                Command::SetI2(val) => self.supervisor.set_i2(val),
                Command::SetJ1(val) => self.supervisor.set_j1(val),
                Command::SetJ2(val) => self.supervisor.set_j2(val),
                Command::SetUIO1(val) => self.supervisor.set_uio1(val),
                Command::SetUIO2(val) => self.supervisor.set_uio2(val),
                Command::SetUIO3(val) => self.supervisor.set_uio3(val),
                Command::Show(part) => self.supervisor.show(part),
                Command::Quit => return true,
            }
        } else {
            warn!("Invalid input: {:?}", self.input_field.last());
        }
        false
    }
    fn maintain(&mut self) {
        // Update keybinding state to reflect machine state
        let continue_possible = self.supervisor.machine().state() == State::Stopped;
        self.keybinding_state
            .set_continue_possible(continue_possible);
        let edge_int_possible = self.supervisor.machine().is_key_edge_int_enabled();
        self.keybinding_state
            .set_edge_int_possible(edge_int_possible);
        let asm_step_on = self.supervisor.is_asm_step_mode();
        self.keybinding_state.set_asm_step_on(asm_step_on);
        let autorun_on = self.supervisor.is_auto_run_mode();
        self.keybinding_state.set_autorun_on(autorun_on);
    }
    pub fn load_program<P: Into<PathBuf>>(&mut self, path: P) -> Result<(), Error> {
        let path = path.into();
        let program = helpers::read_asm_file(&path)?;
        let bytecode = Translator::compile(&program);
        // Update the program display state
        self.program_display_state = ProgramDisplayState::from_bytecode(&bytecode);
        self.supervisor.load_program(&path)?;
        Ok(())
    }
}
