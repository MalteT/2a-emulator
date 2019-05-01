use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;

#[macro_use]
mod block_macro;

pub use block_macro::DFlipFlop;

pub trait Node
where
    Self::Output: Clone,
{
    type Output;
    fn get(&self, index: usize, clock: usize) -> Self::Output;
}

pub trait Wire
where
    Self::Output: Clone,
{
    type Output;
    fn out(&self, clock: usize) -> Self::Output;
}

#[derive(Debug, Clone)]
pub struct Handle<T>
where
    T: Node,
{
    index: usize,
    block: Rc<RefCell<T>>,
}


impl<T> Wire for Handle<T>
where
    T: Node,
{
    type Output = T::Output;

    fn out(&self, clock: usize) -> Self::Output {
        self.block.borrow().get(self.index, clock)
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
