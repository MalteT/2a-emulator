use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::Widget;
use tui::Terminal;

use std::cell::RefCell;
use std::io;
use std::io::Error as IOError;
use std::rc::Rc;
use std::thread;
use std::time::{Duration, Instant};

pub mod events;

use crate::schematic::{Machine, Signal};
use events::{Event, Events};
use node::Display;
use node::Node;

struct NodeWidget<N>(Rc<RefCell<N>>)
where
    N: Node + Display;

pub fn run() -> Result<(), IOError> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;
    terminal.hide_cursor()?;

    let events = Events::new();
    let mut events = events.try_iter();
    let mut frame: usize = 0;
    let mut frame_invalid = false;
    let mut auto_run = false;
    let mut last_event = None;

    let mut machine = Machine::compose();

    loop {
        if let Some(event) = events.next() {
            match event {
                Event::Quit => break,
                Event::Clock => {
                    machine.clk(Signal::Rising);
                    frame_invalid = true;
                    last_event = Some(Instant::now());
                }
                Event::Step => frame += 1,
                Event::ToggleAutoRun => auto_run = !auto_run,
                Event::Interrupt => {
                    //interrupt.send(true).expect("Send interrupt failed");
                    frame_invalid = true;
                    last_event = Some(Instant::now());
                }
                Event::Reset => {
                    //reset.send(true).expect("Send reset failed");
                    frame_invalid = true;
                    last_event = Some(Instant::now());
                }
                Event::Other(_) => {}
            }
            eprintln!("{:?}", event);
        }

        if let Some(ref inst) = last_event {
            if inst.elapsed().as_millis() > 300 {
                // reset.send(false).expect("Send reset failed");
                // interrupt.send(false).expect("Send interrupt failed");
                machine.clk(Signal::Falling);

                frame_invalid = true;
                last_event = None;
            }
        }

        if frame_invalid {
            frame_invalid = false;
            frame += 1;
        }

        if auto_run {
            //clk.send(true).expect("Send clk failed");
            frame += 1;
        }

        terminal.draw(|mut f| {
            let size = f.size().inner(1);
            machine.render(&mut f, size);
        })?;
        thread::sleep(Duration::from_millis(10));
    }

    terminal.clear()?;
    Ok(())
}

// impl<'node> Widget for Machine<'node>
// {
//
//     fn draw(&mut self, area: Rect, buf: &mut Buffer) {
//         let mut x = area.x;
//         let mut y = area.y;
//         self.al1
//             .borrow()
//             .to_utf8_string()
//             .lines()
//             .take(area.height as usize)
//             .for_each(|line| {
//                 x = area.x;
//                 line.char_indices()
//                     .take(area.width as usize)
//                     .for_each(|(_, c)| {
//                         let style = match c {
//                             '○' => Style::default().fg(Color::Gray),
//                             '●' => Style::default().fg(Color::Yellow),
//                             _ => Style::default(),
//                         };
//                         buf.set_string(x, y, c.to_string(), style);
//                         x += 1;
//                     });
//                 y += 1;
//             });
//     }
// }

impl<N> Widget for NodeWidget<N>
where
    N: Node + Display,
{
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let mut x = area.x;
        let mut y = area.y;
        self.0
            .borrow()
            .to_utf8_string()
            .lines()
            .take(area.height as usize)
            .for_each(|line| {
                x = area.x;
                line.char_indices()
                    .take(area.width as usize)
                    .for_each(|(_, c)| {
                        let style = match c {
                            '○' => Style::default().fg(Color::Gray),
                            '●' => Style::default().fg(Color::Yellow),
                            _ => Style::default(),
                        };
                        buf.set_string(x, y, c.to_string(), style);
                        x += 1;
                    });
                y += 1;
            });
    }
}

impl<And4, Or2, DFlipFlop, DFlipFlopC> Widget for InterruptLogic<And4, Or2, DFlipFlop, DFlipFlopC>
where
    And4: Display + Node,
    Or2: Display + Node,
    DFlipFlop: Display + Node,
    DFlipFlopC: Display + Node,
{
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let il1_rect = Rect::new(area.x + 2, area.y + 1, 11, 7);
        let il2_rect = Rect::new(area.x + 22, area.y + 7, 7, 6);
        let iff1_rect = Rect::new(area.x + 30, area.y + 1, 9, 8);
        let iff2_rect = Rect::new(area.x + 10, area.y + 3, 9, 7);
        let mut il1_buf = Buffer::empty(il1_rect);
        let mut il2_buf = Buffer::empty(il2_rect);
        let mut iff1_buf = Buffer::empty(iff1_rect);
        let mut iff2_buf = Buffer::empty(iff2_rect);
        let mut il1: NodeWidget<_> = self.il1.clone().into();
        let mut il2: NodeWidget<_> = self.il2.clone().into();
        let mut iff1: NodeWidget<_> = self.iff1.clone().into();
        let mut iff2: NodeWidget<_> = self.iff2.clone().into();
        il1.draw(il1_rect, &mut il1_buf);
        il2.draw(il2_rect, &mut il2_buf);
        iff1.draw(iff1_rect, &mut iff1_buf);
        iff2.draw(iff2_rect, &mut iff2_buf);
        let x = area.x;
        let mut y = area.y;
        include_str!("../../intlogic.utf8")
            .lines()
            .take(area.height as usize)
            .for_each(|line| {
                buf.set_stringn(x, y, line, area.width as usize, Style::default());
                y += 1;
            });
        buf.merge(&il1_buf);
        buf.merge(&il2_buf);
        buf.merge(&iff1_buf);
        buf.merge(&iff2_buf);
    }
}

impl<N> From<Rc<RefCell<N>>> for NodeWidget<N>
where
    N: Node + Display,
{
    fn from(node: Rc<RefCell<N>>) -> Self {
        NodeWidget(node)
    }
}

struct InterruptLogic<And4, Or2, DFlipFlop, DFlipFlopC>
where
    And4: Node + Display,
    Or2: Node + Display,
    DFlipFlop: Node + Display,
    DFlipFlopC: Node + Display,
{
    il1: Rc<RefCell<And4>>,
    il2: Rc<RefCell<Or2>>,
    iff1: Rc<RefCell<DFlipFlopC>>,
    iff2: Rc<RefCell<DFlipFlop>>,
}
