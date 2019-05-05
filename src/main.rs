use pretty_env_logger;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::style::Color;
use tui::widgets::canvas::Canvas;
use tui::widgets::Widget;
use tui::Terminal;

use std::fmt;
use std::io;
use std::ops;

pub mod fns;
pub mod node;
pub mod schematic;

use crate::node::channel;
use crate::node::DFlipFlop;
use crate::node::DFlipFlopC;
use crate::node::Input;
use crate::node::Node2x1;
use crate::node::Node4x1;
use crate::node::Test;
use crate::node::Inp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Signal {
    High,
    Low,
}

use Signal::*;

fn main() -> Result<(), io::Error> {
    pretty_env_logger::init();

    let mac1 = High;
    let mac0 = High;
    let na0 = High;

    let (_, mac1) = Input::new("MAC1", || mac1);
    let (_, mac0) = Input::new("MAC0", || mac0);
    let (_, na0) = Input::new("NA0", || na0);
    let (reset_send, reset) = channel("reset");
    let (int_send, int) = channel("INTERRUPT");
    let (clk_send, clk) = channel("CLK");
    let (_, high) = Input::new("HIGH", || High);
    let (il1, il1_out) = Node4x1::new("IL1", |&x, &y, &z, &a| x & y & z & a);
    let (il2, il2_out) = Node2x1::new("IL2", |&x, &y| x | y);

    let mut last_value = Default::default();
    let (iff1, mut iff1_out) = DFlipFlopC::new("IFF1", move |&d, &clk, &clear: &Signal| {
        if clk & !clear == High {
            last_value = d;
        } else if clear == High {
            last_value = Low;
        }
        last_value
    });

    let mut last_value = Default::default();
    let (iff2, iff2_out) = DFlipFlop::new("IFF2", move |&d, &clk| {
        if clk == High {
            last_value = d
        };
        last_value
    });

    il1.borrow_mut()
        .plug_in0(iff1_out.clone())
        .plug_in1(mac1)
        .plug_in2(mac0)
        .plug_in3(na0);
    il2.borrow_mut().plug_in0(iff2_out).plug_in1(reset);
    iff1.borrow_mut()
        .plug_input(high)
        .plug_clk(int)
        .plug_clear(il2_out);
    iff2.borrow_mut().plug_input(il1_out).plug_clk(clk);

    for cache in 1..10 {
        let int_raw: Signal = (cache == 0 || cache == 1).into();
        let clk_raw: Signal = (cache % 2 == 1).into();
        clk_send.send(clk_raw).unwrap();
        int_send.send(int_raw).unwrap();
        reset_send.send((cache == 7).into()).unwrap();
        let _ = iff1_out.get(cache);
        println!("{}", il1.borrow());
        println!("{}", iff2.borrow());
    }

    //let (test, test_out)   = Inp::new("test", || true);
    //let (test1, test1_out) = Test::new("test", |y: bool| !y);
    let (test2, mut test2_out) = Test::new("test", |y: bool| ! y);

    test2.borrow_mut().plug_in1(test2_out.clone());
    //test1.borrow_mut().plug_in1(test2_out.clone());

    for x in 0..10 {
        println!("{}", test2_out.get(x));
    }
    println!("{:#?}", test2);

    // let stdout = io::stdout().into_raw_mode()?;
    // let backend = TermionBackend::new(stdout);
    // let mut terminal = Terminal::new(backend)?;

    // terminal.draw(|mut f| {
    //     let size = f.size();
    //     Canvas::default()
    //         .paint(|ctx| {
    //             ctx.print(0.0, 0.0, "test", Color::Red);
    //         })
    //         .render(&mut f, size);
    // })?;

    Ok(())
}

impl Default for Signal {
    fn default() -> Self {
        Signal::Low
    }
}

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Signal::Low => "○",
                Signal::High => "●",
            }
        )
    }
}

impl ops::BitAnd for Signal {
    type Output = Self;

    fn bitand(self, other: Self) -> Self::Output {
        use Signal::*;
        match (self, other) {
            (High, High) => High,
            _ => Low,
        }
    }
}

impl ops::BitOr for Signal {
    type Output = Self;

    fn bitor(self, other: Self) -> Self::Output {
        use Signal::*;
        match (self, other) {
            (Low, Low) => Low,
            _ => High,
        }
    }
}

impl ops::Not for Signal {
    type Output = Self;

    fn not(self) -> Self::Output {
        use Signal::*;
        match self {
            High => Low,
            Low => High,
        }
    }
}

impl From<bool> for Signal {
    fn from(b: bool) -> Self {
        use Signal::*;
        if b {
            High
        } else {
            Low
        }
    }
}

impl From<Signal> for bool {
    fn from(s: Signal) -> Self {
        use Signal::*;
        match s {
            High => true,
            Low => false,
        }
    }
}
