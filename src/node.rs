use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::sync::mpsc::{channel as mpsc_channel, Receiver, Sender};

#[macro_use]
mod node_macro;
mod cache;

pub use cache::Cache;

pub trait Node {
    type Output;
    unsafe fn get(&self, index: usize, clock: usize) -> Self::Output;
}

#[derive(Clone)]
pub struct Wire<'a, O> {
    node: Rc<RefCell<dyn Node<Output = O> + 'a>>,
    index: usize,
}

impl<'a, O> Wire<'a, O> {
    pub fn get(&mut self, clock: usize) -> O {
        unsafe { self.node.borrow().get(self.index, clock) }
    }
}

define_node! {
    pub Input {
        display {
            "TODO {}",
        },
    }
}

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

define_node! {
    pub Node2x1 {
        inputs {
            I1: in0 -> 0,
            I2: in1 -> 1,
        },
        display {
            "TODO {}",
        },
    }
}

define_node! {
    pub Node4x1 {
        inputs {
            I1: in0 -> 0,
            I2: in1 -> 1,
            I3: in2 -> 2,
            I4: in3 -> 3,
        },
        display {
            include_str!("../and4.utf8"), in0, in1, in2, in3,
        }
    }
}

define_node! {
    pub DFlipFlop {
        inputs {
            I1: input -> 0,
            I2: clk -> 1,
        },
        display {
            include_str!("../test.ascii"), input, clk,
        }
    }
}

define_node! {
    pub DFlipFlopC {
        inputs {
            I1: input -> 0,
            I2: clk -> 1,
            I3: clear -> 2,
        },
        display {
            "TODO {}",
        }
    }
}

define_node! {
    pub Mux2x1 {
        inputs {
            I1: in0 -> 0,
            I2: in1 -> 1,
            I3: select -> 2,
        },
        display {
            "TODO {}",
        }
    }
}

define_node! {
    pub Mux2x2 {
        inputs {
            I1: in0 -> 0,
            I2: in1 -> 1,
            I3: select -> 2,
        },
        outputs 2 {
            out -> 0,
            test -> 1,
        },
        display {
            "TODO {}",
        }
    }
}

define_node! {
    pub Mux4x1 {
        inputs {
            I1: in0 -> 0,
            I2: in1 -> 1,
            I3: in2 -> 2,
            I4: in3 -> 3,
            I5: select0 -> 4,
            I6: select1 -> 5,
        },
        display {
            "TODO {}",
        },
    }
}

define_node! {
    pub Mux8x1 {
        inputs {
            I0: in0 -> 0,
            I1: in1 -> 1,
            I2: in2 -> 2,
            I3: in3 -> 3,
            I4: in4 -> 4,
            I5: in5 -> 5,
            I6: in6 -> 6,
            I7: in7 -> 7,
            I8: select0 -> 8,
            I9: select1 -> 9,
            I10: select2 -> 10,
        },
        display {
            "TODO {}",
        },
    }
}
