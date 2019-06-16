use log::trace;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Widget};
use tui::Terminal;

use std::io;
use std::io::Error as IOError;
use std::thread;
use std::time::{Duration, Instant};

pub mod events;
pub mod input;

use crate::schematic::Machine;
use events::{Event, Events};
use input::Input;

pub fn run() -> Result<(), IOError> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;
    terminal.hide_cursor()?;

    let events = Events::new();
    let mut events = events.try_iter();
    let mut auto_run = false;
    let mut last_event = None;

    let mut machine = Machine::compose();
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
                    eprintln!("{}", s);
                }
                Event::Step => {}
                Event::ToggleAutoRun => auto_run = !auto_run,
                Event::Interrupt => {
                    //interrupt.send(true).expect("Send interrupt failed");
                    last_event = Some(Instant::now());
                }
                Event::Reset => {
                    //reset.send(true).expect("Send reset failed");
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
                // reset.send(false).expect("Send reset failed");
                // interrupt.send(false).expect("Send interrupt failed");
                machine.clk(false);

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
        thread::sleep(Duration::from_millis(10));
    }

    terminal.clear()?;
    Ok(())
}
