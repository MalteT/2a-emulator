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

use crate::schematic::{Machine, Part};
use events::{Event, Events};
use input::Input;

lazy_static! {
    static ref DURATION_BETWEEN_FRAMES: Duration = Duration::from_micros(16_666);
    static ref ONE_MICROSECOND: Duration = Duration::from_micros(1);
    static ref ONE_MILLISECOND: Duration = Duration::from_millis(1);
}

fn init_backend() -> Result<CrosstermBackend, IOError> {
    use crossterm::{AlternateScreen, TerminalOutput};
    let stdout = TerminalOutput::new(true);
    let screen = AlternateScreen::to_alternate_screen(stdout, true)?;
    CrosstermBackend::with_alternate_screen(screen)
}

// #[cfg(not(windows))]
// fn init_backend() -> Result<TermionBackend<RawTerminal<Stdout>>, IOError> {
//     use termion::raw::IntoRawMode;
//     let stdout = io::stdout().into_raw_mode()?;
//     Ok(TermionBackend::new(stdout))
// }

pub fn run(program: Option<Asm>) -> Result<(), IOError> {
    let mut terminal = Terminal::new(init_backend()?)?;

    terminal.clear()?;
    terminal.hide_cursor()?;

    let mut events = Events::new();
    let events = events.iter();
    let mut auto_run = false;
    let mut last_event = None;

    let mut machine = Machine::new();
    if let Some(ref program) = program {
        machine.run(program);
    }
    let mut input = Input::new();
    // Time counter for keeping a max framerate of 60fps
    let mut time_since_last_draw = Instant::now();
    // Time counter for machine calculations
    let mut time_since_last_clk = Instant::now();
    // Frequency of the machine (Default: 7.3728 MHz)
    // TODO: Ability to change frequency
    let frequency = 7_372_8;
    // Inverse fo the frequency of the machine
    let clock_period = Duration::from_micros((1_000_000 / frequency) as u64);
    // Block for drawing
    let mut outer_block = Block::default()
        .title("Minirechner 2a")
        .borders(Borders::ALL);
    // MAIN LOOP
    loop {
        // Handle events
        if let Some(event) = events.next() {
            match event {
                Event::Quit => break,
                Event::Clock => {
                    if input.is_empty() {
                        // TODO: Prevent clk on auto-run
                        machine.clk();
                    } else {
                        input.handle(Event::Char('\n'));
                    }
                    let s = input.pop();
                    // TODO: Improve
                    if s.starts_with("show ") {
                        let s = &s[5..];
                        let part = match s {
                            "interrupt" => Part::InterruptLogic,
                            "il1" => Part::Il1,
                            "il2" => Part::Il2,
                            "iff1" => Part::Iff1,
                            "iff2" => Part::Iff2,
                            _ => Part::InterruptLogic,
                        };
                        machine.show(part);
                    }
                    trace!("{}", s);
                }
                Event::Step => {}
                Event::ToggleAutoRun => auto_run = !auto_run,
                Event::Interrupt => {
                    machine.edge_int();
                }
                Event::Reset => {
                    machine.reset();
                }
                Event::Backspace | Event::Char(_) => {
                    input.handle(event.clone());
                }
                x => unimplemented!("{:#?}", x),
            }
            trace!("{:?}", event);
        }
        // Auto-call clk on machine, if the machine should auto run
        // or sleep for a short amount of time between checking inputs
        // and redrawing
        if auto_run {
            if Instant::now() - time_since_last_clk > clock_period {
                time_since_last_clk = Instant::now();
                machine.clk();
            } else {
                // Sleep for the remaining time, minus one microsecond
                let diff = Instant::now() - time_since_last_clk;
                if let Some(dur) = diff.checked_sub(*ONE_MILLISECOND.deref()) {
                    thread::sleep(dur);
                }
            }
        } else {
            thread::sleep(*ONE_MILLISECOND.deref());
        }
        // DRAWING
        // If the time between frames passed, redraw the screen
        if Instant::now() - time_since_last_draw > *DURATION_BETWEEN_FRAMES.deref() {
            time_since_last_draw = Instant::now();
            terminal.draw(|mut f| {
                let mut area = f.size();
                area.height -= 3;
                // machine
                outer_block.render(&mut f, area);
                let inner_area = outer_block.inner(area).inner(1);
                machine.render(&mut f, inner_area);
                // input
                let area = Rect::new(area.x, area.y + area.height, area.width, 3);
                input.render(&mut f, area.inner(1));
            })?;
        }
    }
    // Clear the terminal on exit
    terminal.clear()?;
    Ok(())
}
