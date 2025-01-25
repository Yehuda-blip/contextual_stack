use crate::ctx_stack::{Context, ContextError, ContextHandle, CtxStack, Value};


#[macro_export]
macro_rules! push_context {
    ($stack:ident, $name:ident, $ctx:ident, $put_err:ident) => {
        let handle = $crate::hidden_push_api::push_context_do_not_call($stack, $name, $ctx);
        if let Err(err) = handle {
            *$put_err = Err(err)
        }
    };
}

/// Do not use this function, it must be since it is used in the exported [`push_context`] macro,
/// But it does not know about and does not adhere to the scoping rules used in this crate. The 
/// `context_stack` module assumes some non-trivial things about variable scope, which is why you
/// should only use [`push_context`].
pub fn push_context_do_not_call<'a, C: Context, T: Value>(
    stack: &'a mut CtxStack<C, T>,
    name: String,
    ctx: C
) -> Result<ContextHandle<'a, C, T>, ContextError<C>> {
    stack.push_context(name, ctx)
}