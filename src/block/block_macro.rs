macro_rules! block {
    ($visibility:vis $name:ident;
     $($input_name:ident: $input:tt),*;
     $outputs:expr;
     $($out_name:ident: $out_index:expr);*) => {

        #[derive(Debug, Clone)]
        $visibility struct $name<F, $($input,)* O>
        where
            $($input: Wire,)*
            F: FnMut(usize, $(&$input::Output),*) -> [O; $outputs],
            O: Clone,
        {
            id: String,
            last_clock: usize,
            last_output: Option<[O; $outputs]>,
            $($input_name: Rc<RefCell<$input>>,)*
            f: Box<F>,
        }

        impl<F, $($input,)* O> $name<F, $($input,)* O>
        where
            $($input: Wire,)*
            F: FnMut(usize, $(&$input::Output),*) -> [O; $outputs],
            O: Clone,
        {
            pub fn new(id: &str, $($input_name: Rc<RefCell<$input>>,)* f: F) ->
                (Rc<RefCell<Self>>, [Rc<RefCell<Handle<Self>>>; $outputs]) {
                let id = id.into();
                let last_clock = 0;
                let block = Rc::new(RefCell::new($name {
                    id,
                    f: f.into(),
                    $($input_name,)*
                    last_clock,
                    last_output: None,
                }));
                $(
                    let $out_name = Rc::new(RefCell::new(Handle {
                        index: $out_index,
                        block: block.clone(),
                    }));
                )+
                (block, [$($out_name,)+])
            }
        }

        impl<F, $($input,)* O> Node for $name<F, $($input,)* O>
        where
            $($input: Wire,)*
            F: FnMut(usize, $(&$input::Output,)*) -> [O; $outputs],
            O: Clone,
        {
            type Output = O;

            fn get(&mut self, index: usize, clock: usize) -> Self::Output {
                if self.last_clock != clock || self.last_output.is_none() {
                    self.last_clock = clock;
                    $(
                        let $input_name = self.$input_name.borrow_mut().out(clock);
                    )*
                    self.last_output = Some(self.f.deref_mut()(clock, $(&$input_name,)*));
                }
                self.last_output.as_ref().unwrap()[index].clone()
            }
        }
    };
    ($visibility:vis $name:ident;
     $($input_name:ident: $input:tt),*;
     1) => {

        #[derive(Debug, Clone)]
        $visibility struct $name<F, $($input,)* O>
        where
            $($input: Wire,)*
            F: FnMut(usize, $(&$input::Output),*) -> O,
            O: Clone,
        {
            id: String,
            last_clock: usize,
            last_output: Option<O>,
            $($input_name: Rc<RefCell<$input>>,)*
            f: Box<F>,
        }

        impl<F, $($input,)* O> $name<F, $($input,)* O>
        where
            $($input: Wire,)*
            F: FnMut(usize, $(&$input::Output),*) -> O,
            O: Clone,
        {
            pub fn new(id: &str, $($input_name: Rc<RefCell<$input>>,)* f: F) ->
                (Rc<RefCell<Self>>, Rc<RefCell<Handle<Self>>>) {
                let id = id.into();
                let last_clock = 0;
                let block = Rc::new(RefCell::new($name {
                    id,
                    f: f.into(),
                    $($input_name,)*
                    last_clock,
                    last_output: None,
                }));
                let out = Rc::new(RefCell::new(Handle {
                    index: 0,
                    block: block.clone(),
                }));
                (block, out)
            }
        }

        impl<F, $($input,)* O> Node for $name<F, $($input,)* O>
        where
            $($input: Wire,)*
            F: FnMut(usize, $(&$input::Output,)*) -> O,
            O: Clone,
        {
            type Output = O;

            fn get(&mut self, _index: usize, clock: usize) -> Self::Output {
                if self.last_clock != clock || self.last_output.is_none() {
                    self.last_clock = clock;
                    $(
                        let $input_name = self.$input_name.borrow_mut().out(clock);
                    )*
                    self.last_output = Some(self.f.deref_mut()(clock, $(&$input_name,)*));
                }
                self.last_output.as_ref().unwrap().clone()
            }
        }
    }
}
