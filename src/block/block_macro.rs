#[derive(Debug, Clone)]
pub enum Cache<T> {
    Empty,
    Cached(usize, T),
}

impl<T> Cache<T> {
    pub fn is_valid(&self, cache_id: usize) -> bool {
        match *self {
            Cache::Empty => false,
            Cache::Cached(id, _) => cache_id == id,
        }
    }
    pub fn as_ref(&self) -> Cache<&T> {
        match self {
            Cache::Empty => Cache::Empty,
            Cache::Cached(id, ref value) => Cache::Cached(*id, value),
        }
    }
    pub fn unwrap(self) -> T {
        match self {
            Cache::Empty => panic!("Unwrapped empty cache"),
            Cache::Cached(_, value) => value,
        }
    }
    pub fn update(&mut self, id: usize, value: T) {
        *self = Cache::Cached(id, value);
    }
    pub fn map<F, U>(self, f: F) -> Cache<U>
        where F: FnOnce(T) -> U {
            match self {
                Cache::Empty => Cache::Empty,
                Cache::Cached(id, value) => Cache::Cached(id, f(value)),
            }
        }
}

impl<T> Cache<T>
where T: Default {
    pub fn unwrap_or_default(self) -> T {
        match self {
            Cache::Empty => Default::default(),
            Cache::Cached(_, value) => value,
        }
    }
}

macro_rules! define_node {
    ($visibility:vis $struct_name:ident {
        $output_number:literal {
            $($output_name:ident -> $output_index:literal),+ $(,)?
        }
    }) => {
        define_node! {
            $visibility $struct_name {
                ;$output_number {
                    $($output_name -> $output_index),+
                }
            }
        }
    };
    ($visibility:vis $struct_name:ident
     {
         $( $input_type:ident: $input_name:ident -> $input_index:literal ),*
           ;$output_number:literal {
               $($output_name:ident -> $output_index:literal),+ $(,)?
           }
     } )
    => {

        #[derive(Clone)]
        $visibility struct $struct_name<F, $($input_type,)* O>
        where
            F: FnMut($(&$input_type),*) -> [O; $output_number],
            O: Clone,
        {
            id: String,
            am_i_in_a_cycle: ::std::cell::RefCell<()>,
            cache: ::std::cell::RefCell<$crate::block::block_macro::Cache<[O; $output_number]>>,
            $(
                $input_name: Option<::std::rc::Rc<::std::cell::RefCell<
                    $crate::block::Wire<Output = $input_type>>>>,
            )*
            f: ::std::cell::RefCell<F>,
        }

        paste::item! {
            impl<F, $($input_type,)* O> $struct_name<F, $($input_type,)* O>
            where
                F: FnMut($(&$input_type),*) -> [O; $output_number],
                $($input_type: Clone,)*
                O: Clone + Default + ::std::fmt::Debug,
            {
                pub fn new(id: &str, f: F) ->
                    (::std::rc::Rc<::std::cell::RefCell<Self>>,
                     [::std::rc::Rc<::std::cell::RefCell<$crate::block::Handle<Self>>>;
                     $output_number]) {
                    let id = id.into();
                    let cache = ::std::cell::RefCell::new($crate::block::block_macro::Cache::Empty);
                    let am_i_in_a_cycle = ::std::cell::RefCell::new(());
                    let block = ::std::rc::Rc::new(::std::cell::RefCell::new($struct_name {
                        id,
                        f: f.into(),
                        $($input_name: None,)*
                        cache,
                        am_i_in_a_cycle,
                    }));
                    $(
                        let $output_name =
                        ::std::rc::Rc::new(::std::cell::RefCell::new($crate::block::Handle {
                            index: $output_index,
                            block: block.clone(),
                        }));
                    )+
                    (block, [$($output_name,)+])
                }
                $(
                    pub fn [<plug_ $input_name>](
                        &mut self,
                        inp: ::std::rc::Rc<::std::cell::RefCell<
                            $crate::block::Wire<Output = $input_type>>>
                    ) -> &mut Self {
                        self.$input_name = Some(inp);
                        self
                    }
                )*
            }
        }

        impl<F, $($input_type,)* O> $crate::block::Node for $struct_name<F, $($input_type,)* O>
        where
            F: FnMut($(&$input_type,)*) -> [O; $output_number],
            $($input_type: Clone,)*
            O: Clone + Default + ::std::fmt::Debug,
        {
            type Output = O;

            fn get(&self, index: usize, cache_id: usize) -> Self::Output {
                // Detect recursion
                if self.am_i_in_a_cycle.try_borrow_mut().is_err() {
                    return self.cache
                        .try_borrow()
                        .expect("Borrowing cache failed")
                        .as_ref()
                        .map(|cached| cached[index].clone())
                        .unwrap_or_default();
                }
                let _x = self.am_i_in_a_cycle.borrow_mut();
                if ! self.cache
                    .try_borrow()
                    .expect("Borrow of cache failed #1")
                    .is_valid(cache_id) {
                    $(
                        let $input_name = self.$input_name
                        .as_ref()
                        .expect(stringify!(A $struct_name needs to have all inputs defined))
                        .try_borrow()
                        .expect(stringify!(Borrow of $input_name in $struct_name failed))
                        .out(cache_id);
                    )*;
                    // Interior mutability for f
                    let mut f = self.f
                        .try_borrow_mut()
                        .expect("Mutable borrow of function failed");
                    let f = ::std::ops::DerefMut::deref_mut(&mut f);
                    self.cache
                        .try_borrow_mut()
                        .expect("Mutable borrow of cache failed")
                        .update(cache_id, f($(&$input_name,)*));
                }
                self.cache
                    .try_borrow()
                    .expect("Borrow of cache failed #2")
                    .as_ref()
                    .unwrap()[index]
                    .clone()
            }
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
