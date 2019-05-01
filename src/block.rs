use std::cell::RefCell;
use std::rc::Rc;

#[macro_use]
mod block_macro;
mod cache;

pub trait Node
where
    Self::Output: Clone,
{
    type Output;
    unsafe fn get(&self, index: usize, clock: usize) -> Self::Output;
}

#[derive(Clone)]
pub struct Wire<'a, O>
where
    O: Clone,
{
    node: Rc<RefCell<dyn Node<Output = O> + 'a>>,
    index: usize,
}

impl<'a, O> Wire<'a, O>
where
    O: Clone,
{
    pub fn get(&mut self, clock: usize) -> O {
        unsafe { self.node.borrow().get(self.index, clock) }
    }
}

define_node! {
    pub Input {
        1 {
            out_q -> 0,
        }
    }
}

define_node! {
    pub Node2x1 {
        I1: in1 -> 0,
        I2: in2 -> 1;
        1 {
            out -> 0
        }
    }
}

define_node! {
    pub DFlipFlop {
        I1: input -> 0,
        I2: clk -> 1;
        1 {
            out_q -> 0,
        }
    }
}
