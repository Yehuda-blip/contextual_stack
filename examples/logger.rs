extern crate contextual_stack;
use std::cell::UnsafeCell;

use contextual_stack::CtxStack;
use contextual_stack::push_context;

struct Logger {
    logs: CtxStack<String, String>,
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

// fn add_ctx(name: &str, value: &str) -> Result<ContextHandle<'static, String, String>, ContextError<String>> {
//     let logger = unsafe { get_logger() };
//     let handle = logger.logs.push_context(name.to_owned(), value.to_owned());
//     handle
// }

macro_rules! add_ctx {
    ($name:expr, $value:expr, $put_err:ident) => {
        let logger = unsafe { get_logger() };
        let logs = &mut logger.logs;
        let (name, value) = ($name, $value);
        push_context!(logs, name, value, $put_err)
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
    let put_err = &mut Ok(());
    add_ctx!("context1".into(), "value1".into(), put_err);
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
    let put_err = &mut Ok(());
    add_ctx!("context2".into(), "value2".into(), put_err);
    if let Err(e) = put_err {
        panic!("{e:?}")
    }
    write("Second log".into());
}
fn context3() {
    let put_err = &mut Ok(());
    add_ctx!("context3".into(), "value3".into(), put_err);
    if let Err(e) = put_err {
        panic!("{e:?}")
    }
    write("Third log".into());
}
fn context_loop() {
    let put_err = &mut Ok(());
    add_ctx!(
        "outer loop context".into(),
        "outer_context_value".into(),
        put_err
    );
    if let Err(e) = put_err {
        panic!("{e:?}")
    }
    for i in 4..6 {
        let put_err = &mut Ok(());
        add_ctx!(format!("context{i}"), format!("value{i}"), put_err);
        if let Err(e) = put_err {
            panic!("{e:?}")
        }
        write(format!("writing in context {i}"));
    }
}
