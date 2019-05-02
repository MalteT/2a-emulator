/// Decide which output type to use for [`define_node`].
macro_rules! out {
    [$type:ty; $number:literal, $x:ident] => { $type };
    [$type:ty; $number:literal, $($x:ident),+] => { [ $type; $number ] };
    [$el:expr $(,)?] => { $el };
    [$($el:expr),+ $(,)?] => { [ $($el),+ ] };
}

/// Return `$if`, if more than one token is given in `$x` and `$else` otherwise.
macro_rules! iftwo {
    ($x:tt; $if:tt; $else:tt $(;)?) => {
        $else
    };
    ($($x:tt),+; $if:tt; $else:tt $(;)?) => {
        $if
    };
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
    ($visibility:vis $struct_name:ident {
         $( $input_type:ident: $input_name:ident -> $input_index:literal ),* $(,)?
    }) => {
        define_node! {
            $visibility $struct_name {
                $( $input_type: $input_name -> $input_index ),*;
                1 {
                    out -> 0
                }
            }
        }
    };
    ($visibility:vis $struct_name:ident {
         $( $input_type:ident: $input_name:ident -> $input_index:literal ),*
           ;$output_number:literal {
               $($output_name:ident -> $output_index:literal),+ $(,)?
           }
     }) => {

        $visibility struct $struct_name<'a, F, $($input_type,)* O>
        where
            F: FnMut($(&$input_type),*) -> out![O; $output_number, $($output_name),*],
            O: Clone,
            $($input_type: Clone + ::std::fmt::Debug,)*
        {
            id: String,
            am_i_in_a_cycle: ::std::cell::RefCell<()>,
            lifetime: ::std::marker::PhantomData<&'a O>,
            cache: ::std::cell::RefCell<$crate::node::Cache<out![O; $output_number, $($output_name),*]>>,
            $(
                $input_name: Option<::std::cell::RefCell<$crate::node::Wire<'a, $input_type>>>,
            )*
            f: ::std::cell::RefCell<F>,
        }

        paste::item! {
            impl<'a, F, $($input_type,)* O> $struct_name<'a, F, $($input_type,)* O>
            where
                F: FnMut($(&$input_type),*) -> out![O; $output_number, $($output_name),+] + 'a,
                $($input_type: Clone + ::std::fmt::Debug + 'a,)*
                O: Clone + Default + ::std::fmt::Debug,
            {
                pub fn new(id: &str, f: F) ->
                    (::std::rc::Rc<::std::cell::RefCell<Self>>,
                     out![$crate::node::Wire<'a, O>; $output_number, $($output_name),+]
                ) {
                    use std::cell::RefCell;
                    use std::rc::Rc;
                    use std::marker::PhantomData;
                    use $crate::node::Cache;

                    let id = id.into();
                    let cache = RefCell::new(Cache::Empty);
                    let am_i_in_a_cycle = RefCell::new(());
                    let lifetime = PhantomData{};
                    let node = Rc::new(RefCell::new($struct_name {
                        id,
                        f: f.into(),
                        $($input_name: None,)*
                        cache,
                        lifetime,
                        am_i_in_a_cycle,
                    }));
                    $(
                        let $output_name =
                        $crate::node::Wire {
                            index: $output_index,
                            node: node.clone(),
                        };
                    )+
                    (node, out![$($output_name,)+])
                }
                $(
                    pub fn [<plug_ $input_name>](
                        &mut self,
                        inp: $crate::node::Wire<'a, $input_type>
                    ) -> &mut Self {
                        use std::cell::RefCell;
                        self.$input_name = Some(RefCell::new(inp));
                        self
                    }
                )*
            }
        }

        impl<'a, F, $($input_type,)* O> $crate::node::Node for $struct_name<'a, F, $($input_type,)* O>
        where
            F: FnMut($(&$input_type,)*) -> out![O; $output_number, $($output_name),+],
            $($input_type: Clone + ::std::fmt::Debug,)*
            O: Clone + Default + ::std::fmt::Debug,
        {
            type Output = O;

            unsafe fn get(&self, index: usize, cache_id: usize) -> Self::Output {
                // Detect recursion
                if self.am_i_in_a_cycle.try_borrow_mut().is_err() {
                    let last_value = self.cache
                        .try_borrow()
                        .expect("Borrowing cache failed")
                        .as_ref()
                        .map(|cached| iftwo! { $($output_name),+;
                            { cached[index].clone() };
                            { cached.clone() };
                        })
                        .unwrap_or_default();
                    log::trace!(target: &format!("{} > {}", stringify!($struct_name), self.id),
                                "Cycle detected. Returning {:?}", last_value);
                    return last_value;
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
                        .try_borrow_mut()
                        .expect(stringify!(Mutable borrow failed for $input_name on $struct_name))
                        .get(cache_id);
                        log::trace!(target: &format!("{} > {}", stringify!($struct_name), self.id),
                                    "Got {:?} from input {}", $input_name, stringify!($input_name));
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
                iftwo! { $($output_name),+;
                    {
                        self.cache
                            .try_borrow()
                            .expect("Borrow of cache failed #2")
                            .as_ref()
                            .unwrap()[index]
                            .clone()
                    };
                    {
                        self.cache
                            .try_borrow()
                            .expect("Borrow of cache failed #2")
                            .as_ref()
                            .unwrap()
                            .clone()
                    };
                }
            }
        }
    };
}
