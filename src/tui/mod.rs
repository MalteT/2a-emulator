/// Terminal User Interface.
use lazy_static::lazy_static;
use log::trace;
use mr2a_asm_parser::asm::Asm;
use tui::backend::CrosstermBackend;
use tui::layout::Rect;
use tui::widgets::{Block, Borders, Widget};
use tui::Terminal;

use std::io::Error as IOError;
use std::ops::Deref;
use std::thread;
use std::time::{Duration, Instant};

pub mod display;
pub mod events;
pub mod grid;
pub mod input;

use crate::schematic::Machine;
use events::{Event, Events};
use input::Input;

lazy_static! {
    static ref DURATION_BETWEEN_FRAMES: Duration = Duration::from_micros(16_666);
    static ref ONE_MICROSECOND: Duration = Duration::from_micros(1);
    static ref ONE_MILLISECOND: Duration = Duration::from_millis(1);
    static ref DEFAULT_CLK_PERIOD: Duration = Duration::from_nanos((1_000.0 / 7.3728) as u64);
}

type Backend = Terminal<CrosstermBackend>;

pub struct Tui {
    /// The actual minirechner.
    machine: Machine,
    /// Event iterator.
    events: Events,
    /// Auto run mode for emulation.
    clk_auto_run_mode: bool,
    /// The input field at the bottom of the TUI.
    input_field: Input,
    time_since_last_clk: Instant,
    time_since_last_draw: Instant,
    /// Time between two clock rising edges.
    clk_period: Duration,
    is_main_loop_running: bool,
}

impl Tui {
    /// Creates a new Tui and shows it.
    pub fn new() -> Result<Self, IOError> {
        let events = Events::new();
        // Return the Tui object
        let machine = Machine::new();
        let clk_auto_run_mode = false;
        let input_field = Input::new();
        let time_since_last_clk = Instant::now();
        let time_since_last_draw = Instant::now();
        let clk_period = *DEFAULT_CLK_PERIOD.deref();
        let is_main_loop_running = false;
        Ok(Tui {
            machine,
            events,
            clk_auto_run_mode,
            input_field,
            time_since_last_clk,
            time_since_last_draw,
            clk_period,
            is_main_loop_running,
        })
    }
    /// Run the main loop using the optional asm program.
    pub fn run(mut self, program: Option<Asm>) -> Result<(), IOError> {
        // Initialize backend.
        let mut backend = Terminal::new(init_backend()?)?;
        // Clear the terminal and hide the cursor
        backend.clear()?;
        backend.hide_cursor()?;
        // Run program if given.
        if let Some(ref program) = program {
            self.machine.run(program);
        }
        self.is_main_loop_running = true;
        while self.is_main_loop_running {
            let now = Instant::now();
            // Handle event
            self.handle_event();
            // Next clock for the machine.
            if self.clk_auto_run_mode && (now - self.time_since_last_clk) >= self.clk_period {
                self.time_since_last_clk = now;
                self.machine.clk();
            }
            // Next draw of the machine
            if now - self.time_since_last_draw >= *DURATION_BETWEEN_FRAMES.deref() {
                self.time_since_last_draw = now;
                self.draw_machine(&mut backend)?;
            }
            thread::sleep(*ONE_MICROSECOND.deref());
        }
        backend.clear()?;
        Ok(())
    }
    /// Create the user interface.
    fn dummy() {}
    /// Handle one single event in the queue.
    fn handle_event(&mut self) {
        if let Some(event) = self.events.next() {
            match event {
                Event::Quit => self.is_main_loop_running = false,
                Event::Clock => {
                    // Only interpret Enter as CLK if no text was input
                    if self.input_field.is_empty() {
                        if !self.clk_auto_run_mode {
                            self.machine.clk();
                        }
                    } else {
                        self.input_field.handle(Event::Char('\n'));
                        let query = self.input_field.pop();
                        self.handle_input(&query);
                        trace!("Command entered: {}", query);
                    }
                }
                Event::Step => {}
                Event::ToggleAutoRun => self.clk_auto_run_mode = !self.clk_auto_run_mode,
                Event::Interrupt => {
                    self.machine.edge_int();
                }
                Event::Reset => {
                    self.machine.reset();
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
            // TODO: Load program
        } else if query == "quit" {
            self.is_main_loop_running = false;
        }
    }
    /// Draw the machines current state.
    fn draw_machine(&mut self, backend: &mut Backend) -> Result<(), IOError> {
        let mut outer_block = Block::default()
            .title("Minirechner 2a")
            .borders(Borders::ALL);
        backend.draw(|mut f| {
            let mut area = f.size();
            area.height -= 3;
            // machine
            outer_block.render(&mut f, area);
            let inner_area = outer_block.inner(area).inner(1);
            self.machine.render(&mut f, inner_area);
            // input
            let area = Rect::new(area.x, area.y + area.height, area.width, 3);
            self.input_field.render(&mut f, area.inner(1));
        })
    }
}

fn init_backend() -> Result<CrosstermBackend, IOError> {
    use crossterm::{AlternateScreen, TerminalOutput};
    let stdout = TerminalOutput::new(true);
    let screen = AlternateScreen::to_alternate_screen(stdout, true)?;
    CrosstermBackend::with_alternate_screen(screen)
}
