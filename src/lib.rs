pub mod ctx_stack;

#[cfg(test)]
mod tests {
    use ctx_stack::CtxStack;

    use super::*;

    #[test]
    fn it_works() {
        let stc = CtxStack::<i32>::new();
        print!("{stc:?}")
    }
}
