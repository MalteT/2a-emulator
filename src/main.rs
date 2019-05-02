use pretty_env_logger;

use std::fmt::Debug;
use std::sync::mpsc::{channel, Sender};

mod fns;
mod node;

use node::DFlipFlop;
use node::DFlipFlopC;
use node::Input;
use node::Node2x1;
use node::Node4x1;
use node::Wire;

fn input_channel<'a, O>(id: &str) -> (Sender<O>, Wire<'a, O>)
where
    O: Clone + Debug + Default + 'a,
{
    let (sender, receiver) = channel();
    let mut last = Default::default();
    (
        sender,
        Input::new(id, move || {
            while let Ok(value) = receiver.try_recv() {
                last = value;
            }
            last.clone()
        })
        .1,
    )
}

fn main() {
    pretty_env_logger::init();

    let mac1 = true;
    let mac0 = true;
    let na0 = false;

    let (_, mac1) = Input::new("MAC1", || mac1);
    let (_, mac0) = Input::new("MAC0", || mac0);
    let (_, na0) = Input::new("NA0", || na0);
    let (reset_send, reset) = input_channel("reset");
    let (int_send, int) = input_channel("INTERRUPT");
    let (clk_send, clk) = input_channel("CLK");
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
        .plug_in1(iff1_out.clone())
        .plug_in2(mac1)
        .plug_in3(mac0)
        .plug_in4(na0);
    il2.borrow_mut().plug_in1(iff2_out).plug_in2(reset);
    iff1.borrow_mut()
        .plug_input(high)
        .plug_clk(int)
        .plug_clear(il2_out);
    iff2.borrow_mut().plug_input(il1_out).plug_clk(clk);

    for cache in 0..10 {
        clk_send.send(cache % 2 == 1).unwrap();
        int_send.send(cache == 0).unwrap();
        reset_send.send(cache == 7).unwrap();
        iff1_out.get(cache);
        //println!("{}", iff1_out.get(cache));
    }
}
