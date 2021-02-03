//! Everything necessary to run the Terminal User Interface.
use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers as Mod},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use emulator_2a_lib::{
    compiler::Translator,
    machine::{State, StepMode},
};
use log::{trace, warn};
use scopeguard::defer;
use tui::{backend::CrosstermBackend, Terminal};

use std::{
    io::{Stdout, Write},
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

mod board_info_sidebar;
pub mod display;
pub mod events;
pub mod input;
pub mod interface;
mod program_help_sidebar;
pub mod show_widgets;
mod supervisor_wrapper;

use crate::{args::InteractiveArgs, error::Error, helpers};
pub use board_info_sidebar::BoardInfoSidebarWidget;
use events::Events;
use input::{Command, InputRegister, InputState};
use interface::Interface;
pub use program_help_sidebar::{KeybindingHelpState, ProgramDisplayState, ProgramHelpSidebar};
pub use supervisor_wrapper::{MachineState, MachineWidget, Part};

pub type Backend = CrosstermBackend<Stdout>;
type AbortEmulation = bool;

const DURATION_BETWEEN_FRAMES: Duration = Duration::from_micros(1_000_000 / 60);
const ONE_MILLISECOND: Duration = Duration::from_millis(1);

/// The Terminal User Interface (TUI)
pub struct Tui {
    /// State for the [`MachineWidget`].
    /// It contains the [`Machine`](emulator_2a_lib::machine::Machine)
    /// which contains the [`RawMachine`](emulator_2a_lib::machine::RawMachine).
    machine: MachineState,
    /// Event iterator.
    events: Events,
    /// State for the input field at the bottom of the TUI.
    /// This is needed to draw the [`InputWidget`](crate::tui::input::InputWidget).
    input_field: InputState,
    /// State for the
    /// [`KeybindingHelpWidget`](program_help_sidebar::KeybindingHelpWidget).
    keybinding_state: KeybindingHelpState,
    /// State for the
    /// [`ProgramDisplayWidget`](program_help_sidebar::KeybindingHelpWidget).
    program_display_state: ProgramDisplayState,
}

impl Tui {
    /// Creates a new Tui and shows it.
    pub fn new(args: &InteractiveArgs) -> Self {
        let machine = MachineState::new(&args.init);
        let events = Events::new();
        let input_field = InputState::new();
        let keybinding_state = KeybindingHelpState::init();
        let program_display_state = ProgramDisplayState::empty();
        Tui {
            program_display_state,
            keybinding_state,
            machine,
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
                let area = f.size();
                f.render_stateful_widget(Interface, area, &mut self);
            })?;
            last_draw = Instant::now();
            // Loop until the next draw is necessary
            while last_draw.elapsed() < DURATION_BETWEEN_FRAMES {
                // Let the machine do some work
                if self.machine.auto_run_mode {
                    self.machine.trigger_key_clock();
                }
                // Update interface state
                self.maintain();
                // Handle event
                if self.handle_event() {
                    trace!("Quitting application");
                    // Quit
                    break 'outer;
                }
                if self.machine.auto_run_mode {
                    thread::sleep(ONE_MILLISECOND);
                }
            }
        }
        backend.clear()?;
        backend.show_cursor()?;
        Ok(())
    }
    /// Get a reference to the underlying machine.
    pub const fn machine(&self) -> &MachineState {
        &self.machine
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
                        self.machine.toggle_auto_run_mode();
                        false
                    }
                    Char('w') => {
                        self.machine.toggle_step_mode();
                        false
                    }
                    Char('e') => {
                        self.machine.trigger_key_interrupt();
                        self.keybinding_state.int_pressed();
                        false
                    }
                    Char('r') => {
                        self.machine.cpu_reset();
                        self.keybinding_state.reset_pressed();
                        false
                    }
                    Char('l') => {
                        self.machine.trigger_key_continue();
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
                            self.machine.trigger_key_clock();
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
        let last_cmd = self.input_field.last_cmd();
        if let Some(cmd) = last_cmd {
            trace!("Command entered: {:?}", cmd);
            match cmd {
                Command::LoadProgram(path) => {
                    let path = path.to_owned();
                    match self.load_program(path) {
                        Ok(()) => {}
                        Err(e) => warn!("Failed to run program: {}", e),
                    }
                }
                Command::SetInputReg(InputRegister::FC, val) => self.machine.set_input_fc(val),
                Command::SetInputReg(InputRegister::FD, val) => self.machine.set_input_fd(val),
                Command::SetInputReg(InputRegister::FE, val) => self.machine.set_input_fe(val),
                Command::SetInputReg(InputRegister::FF, val) => self.machine.set_input_ff(val),
                Command::SetIRG(val) => self.machine.set_digital_input1(val),
                Command::SetTEMP(val) => self.machine.set_temp(val),
                Command::SetI1(val) => self.machine.set_analog_input1(val),
                Command::SetI2(val) => self.machine.set_analog_input2(val),
                Command::SetJ1(val) => self.machine.set_jumper1(val),
                Command::SetJ2(val) => self.machine.set_jumper2(val),
                Command::SetUIO1(val) => self.machine.set_universal_input_output1(val),
                Command::SetUIO2(val) => self.machine.set_universal_input_output2(val),
                Command::SetUIO3(val) => self.machine.set_universal_input_output3(val),
                Command::Show(part) => self.machine.show(part),
                Command::Next(cycles) => {
                    for _ in 0..cycles {
                        self.machine.trigger_key_clock()
                    }
                }
                Command::Quit => return true,
            }
        } else {
            warn!("Invalid input: {:?}", self.input_field.last());
        }
        false
    }
    fn maintain(&mut self) {
        // Update keybinding state to reflect machine state
        let continue_possible = self.machine.state() == State::Stopped;
        self.keybinding_state
            .set_continue_possible(continue_possible);
        let edge_int_possible = self.machine.bus().is_key_edge_int_enabled();
        self.keybinding_state
            .set_edge_int_possible(edge_int_possible);
        let asm_step_on = self.machine.step_mode() == StepMode::Assembly;
        self.keybinding_state.set_asm_step_on(asm_step_on);
        let autorun_on = self.machine.auto_run_mode;
        self.keybinding_state.set_autorun_on(autorun_on);
    }
    pub fn load_program<P: Into<PathBuf>>(&mut self, path: P) -> Result<(), Error> {
        let path = path.into();
        let program = helpers::read_asm_file(&path)?;
        let bytecode = Translator::compile(&program);
        // Update the program display state
        self.program_display_state = ProgramDisplayState::from_bytecode(&bytecode);
        // Load the program into the machine
        self.machine.load_program(path, bytecode);
        Ok(())
    }
}
