use decorate_macro::decorate;
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};

static EXECUTION_COUNT: AtomicUsize = AtomicUsize::new(0);

// Basic async decorator
async fn log_async<F, Fut, R>(f: F) -> R
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = R>,
{
    EXECUTION_COUNT.fetch_add(1, Ordering::SeqCst);
    let result = f().await;
    result
}

// Test simple async decoration
#[decorate(log_async)]
async fn simple_async() -> i32 {
    42
}

#[test]
fn test_async_functions() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        assert_eq!(simple_async().await, 42);
        assert_eq!(EXECUTION_COUNT.load(Ordering::SeqCst), 1);
    });
}

fn main() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let _ = simple_async().await;
    });
}
