use contextual_stack::global_context;

global_context!(
    logger {String, String} 
    with unchecked as logger_ctx_unchecked
);


fn main() {
    let val = 1;
    let _h = logger_ctx_unchecked!("".into());
    logger::with_ctx("".into(), || print!("{val}")).ok();
}
