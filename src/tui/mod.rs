use log::trace;
use mr2a_asm_parser::asm::Asm;
use tui::backend::CrosstermBackend;
use tui::layout::Rect;
use tui::widgets::{Block, Borders, Widget};
use tui::Terminal;

use std::io::Error as IOError;
use std::thread;
use std::time::{Duration, Instant};

pub mod display;
pub mod events;
pub mod grid;
pub mod input;

use crate::schematic::{Machine, Part};
use events::{Event, Events};
use input::Input;

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

    loop {
        if let Some(event) = events.next() {
            match event {
                Event::Quit => break,
                Event::Clock => {
                    if input.is_empty() {
                        machine.clk(true);
                        last_event = Some(Instant::now());
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
                    last_event = Some(Instant::now());
                }
                Event::Reset => {
                    machine.reset(true);
                    last_event = Some(Instant::now());
                }
                Event::Backspace | Event::Char(_) => {
                    input.handle(event.clone());
                }
                x => unimplemented!("{:#?}", x),
            }
            trace!("{:?}", event);
        }

        if let Some(ref inst) = last_event {
            if inst.elapsed().as_millis() > 300 {
                machine.reset(false);

                last_event = None;
            }
        }

        if auto_run {
            machine.clk(true);
            last_event = Some(Instant::now());
        }

        let mut outer_block = Block::default()
            .title("Minirechner 2a")
            .borders(Borders::ALL);

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

    terminal.clear()?;
    Ok(())
}
