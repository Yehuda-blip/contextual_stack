pub mod ctx_stack;
pub mod globaloid; 

pub use ctx_stack::CtxStack;
pub use ctx_stack::StackHandle;

#[cfg(test)]
mod tests {
    use ctx_stack::CtxStack;

    use super::*;

    #[test]
    fn it_works() {
        let stc = CtxStack::<i32, f64>::new();
        print!("{stc:?}")
    }
}
