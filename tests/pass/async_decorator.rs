use decorate_macro::decorate;
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};

static EXECUTION_COUNT: AtomicUsize = AtomicUsize::new(0);

// Async-aware decorator that works with async functions
// The decorator receives a closure that returns a Future
fn log_call<F, Fut, R>(f: F) -> impl Future<Output = R>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = R>,
{
    EXECUTION_COUNT.fetch_add(1, Ordering::SeqCst);
    println!("Executing decorated async function");
    async move {
        let result = f().await;
        println!("Async function completed");
        result
    }
}

// Test simple async decoration
#[decorate(log_call)]
async fn simple_async() -> i32 {
    42
}

// Test async function with await inside
#[decorate(log_call)]
async fn async_with_await() -> String {
    let value = async { "hello" }.await;
    format!("{} world", value)
}

#[test]
fn test_async_functions() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        EXECUTION_COUNT.store(0, Ordering::SeqCst);

        assert_eq!(simple_async().await, 42);
        assert_eq!(EXECUTION_COUNT.load(Ordering::SeqCst), 1);

        assert_eq!(async_with_await().await, "hello world");
        assert_eq!(EXECUTION_COUNT.load(Ordering::SeqCst), 2);
    });
}

fn main() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let _ = simple_async().await;
        let _ = async_with_await().await;
    });
}
