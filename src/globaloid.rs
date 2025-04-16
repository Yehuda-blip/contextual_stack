#[macro_export]
macro_rules! global_context {
    ($name:tt {$ctx:ty, $val:ty}) => {
        pub mod $name {
            use std::cell::UnsafeCell;

            use contextual_stack::{
                CtxStack, StackHandle,
                ctx_stack::{Context, Value},
            };

            type C = $ctx;
            type T = $val;
            

            static mut GLOBAL_STACK: UnsafeCell<Option<CtxStack<C, T>>> = UnsafeCell::new(None);

            #[allow(static_mut_refs)]
            fn get() -> &'static mut CtxStack<C, T> {
                unsafe {
                    match *GLOBAL_STACK.get() {
                        None => *GLOBAL_STACK.get() = Some(CtxStack::<C, T>::new()),
                        _ => {}
                    }
                    (*GLOBAL_STACK.get()).as_mut().unwrap()
                }
            }

            pub fn with_ctx<O, F> (
                ctx: C,
                action: F
            ) -> Result<
                O,
                contextual_stack::ctx_stack::ContextError<C>,
            > 
            where F: FnOnce() -> O {
                let push_result = get().push_context(ctx);
                match push_result {
                    Ok(_handle) => {
                        Ok(action())
                    },
                    Err(err) => Err(err) 
                }
            }

            pub fn write(value: T) {
                get().write(value);
            }

            pub fn get_all() -> contextual_stack::ctx_stack::CtxStackIterator<'static, C, T> {
                get().get_all()
            }
        }
    };
}
