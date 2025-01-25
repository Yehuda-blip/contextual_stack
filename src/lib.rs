pub mod ctx_stack;
pub mod hidden_push_api;

pub use ctx_stack::CtxStack;

#[cfg(test)]
mod tests {
    use ctx_stack::CtxStack;

    use super::*;

    #[test]
    fn it_works() {
        let stc = CtxStack::<i32, i32>::new();
        print!("{stc:?}")
    }
}
