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
mod notification;
mod program_help_sidebar;
pub mod show_widgets;
mod supervisor_wrapper;
mod wrapper_widgets;

use crate::{
    args::InteractiveArgs,
    error::Error,
    helpers::{self, dur_sub},
};
pub use board_info_sidebar::BoardInfoSidebarWidget;
use events::Events;
use input::{Command, InputRegister, InputState};
use interface::Interface;
pub use notification::{NotificationState, NotificationWidget};
pub use program_help_sidebar::{KeybindingHelpState, ProgramDisplayState, ProgramHelpSidebar};
pub use supervisor_wrapper::{MachineState, MachineWidget, Part};

pub type Backend = CrosstermBackend<Stdout>;
type AbortEmulation = bool;

const FRAMES_PER_SECOND: u64 = 24;
const CYCLES_PER_SECOND: u64 = 7_372_800;
const DURATION_BETWEEN_FRAMES: Duration = Duration::from_micros(1_000_000 / FRAMES_PER_SECOND);

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
    /// Measured frequency derived in the main loop
    measured_freq: f32,
    /// State for the notification area.
    notification_state: NotificationState,
}

impl Tui {
    /// Creates a new Tui and shows it.
    pub fn new(args: &InteractiveArgs) -> Result<Self, Error> {
        let (machine, program_display_state) = if let Some(path) = args.program.as_ref() {
            let program = helpers::read_asm_file(&path)?;
            let bytecode = Translator::compile(&program);
            (
                MachineState::new_with_program(&args.init, path, bytecode.clone()),
                ProgramDisplayState::from_bytecode(&bytecode),
            )
        } else {
            (MachineState::new(&args.init), ProgramDisplayState::empty())
        };
        let events = Events::new();
        let input_field = InputState::new();
        let keybinding_state = KeybindingHelpState::init();
        let measured_freq = 0.0;
        let notification_state = NotificationState::empty();
        Ok(Tui {
            machine,
            events,
            input_field,
            keybinding_state,
            program_display_state,
            measured_freq,
            notification_state,
        })
    }
    /// Create a new TUI from the given command line arguments
    /// and start it immidiately.
    pub fn run_with_args(args: &InteractiveArgs) -> Result<(), Error> {
        Tui::new(args)?.run()
    }
    /// Run the main loop.
    pub fn run(mut self) -> Result<(), Error> {
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
        // Prepare for main loop
        let mut last_draw;
        // Loop until exit is requested
        loop {
            let mut executed_cycles = 0;
            last_draw = Instant::now();
            // Update interface state
            self.maintain();
            // Handle one event and exit if necessary
            if self.handle_event() {
                trace!("Quitting application");
                // Quit
                break;
            }
            // Next draw of the machine
            backend.draw(|mut f| {
                let area = f.size();
                f.render_stateful_widget(Interface, area, &mut self);
            })?;
            // Wait or calculate, depending on auto_run_mode
            if self.machine.auto_run_mode {
                // Do some calculations between frames
                while last_draw.elapsed() < DURATION_BETWEEN_FRAMES
                    && executed_cycles < CYCLES_PER_SECOND / FRAMES_PER_SECOND
                {
                    // Let the machine do some work
                    self.machine.trigger_key_clock();
                    executed_cycles += 1;
                }
                thread::sleep(dur_sub(DURATION_BETWEEN_FRAMES, last_draw.elapsed()));
            } else if last_draw.elapsed() < DURATION_BETWEEN_FRAMES {
                thread::sleep(DURATION_BETWEEN_FRAMES - last_draw.elapsed());
            }
            self.measured_freq =
                1e6 * executed_cycles as f32 / last_draw.elapsed().as_micros() as f32;
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
            // If some notification exist, clear that
            if !self.notification_state.is_empty() {
                self.notification_state.clear();
                return false;
            }
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
                        Err(e) => self.warn_about_failed_load(e),
                    }
                }
                Command::SetInputReg(InputRegister::Fc, val) => self.machine.set_input_fc(val),
                Command::SetInputReg(InputRegister::Fd, val) => self.machine.set_input_fd(val),
                Command::SetInputReg(InputRegister::Fe, val) => self.machine.set_input_fe(val),
                Command::SetInputReg(InputRegister::Ff, val) => self.machine.set_input_ff(val),
                Command::SetIrg(val) => self.machine.set_digital_input1(val),
                Command::SetTemp(val) => self.machine.set_temp(val),
                Command::SetI1(val) => self.machine.set_analog_input1(val),
                Command::SetI2(val) => self.machine.set_analog_input2(val),
                Command::SetJ1(val) => self.machine.set_jumper1(val),
                Command::SetJ2(val) => self.machine.set_jumper2(val),
                Command::SetUio1(val) => self.machine.set_universal_input_output1(val),
                Command::SetUio2(val) => self.machine.set_universal_input_output2(val),
                Command::SetUio3(val) => self.machine.set_universal_input_output3(val),
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
            self.notification_state.current = self
                .input_field
                .last()
                .map(|text| format!("Invalid input:\n> {}", text));
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
    fn warn_about_failed_load(&mut self, error: Error) {
        warn!("Failed to run program: {}", error);
        let warning = format!("Failed to load program:\n\n{}", error);
        self.notification_state.current = Some(warning);
    }
}
