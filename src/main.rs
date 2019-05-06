use pretty_env_logger;
//use termion::raw::IntoRawMode;
//use tui::backend::TermionBackend;
//use tui::style::Color;
//use tui::widgets::canvas::Canvas;
//use tui::widgets::Widget;
//use tui::Terminal;

use std::io;

pub mod fns;
pub mod schematic;

use schematic::channel;
use schematic::DFlipFlop;
use schematic::DFlipFlopC;
use schematic::Input;
use schematic::Node2x1;
use schematic::Node4x1;

fn main() -> Result<(), io::Error> {
    pretty_env_logger::init();

    let mac1 = true;
    let mac0 = true;
    let na0 = true;

    let (_, mac1) = Input::new("MAC1", || mac1);
    let (_, mac0) = Input::new("MAC0", || mac0);
    let (_, na0) = Input::new("NA0", || na0);
    let (reset_send, reset) = channel("reset");
    let (int_send, int) = channel("INTERRUPT");
    let (clk_send, clk) = channel("CLK");
    let (_, high) = Input::new("HIGH", || true);
    let (il1, il1_out) = Node4x1::new("IL1", |&x, &y, &z, &a| x && y && z && a);
    let (il2, il2_out) = Node2x1::new("IL2", |&x, &y| x || y);

    let mut last_value = Default::default();
    let (iff1, mut iff1_out) = DFlipFlopC::new("IFF1", move |&d, &clk, &clear: &bool| {
        if clk && !clear {
            last_value = d;
        } else if clear {
            last_value = false;
        }
        last_value
    });

    let mut last_value = Default::default();
    let (iff2, iff2_out) = DFlipFlop::new("IFF2", move |&d, &clk| {
        if clk {
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
        let int_raw = (cache == 0 || cache == 1).into();
        let clk_raw = (cache % 2 == 1).into();
        clk_send.send(clk_raw).unwrap();
        int_send.send(int_raw).unwrap();
        reset_send.send((cache == 7).into()).unwrap();
        let _ = iff1_out.get(cache);
        println!("{}", il1.borrow());
        println!("{}", iff2.borrow());
    }

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
