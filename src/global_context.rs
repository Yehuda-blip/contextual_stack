#[macro_export]
macro_rules! global_contexter {
    ($name:tt) => {
        pub mod $name {
            use contextual_stack::{Column, ContextHandle, Contexter, Frame};
            use std::sync::{Mutex, MutexGuard, OnceLock};

            static CONTEXTER: OnceLock<Mutex<Contexter>> = OnceLock::new();
            const POISONED_MUTEX_ERROR: &str =
                concat!("global contexter ", stringify!($name), " mutex is poisoned");

            pub struct ThreadSafeContextHandle {
                handle: Option<contextual_stack::ContextHandle<'static>>,
            }

            impl ThreadSafeContextHandle {
                pub fn new(handle: contextual_stack::ContextHandle<'static>) -> Self {
                    Self {
                        handle: Some(handle),
                    }
                }
            }

            impl Drop for ThreadSafeContextHandle {
                fn drop(&mut self) {
                    let _guard = get();
                    let _ = self.handle.take();
                }
            }

            pub struct LockingFrameIter<'a> {
                _guard: MutexGuard<'a, Contexter>,
                iter: contextual_stack::FrameIter<'a>,
            }

            impl<'a> Iterator for LockingFrameIter<'a> {
                type Item = Frame<'a>;
                fn next(&mut self) -> Option<Self::Item> {
                    self.iter.next()
                }
            }

            fn get() -> MutexGuard<'static, Contexter> {
                CONTEXTER
                    .get_or_init(|| Mutex::new(Contexter::new()))
                    .lock()
                    .expect(POISONED_MUTEX_ERROR)
            }

            pub fn write_to<C: Column>(val: C::Entry) {
                get().write_to::<C>(val);
            }

            pub fn add_ctx_to<C: Column>(ctx: C::Entry) -> ThreadSafeContextHandle {
                ThreadSafeContextHandle::new(get().add_ctx_to::<C>(ctx))
            }

            pub fn iter() -> LockingFrameIter<'static> {
                // As of now, we lock writing for the contexter while reading results.
                // We can avoid this by making the FrameIter construct with limits on the different column lengths,
                // but currently we want to imitate an immutable borrow.
                let guard = get();
                let iter = guard.iter();
                LockingFrameIter {
                    _guard: guard,
                    iter,
                }
            }
        }
    };
}
