use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;

#[macro_use]
mod block_macro;

pub trait Node
where
    Self::Output: Clone,
{
    type Output;
    fn get(&mut self, index: usize, clock: usize) -> Self::Output;
}

pub trait Wire
where
    Self::Output: Clone,
{
    type Output;
    fn out(&mut self, clock: usize) -> Self::Output;
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

    fn out(&mut self, clock: usize) -> Self::Output {
        self.block.borrow_mut().get(self.index, clock)
    }
}

block! { pub Input;; 1 }
block! { pub Node2x1; in1: I1, in2: I2; 1 }
