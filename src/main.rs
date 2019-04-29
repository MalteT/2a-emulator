use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;

trait Input {
    type Output;
    fn out(&mut self, clock: usize) -> &Self::Output;
}

#[derive(Debug, Clone)]
struct Block2<F, I1, I2, O>
where
    I1: Input,
    I2: Input,
    F: FnMut(usize, &I1::Output, &I2::Output) -> O,
{
    id: String,
    last_clock: usize,
    last_result: Option<O>,
    in1: Rc<RefCell<I1>>,
    in2: Rc<RefCell<I2>>,
    f: Box<F>,
}

#[derive(Debug, Clone)]
struct RawInput<F, O>
where
    F: FnMut(usize) -> O,
{
    id: String,
    last_clock: usize,
    last_result: Option<O>,
    f: Box<F>,
}

fn main() {
    let in1 = RawInput::new("Always TRUE", |c| {
        let now = c % 4 == 2 || c % 4 == 3;
        print!("CLOCK: {}, IN1 {},\t", c, now);
        now
    });
    let in2 = RawInput::new("Always FALSE", |c| {
        let now = c % 2 == 1;
        print!("IN2 {},\t", now);
        now
    });
    let and1 = Block2::new("AND_1", in1.clone(), in2.clone(), |_, &x, &y| x && y);
    let and2 = Block2::new("AND_2", in1.clone(), in2.clone(), |_, &x, &y| !x && !y);
    let and3 = Block2::new("EQUALS", and1.clone(), and2.clone(), |_, &x, &y| x || y);

    for clock in 0..4 {
        println!("EQUALS {}", and3.borrow_mut().out(clock));
    }
}

impl<F, O> RawInput<F, O>
where
    F: FnMut(usize) -> O,
{
    fn new(id: &str, f: F) -> Rc<RefCell<Self>> {
        let f = Box::from(f);
        let id = id.into();
        let last_clock = 0;
        let last_result = None;
        Rc::new(RefCell::new(RawInput {
            id,
            f,
            last_clock,
            last_result,
        }))
    }
}

impl<F, I1, I2, O> Block2<F, I1, I2, O>
where
    F: FnMut(usize, &I1::Output, &I2::Output) -> O,
    I1: Input,
    I2: Input,
{
    fn new(id: &str, in1: Rc<RefCell<I1>>, in2: Rc<RefCell<I2>>, f: F) -> Rc<RefCell<Self>> {
        let id = id.into();
        let last_clock = 0;
        let last_result = None;
        Rc::new(RefCell::new(Block2 {
            id,
            f: f.into(),
            in1,
            in2,
            last_clock,
            last_result,
        }))
    }
}

impl<F, O> Input for RawInput<F, O>
where
    F: FnMut(usize) -> O,
{
    type Output = O;

    fn out(&mut self, clock: usize) -> &Self::Output {
        if self.last_clock != clock || self.last_result.is_none() {
            self.last_clock = clock;
            self.last_result = Some(self.f.deref_mut()(clock));
        }
        self.last_result.as_ref().unwrap()
    }
}

impl<F, I1, I2, O> Input for Block2<F, I1, I2, O>
where
    I1: Input,
    I2: Input,
    F: FnMut(usize, &I1::Output, &I2::Output) -> O,
{
    type Output = O;

    fn out(&mut self, clock: usize) -> &O {
        if self.last_clock != clock || self.last_result.is_none() {
            self.last_clock = clock;
            let mut in1 = self.in1.borrow_mut();
            let mut in2 = self.in2.borrow_mut();
            self.last_result = Some(self.f.deref_mut()(clock, &in1.out(clock), &in2.out(clock)));
        }
        self.last_result.as_ref().unwrap()
    }
}
