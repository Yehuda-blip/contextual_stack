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

fn main() {
    println!("running");
    let _handle = add_ctx("context1", "value1").expect("");
    write("first log".into());
    context2();
    context3();
    print_logs();
    // prints:
    // Frame { context: {"context3": "value3", "context1": "value1"}, value: "Third log" }
    // Frame { context: {"context2": "value2", "context1": "value1"}, value: "Second log" }
    // Frame { context: {"context1": "value1"}, value: "first log" }
}

fn context2() {
    let _handle = add_ctx("context2", "value2").expect("");
    write("Second log".into());
}
fn context3() {
    let _handle = add_ctx("context3", "value3").expect("");
    write("Third log".into());
}
