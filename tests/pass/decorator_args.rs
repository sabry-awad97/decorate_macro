use decorate_macro::decorate;
use std::sync::atomic::{AtomicUsize, Ordering};

static RETRY_COUNT: AtomicUsize = AtomicUsize::new(0);

fn with_retry<F, R>(attempts: u32, f: F) -> R
where
    F: FnOnce() -> R,
{
    RETRY_COUNT.store(attempts as usize, Ordering::SeqCst);
    f()
}

fn with_threshold<F, R>(min: i32, max: i32, f: F) -> R
where
    F: FnOnce() -> R,
{
    assert!(min <= max, "Invalid threshold range");
    f()
}

#[decorate(with_retry(3))]
fn simple_retry() -> i32 {
    42
}

#[decorate(with_threshold(0, 100), with_retry(2))]
fn complex_decoration(x: i32) -> i32 {
    x * 2
}

fn main() {
    assert_eq!(simple_retry(), 42);
    assert_eq!(RETRY_COUNT.load(Ordering::SeqCst), 3);

    assert_eq!(complex_decoration(21), 42);
}
