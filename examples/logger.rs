use std::cell::UnsafeCell;

use contextual_stack::ctx_stack::{ContextError, ContextHandle, CtxStack};

struct Logger {
    logs: CtxStack<String>,
}

static mut LOGGER: UnsafeCell<Option<Logger>> = UnsafeCell::new(None);

#[allow(static_mut_refs)]
unsafe fn get_logger() -> &'static mut Logger {
    unsafe {
        match *LOGGER.get() {
            None => {
                *LOGGER.get() = Some(Logger {
                    logs: CtxStack::new(),
                })
            }
            _ => {}
        }
        (*LOGGER.get()).as_mut().unwrap()
    }
}

// macro_rules! add_ctx {
//     ($name:literal, $value:literal) => {
//         let logs = &mut unsafe{&*LOGGER.get()};
//         let _handle = logs.push_context($name.to_owned(), $value.to_owned());
//     };
// }

fn add_ctx(name: &str, value: &str) -> Result<ContextHandle<'static, String>, ContextError> {
    let logger = unsafe { get_logger() };
    let handle = logger.logs.push_context(name.to_owned(), value.to_owned());
    handle
}

fn write(log: String) {
    let logger = unsafe { get_logger() };
    logger.logs.write(log);
}

fn print_logs() {
    let logger = unsafe { get_logger() };
    for frame in logger.logs.get_all() {
        println!("{frame:?}")
    }
}

// macro_rules! write {
//     ($log:expr) => {
//         let logs = &mut unsafe{&*LOGGER}.borrow_mut().logs;
//         logs.
//     };
// }

fn main() {
    // add_ctx!("context1", "the_first_value");
    println!("running");
    let handle = add_ctx("context1", "value1").expect("");
    write("first log".into());
    drop(handle);
    print_logs();
}
