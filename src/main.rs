mod block;
mod fns;

use block::DFlipFlop;
use block::Input;
use block::Node2x1;

fn main() {
    let (_, [in_true]) = Input::new("Always TRUE", || [true]);
    let (xor, [mut xor_out]) = Node2x1::new("IL2", |&x, &y| [(x || y) && (!x || !y)]);
    let (iff2, [iff2_q]) = DFlipFlop::new("IFF2", |&inp, &clk| [clk && inp]);

    xor.borrow_mut()
        .plug_in1(in_true.clone())
        .plug_in2(xor_out.clone());
    //iff2.borrow_mut().plug_input(xor_out.clone()).plug_clk(in_true);

    for id in 0..8 {
        println!("{}", xor_out.get(id));
    }
}
