use contextual_stack::global_context;

global_context!(
    logger {String, String}
);


fn main() {
    let val = 1;
    logger::with_ctx("".into(), || print!("{val}")).ok();
}
