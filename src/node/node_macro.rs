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
    // The one with only one output and no inputs
    ($visibility:vis $struct_name:ident {
        display {
            $format_str:expr, $($format_args:ident),* $(,)?
        } $(,)?
    }) => {
        define_node! {
            $visibility $struct_name {
                outputs 1 {
                    output -> 0
                },
                display {
                    $format_str, $($format_args),*
                },
            }
        }
    };
    // The one without inputs
    ($visibility:vis $struct_name:ident {
        outputs $output_number:literal {
            $($output_name:ident -> $output_index:literal),+ $(,)?
        },
        display {
            $format_str:expr, $($format_args:ident),* $(,)?
        } $(,)?
    }) => {
        define_node! {
            $visibility $struct_name {
                inputs {},
                outputs $output_number {
                    $($output_name -> $output_index),+
                },
                display {
                    $format_str, $($format_args),*
                },
            }
        }
    };
    // The one without outputs (just the one default output)
    ($visibility:vis $struct_name:ident {
        inputs {
            $( $input_type:ident: $input_name:ident -> $input_index:literal ),* $(,)?
        },
        display {
            $format_str:expr, $($format_args:ident),* $(,)?
        } $(,)?
    }) => {
        define_node! {
            $visibility $struct_name {
                inputs {
                    $($input_type: $input_name -> $input_index),*
                },
                outputs 1 {
                    output -> 0
                },
                display {
                    $format_str, $($format_args),*
                },
            }
        }
    };
    // The complete one!
    ($visibility:vis $struct_name:ident {
        inputs {
            $( $input_type:ident: $input_name:ident -> $input_index:literal ),* $(,)?
        },
        outputs $output_number:literal {
            $($output_name:ident -> $output_index:literal),+ $(,)?
        },
        display {
            $format_str:expr, $($format_args:ident),* $(,)?
        } $(,)?
     }) => {

        $visibility struct $struct_name<'a, F, $($input_type,)* O>
        {
            id: String,
            am_i_in_a_cycle: ::std::cell::RefCell<()>,
            lifetime: ::std::marker::PhantomData<&'a O>,
            cache: ::std::cell::RefCell<$crate::node::Cache<out![O; $output_number, $($output_name),*]>>,
            $(
                $input_name: (Option<::std::cell::RefCell<$crate::node::Wire<'a, $input_type>>>,
                              ::std::cell::RefCell<Option<$input_type>>),
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
                    $(let $input_name = (None, RefCell::new(None));)*
                    let node = Rc::new(RefCell::new($struct_name {
                        id,
                        f: f.into(),
                        $($input_name,)*
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
                        self.$input_name = (Some(RefCell::new(inp)), RefCell::new(None));
                        self
                    }
                )*

            }
        }

        impl<'a, F, $($input_type,)* O> $crate::node::Node for $struct_name<'a, F, $($input_type,)* O>
        where
            F: FnMut($(&$input_type,)*) -> out![O; $output_number, $($output_name),+],
            $($input_type: ::std::fmt::Debug,)*
            O: Clone + Default + ::std::fmt::Debug,
        {
            type Output = O;

            unsafe fn get(&self, _index: usize, cache_id: usize) -> Self::Output {
                // Detect recursion
                if self.am_i_in_a_cycle.try_borrow_mut().is_err() {
                    let last_value = self.cache
                        .try_borrow()
                        .expect("Borrowing cache failed")
                        .as_ref()
                        .map(|cached| iftwo! { $($output_name),+;
                            { cached[_index].clone() };
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
                        *self.$input_name.1.borrow_mut() = self.$input_name.0
                        .as_ref()
                        .expect(stringify!(A $struct_name needs to have all inputs defined))
                        .try_borrow_mut()
                        .expect(stringify!(Mutable borrow failed for $input_name on $struct_name))
                        .get(cache_id)
                        .into();
                        log::trace!(target: &format!("{} > {}", stringify!($struct_name), self.id),
                                    "Got {:?} from input {}",
                                    self.$input_name.1.borrow().as_ref().unwrap(),
                                    stringify!($input_name));
                    )*;
                    // Interior mutability for f
                    let mut f = self.f
                        .try_borrow_mut()
                        .expect("Mutable borrow of function failed");
                    let f = ::std::ops::DerefMut::deref_mut(&mut f);
                    self.cache
                        .try_borrow_mut()
                        .expect("Mutable borrow of cache failed")
                        .update(cache_id,
                                f($(
                                    &self.$input_name.1.borrow().as_ref().unwrap(),
                                )*));
                }
                iftwo! { $($output_name),+;
                    {
                        self.cache
                            .try_borrow()
                            .expect("Borrow of cache failed #2")
                            .as_ref()
                            .unwrap()[_index]
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

        impl_display! {
            $struct_name {
                inputs {
                    $( $input_type ),*
                },
                display {
                    $format_str $(, $format_args)*
                }
            }
        }
    };
}

/// Implement [`Display`](::std::fmt::Display) for the given struct.
macro_rules! impl_display {
    ($struct_name:ident {
        inputs {
            $( $input_type:ident ),*
        },
        display {
            $format_str:expr $(,$format_args:ident)* $(,)?
        }
    }) => {
        // Implement Display for the given struct
        impl<'a, F, $($input_type,)* O> ::std::fmt::Display
            for $struct_name<'a, F, $($input_type,)* O>
        where
            $($input_type: Default + Clone + ::std::fmt::Display,)*
        {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, $format_str, self.id,
                       $(self.$format_args.1.borrow().clone().unwrap_or_default()),*)
            }
        }
    }
}
