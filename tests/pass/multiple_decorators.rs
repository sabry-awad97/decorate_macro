use decorate_macro::decorate;
use std::cell::RefCell;

// Track execution order
thread_local! {
    static EXECUTION_ORDER: RefCell<Vec<&'static str>> = RefCell::new(Vec::new());
}

fn log_start<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    EXECUTION_ORDER.with(|order| order.borrow_mut().push("start"));
    let result = f();
    EXECUTION_ORDER.with(|order| order.borrow_mut().push("start_end"));
    result
}

fn log_middle<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    EXECUTION_ORDER.with(|order| order.borrow_mut().push("middle"));
    let result = f();
    EXECUTION_ORDER.with(|order| order.borrow_mut().push("middle_end"));
    result
}

fn log_end<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    EXECUTION_ORDER.with(|order| order.borrow_mut().push("end"));
    let result = f();
    EXECUTION_ORDER.with(|order| order.borrow_mut().push("end_end"));
    result
}

#[decorate(log_start, log_middle, log_end)]
fn test_function(x: i32) -> i32 {
    EXECUTION_ORDER.with(|order| order.borrow_mut().push("function"));
    x * 2
}

fn main() {
    let result = test_function(5);
    assert_eq!(result, 10);

    // Verify execution order
    EXECUTION_ORDER.with(|order| {
        assert_eq!(
            &*order.borrow(),
            &[
                "start",
                "middle",
                "end",
                "function",
                "end_end",
                "middle_end",
                "start_end"
            ]
        );
    });
}
