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

pub fn channel<'a, O>(id: &str) -> (Sender<O>, Wire<'a, O>)
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

node! {
    pub struct Input {
        inputs {},
        outputs { out },
        display {
            ascii { "TODO {}", id }
            utf8 { "TODO {}", id }
        },
    }
}

node! {
    pub struct Node2x1 {
        inputs { in0, in1, },
        outputs { out },
        display {
            ascii { "TODO {}", id },
            utf8 { "TODO {}", id },
        },
    }
}

node! {
    pub struct Node4x1 {
        inputs { in0, in1, in2, in3, },
        outputs { out }
        display {
            ascii { "TODO {}", id },
            utf8 { include_str!("../displays/and4.utf8"), id, in0, in1, in2, in3, },
        }
    }
}

node! {
    pub struct DFlipFlop {
        inputs {
            input,
            clk,
        },
        outputs { out },
        display {
            ascii { "TODO {}", id },
            utf8 { include_str!("../displays/dflipflop.utf8"), id, input, clk, out, }
        }
    }
}

node! {
    pub struct DFlipFlopC {
        inputs {
            input,
            clk,
            clear,
        },
        outputs { out }
        display {
            ascii { "TODO {}", id },
            utf8 { "TODO {}", id },
        },
    }
}

node! {
    pub struct Mux2x1 {
        inputs {
            in0,
            in1,
            select,
        },
        outputs { out },
        display {
            ascii { "TODO {}", id },
            utf8 { "TODO {}", id },
        },
    }
}

node! {
    pub struct Mux4x1 {
        inputs {
            in0,
            in1,
            in2,
            in3,
            select0,
            select1,
        },
        outputs {
            out,
        },
        display {
            ascii { "TODO {}", id },
            utf8 { "TODO {}", id },
        },
    }
}

node! {
    pub struct Mux8x1 {
        inputs {
            in0,
            in1,
            in2,
            in3,
            in4,
            in5,
            in6,
            in7,
            select0,
            select1,
            select2,
        },
        outputs {
            out,
        },
        display {
            ascii { "TODO {}", id },
            utf8 { "TODO {}", id },
        },
    }
}
