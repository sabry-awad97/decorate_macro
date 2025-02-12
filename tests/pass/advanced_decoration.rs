use decorate_macro::decorate;
use std::sync::atomic::{AtomicUsize, Ordering};

static EXECUTION_COUNT: AtomicUsize = AtomicUsize::new(0);

fn log_execution<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    println!("Log: executing function");
    let result = f();
    println!("Log: finished");
    result
}

#[decorate(
    pre = EXECUTION_COUNT.fetch_add(1, Ordering::SeqCst),
    post = println!("Execution finished"),
    log_execution
)]
fn compute(x: i32, y: i32) -> i32 {
    x + y
}

fn main() {
    EXECUTION_COUNT.store(0, Ordering::SeqCst);
    let result = compute(1, 2);
    assert_eq!(result, 3);
    assert_eq!(EXECUTION_COUNT.load(Ordering::SeqCst), 1);
}
