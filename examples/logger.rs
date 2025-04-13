extern crate contextual_stack;
use std::cell::UnsafeCell;
use std::time::SystemTime;

use contextual_stack::CtxStack;
use contextual_stack::ctx_stack::StackHandle;

struct Logger {
    logs: CtxStack<SystemTime, String, String>,
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

macro_rules! ctx {
    ($ctx:expr) => {
        let logger = unsafe { get_logger() };
        let _handle = logger.logs.push_context(SystemTime::now(), $ctx);
    };
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
    write("first log".into());
    context2();
    context3();
    print_logs();
    // prints:
    // Frame { context: {"context3": "value3", "context1": "value1"}, value: "Third log" }
    // Frame { context: {"context2": "value2", "context1": "value1"}, value: "Second log" }
    // Frame { context: {"context1": "value1"}, value: "first log" }
    // should_fail_compilation();
    context_loop();
    print_logs();
    // prints:
    // Frame { context: {"context1": "value1", "context3": "value3"}, value: "Third log" }
    // Frame { context: {"context2": "value2", "context1": "value1"}, value: "Second log" }
    // Frame { context: {"context1": "value1"}, value: "first log" }
    // Frame { context: {"context5": "value5", "context1": "value1", "outer loop context": "outer_context_value"}, value: "writing in context 5" }
    // Frame { context: {"context1": "value1", "outer loop context": "outer_context_value", "context4": "value4"}, value: "writing in context 4" }
    // Frame { context: {"context1": "value1", "context3": "value3"}, value: "Third log" }
    // Frame { context: {"context1": "value1", "context2": "value2"}, value: "Second log" }
    // Frame { context: {"context1": "value1"}, value: "first log" }
}

fn context2() {
    ctx!("context2".into());
    write("Second log".into());
}
fn context3() {
    ctx!("context3".into());
    write("Third log".into());
}
fn context_loop() {
    ctx!("outer loop context".into());
    for i in 4..6 {
        ctx!(format!("context{i}"));
        write(format!("writing in context {i}"));
    }
}
