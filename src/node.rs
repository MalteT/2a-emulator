use ::node::node;
use std::cell::RefCell;
use std::fmt;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::mpsc::{channel as mpsc_channel, Receiver, Sender};

mod cache;

pub use cache::Cache;

pub trait Node {
    fn update(&mut self, cache_id: usize);
}

#[derive(Clone)]
pub struct Wire<'a, O>
where
    O: Clone,
{
    node: Rc<RefCell<dyn Node + 'a>>,
    last_output: Rc<RefCell<Cache<O>>>,
}

impl<'a, O> fmt::Debug for Wire<'a, O>
where
    O: Clone + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Wire")
            .field("last_output", &self.last_output)
            .field("node", &"[hidden]".to_string())
            .finish()
    }
}

impl<'a, O> Wire<'a, O>
where
    O: Clone + Default,
{
    pub fn get(&mut self, cache_id: usize) -> O {
        if self
            .last_output
            .try_borrow()
            .expect("Ref borrow of last_output failed in Wire")
            .is_valid(cache_id)
        {
            self.last_output.borrow().clone().unwrap()
        } else {
            let node = self.node.try_borrow_mut();
            match node {
                Ok(mut node) => {
                    node.update(cache_id);
                    self.last_output.borrow().clone().unwrap()
                }
                Err(_) => self.last_output.borrow().clone().unwrap_or_default(),
            }
        }
    }
}

pub trait Display {
    fn to_ascii_string(&self) -> String;
    fn to_utf8_string(&self) -> String;
}

impl Display for String {
    fn to_ascii_string(&self) -> String {
        self.clone()
    }
    fn to_utf8_string(&self) -> String {
        self.clone()
    }
}

impl<T> Display for ::std::cell::Ref<'_, T>
where
    T: Display,
{
    fn to_ascii_string(&self) -> String {
        Display::to_ascii_string(Deref::deref(self))
    }
    fn to_utf8_string(&self) -> String {
        Display::to_utf8_string(Deref::deref(self))
    }
}

impl Display for bool {
    fn to_ascii_string(&self) -> String {
        match self {
            false => "0",
            true => "1",
        }
        .into()
    }
    fn to_utf8_string(&self) -> String {
        match self {
            false => "○",
            true => "●",
        }
        .into()
    }
}

impl Display for u8 {
    fn to_ascii_string(&self) -> String {
        format!("{:>08b}", self)
    }
    fn to_utf8_string(&self) -> String {
        Display::to_ascii_string(self)
            .replace("0", "○")
            .replace("1", "●")
    }
}

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
