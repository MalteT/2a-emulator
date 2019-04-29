mod block;
mod fns;

use block::Node2x1;
use block::Input;
use block::Wire;

fn main() {
    let (_, in1) = Input::new("Always TRUE", |c| {
        let now = c % 4 == 2 || c % 4 == 3;
        print!("CLOCK: {}, IN1 {},\t", c, now);
        now
    });
    let (_, in2) = Input::new("Always FALSE", |c| {
        let now = c % 2 == 1;
        print!("IN2 {},\t", now);
        now
    });
    let (_, and_out) = Node2x1::new("AND", in1, in2, fns::and);
    let (_, eq_1) = Node2x1::new("EQ", and_out.clone(), and_out.clone(), |_, &x, &y| {
        x == y
    });

    for clock in 0..4 {
        println!("{}", and_out.borrow_mut().out(clock));
    }
    for clock in 0..4 {
        println!("{}", eq_1.borrow_mut().out(clock));
    }
}
