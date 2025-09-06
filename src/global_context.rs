// #[macro_export]
// macro_rules! global_contexter {
//     (TEMP_NAME:tt) => {
pub mod TEMP_NAME {
    use crate::contexter::{Column, ContextHandle, Contexter, Frame, FrameIter};
    use std::sync::{Mutex, MutexGuard, OnceLock};

    static CONTEXTER: OnceLock<Mutex<Contexter>> = OnceLock::new();
    const POISONED_MUTEX_ERROR: &str = concat!(
        "global contexter ",
        stringify!(TEMP_NAME),
        " mutex is poisoned"
    );

    pub struct ThreadSafeContextHandle {
        handle: Option<crate::contexter::ContextHandle<'static>>,
    }

    impl ThreadSafeContextHandle {
        pub fn new(handle: crate::contexter::ContextHandle<'static>) -> Self {
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

    pub struct LockingFrameValues<'a> {
        _guard: MutexGuard<'a, Contexter>,
    }

    impl<'a> LockingFrameValues<'a> {
        fn iter(&'a self) -> FrameIter<'a> {
            self._guard.iter()
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

    pub fn values() -> LockingFrameValues<'static> {
        // As of now, we lock writing for the contexter while reading results.
        // We can avoid this by making the FrameIter construct with limits on the different column lengths,
        // but currently we want to imitate an immutable borrow.
        let guard = get();
        LockingFrameValues { _guard: guard }
    }
}
//     };
// }
