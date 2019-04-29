use std::ops::DerefMut;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug)]
struct Block<F, F1, F2> {
    id: String,
    in1: Rc<RefCell<Input<F1>>>,
    in2: Rc<RefCell<Input<F2>>>,
    f: Box<F>,
}

#[derive(Debug, Clone)]
struct Input<F> {
    id: String,
    f: Box<F>,
}

fn main() {
    let in1 = Input::new("Always TRUE", || {
        println!("IN1: {}", true);
        true
    });
    let in2 = Input::new("Always FALSE", || {
        println!("IN1: {}", false);
        false
    });
    let mut and = Block::new("AND", in1.clone(), in2.clone(), |x, y| x && y);
    let mut or = Block::new("AND", in1.clone(), in2.clone(), |x, y| x || y);

    println!("AND {}", and.out());
    println!("OR  {}", or.out());
}

impl<F, O> Input<F>
where
    F: FnMut() -> O,
{
    fn new(id: &str, f: F) -> Rc<RefCell<Self>> {
        let f = Box::from(f);
        let id = id.into();
        Rc::new(RefCell::new(Input { id, f } ))
    }

    fn out(&mut self) -> O {
        (*self.f.deref_mut())()
    }
}

impl<F, F1, F2> Block<F, F1, F2>
where
    F: FnMut(bool, bool) -> bool,
    F1: FnMut() -> bool,
    F2: FnMut() -> bool,
{
    fn new(id: &str, in1: Rc<RefCell<Input<F1>>>, in2: Rc<RefCell<Input<F2>>>, f: F) -> Self
    where
        F: FnMut(bool, bool) -> bool,
    {
        let id = id.into();
        Block {
            id,
            f: f.into(),
            in1,
            in2,
        }
    }

    fn out(&mut self) -> bool {
        let mut in1 = self.in1.borrow_mut();
        let mut in2 = self.in2.borrow_mut();
        (*self.f.deref_mut())(in1.out(), in2.out())
    }
}
