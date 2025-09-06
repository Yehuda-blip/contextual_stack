use std::{
    hash::{DefaultHasher, Hash, Hasher},
    time::Instant,
};

use contextual_stack::{column, global_contexter};

const HASHER_VAL: u8 = 19;

global_contexter!(logger);

fn main() {
    call_chain_1();
    call_chain_2();
    let logs = logger::iter().collect::<Vec<_>>();
    for log in logs.iter().rev() {
        println!("{log:?}")
    }
}

fn call_chain_1() {
    let state = Instant::now().elapsed().as_micros() as i32 + 1;
    column!(CallChain1 stores (String, i32));
    let _h = logger::add_ctx_to::<CallChain1>((format!("call chain 1 state = {state}"), state));
    compute(state)
}


fn call_chain_2() {
    let mut state = -(Instant::now().elapsed().as_micros() as i32 + 1);
    column!(CallChain2 stores (String, i32));
    let _h = logger::add_ctx_to::<CallChain2>(format!("call chain 2 state = {state}"));
    column!(InLoop stores (String, usize));
    for i in 1..=5 {
        let _h = logger::add_ctx_to::<InLoop>((format!("in loop iteration {i}"), i));
        state = compute(state) as i32;
    }
    println!("final product = {}", compute(state));
}

fn compute(init: i32) -> u64 {
    let mut h = DefaultHasher::new();
    init.hash(&mut h);
    column!(HashVal stores (String, u64));
    logger::write_to::<HashVal>(format!("computed intermediate {}", h.clone().finish()));
    HASHER_VAL.hash(&mut h);
    let result = h.finish();
    logger::write_to::<HashVal>((format!("result = {result}"), result));
    result
}
