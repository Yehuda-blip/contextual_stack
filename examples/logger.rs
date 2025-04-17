use std::{
    hash::{DefaultHasher, Hash, Hasher},
    time::Instant,
};

use contextual_stack::global_context;

const HASHER_VAL: u8 = 19;

global_context!(
    logger {String, String}
    with unchecked as logger_ctx_unchecked
);

fn main() {
    call_chain_1();
    call_chain_2();
    let logs = logger::get_all().collect::<Vec<_>>();
    for log in logs.iter().rev() {
        println!("{log:?}")
    }
}

fn call_chain_1() {
    let state = Instant::now().elapsed().as_micros() as i32 + 1;
    logger::with_ctx(format!("call chain 1 state = {state}"), || {
        println!("final product = {}", compute(state));
    })
    .expect("bad context");
}

fn call_chain_2() {
    let state = -(Instant::now().elapsed().as_micros() as i32 + 1);
    let Ok(_handle) = logger_ctx_unchecked!(format!("call chain 2 state = {state}"))
    else {panic!("bad ctx")};
    println!("final product = {}", compute(state));
}

fn compute(init: i32) -> u64 {
    let mut h = DefaultHasher::new();
    init.hash(&mut h);
    logger::write(format!("computed intermediate {}", h.clone().finish()));
    HASHER_VAL.hash(&mut h);
    let result = h.finish();
    logger::write(format!("result = {result}"));
    result
}
