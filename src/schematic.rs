use node::node;
use node::Cache;
use node::Display;
use node::Node;
use node::Wire;

use std::cell::RefCell;
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::mpsc::{channel as mpsc_channel, Receiver, Sender};

pub fn channel<'a, O>(id: &'static str) -> (Sender<O>, Wire<'a, O>)
where
    O: Clone + fmt::Debug + Default + 'a,
{
    let (sender, receiver): (Sender<O>, Receiver<O>) = mpsc_channel();
    let mut last = Default::default();
    let f = move || {
        while let Ok(value) = receiver.try_recv() {
            last = value;
        }
        last.clone()
    };
    let (_, out) = Input::new(id, f);
    (sender, out)
}

pub type Or2<'a, F, I1, I2, O> = Node2x1<'a, F, I1, I2, O>;
pub type Xor<'a, F, I1, I2, O> = Node2x1<'a, F, I1, I2, O>;
pub type And2<'a, F, I1, I2, O> = Node2x1<'a, F, I1, I2, O>;
pub type And4<'a, F, I1, I2, I3, I4, O> = Node4x1<'a, F, I1, I2, I3, I4, O>;

#[node {
    ascii("TODO {}", id),
    utf8("TODO {}", id),
    add_new(id),
}]
pub struct Input {
    pub id: &'static str,
    out: Output,
}

#[node {
    ascii("TODO {}", id),
    utf8("TODO {}", id),
    add_new(id),
}]
pub struct Node2x1 {
    pub id: &'static str,
    in0: Input,
    in1: Input,
    out: Output,
}

#[node {
    ascii("TODO {}", id),
    utf8(file("../displays/and4.utf8"), id, in0, in1, in2, in3),
    add_new(id),
}]
pub struct Node4x1 {
    pub id: &'static str,
    in0: Input,
    in1: Input,
    in2: Input,
    in3: Input,
    out: Output,
}

#[node {
    ascii("TODO {}", id),
    utf8(file("../displays/dflipflop.utf8"), id, input, clk, out),
    add_new(id),
}]
pub struct DFlipFlop {
    pub id: &'static str,
    input: Input,
    clk: Input,
    out: Output,
}

#[node {
    ascii("TODO {}", id),
    utf8("TODO {}", id),
    add_new(id),
}]
pub struct DFlipFlopC {
    pub id: &'static str,
    input: Input,
    clk: Input,
    clear: Input,
    out: Output,
}

#[node {
    ascii("TODO {}", id),
    utf8("TODO {}", id),
    add_new(id),
}]
pub struct Mux2x1 {
    pub id: &'static str,
    in0: Input,
    in1: Input,
    select: Input,
    out: Output,
}

#[node {
    ascii("TODO {}", id),
    utf8("TODO {}", id),
    add_new(id),
}]
pub struct Mux4x1 {
    pub id: &'static str,
    in0: Input,
    in1: Input,
    in2: Input,
    in3: Input,
    select0: Input,
    select1: Input,
    out: Output,
}

#[node {
    ascii("TODO {}", id),
    utf8("TODO {}", id),
    add_new(id),
}]
pub struct Mux8x1 {
    pub id: &'static str,
    in0: Input,
    in1: Input,
    in2: Input,
    in3: Input,
    in4: Input,
    in5: Input,
    in6: Input,
    in7: Input,
    select0: Input,
    select1: Input,
    select2: Input,
    select3: Input,
    out: Output,
}
