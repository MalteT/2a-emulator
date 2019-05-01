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
        $visibility struct $struct_name<'a, F, $($input_type,)* O>
        where
            F: FnMut($(&$input_type),*) -> [O; $output_number],
            O: Clone,
            $($input_type: Clone,)*
        {
            id: String,
            am_i_in_a_cycle: ::std::cell::RefCell<()>,
            lifetime: ::std::marker::PhantomData<&'a O>,
            cache: ::std::cell::RefCell<$crate::block::cache::Cache<[O; $output_number]>>,
            $(
                $input_name: Option<::std::cell::RefCell<$crate::block::Wire<'a, $input_type>>>,
            )*
            f: ::std::cell::RefCell<F>,
        }

        paste::item! {
            impl<'a, F, $($input_type,)* O> $struct_name<'a, F, $($input_type,)* O>
            where
                F: FnMut($(&$input_type),*) -> [O; $output_number] + 'a,
                $($input_type: Clone + 'a,)*
                O: Clone + Default + ::std::fmt::Debug,
            {
                pub fn new(id: &str, f: F) ->
                    (::std::rc::Rc<::std::cell::RefCell<Self>>,
                     [$crate::block::Wire<'a, O>; $output_number]
                ) {
                    use std::cell::RefCell;
                    use std::rc::Rc;
                    use std::marker::PhantomData;
                    use $crate::block::cache::Cache;

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
                        $crate::block::Wire {
                            index: $output_index,
                            node: node.clone(),
                        };
                    )+
                    (node, [$($output_name,)+])
                }
                $(
                    pub fn [<plug_ $input_name>](
                        &mut self,
                        inp: $crate::block::Wire<'a, $input_type>
                    ) -> &mut Self {
                        use std::cell::RefCell;
                        self.$input_name = Some(RefCell::new(inp));
                        self
                    }
                )*
            }
        }

        impl<'a, F, $($input_type,)* O> $crate::block::Node for $struct_name<'a, F, $($input_type,)* O>
        where
            F: FnMut($(&$input_type,)*) -> [O; $output_number],
            $($input_type: Clone,)*
            O: Clone + Default + ::std::fmt::Debug,
        {
            type Output = O;

            unsafe fn get(&self, index: usize, cache_id: usize) -> Self::Output {
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
                        .try_borrow_mut()
                        .expect(stringify!(Mutable borrow failed for $input_name on $crate_name))
                        .get(cache_id);
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
