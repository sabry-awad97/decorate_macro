use decorate_macro::decorate;
use std::sync::atomic::{AtomicUsize, Ordering};

static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

fn transform_params(x: i32, y: i32) -> (i32, i32) {
    (x + 1, y + 1)
}

fn double_params(x: i32, y: i32) -> (i32, i32) {
    (x * 2, y * 2)
}

fn transform_result(x: i32) -> i32 {
    x * 2
}

fn add_one_result(x: i32) -> i32 {
    x + 1
}

fn log_execution<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    CALL_COUNT.fetch_add(1, Ordering::SeqCst);
    println!("Executing function");
    let result = f();
    println!("Function complete");
    result
}

#[decorate(
    transform_params = transform_params,
    transform_result = transform_result,
    log_execution
)]
fn compute(x: i32, y: i32) -> i32 {
    x + y
}

fn main() {
    let result = compute(1, 2);
    // (1+1) + (2+1) = 5
    // 5 * 2 = 10
    assert_eq!(result, 10);
}

#[test]
fn test_transform_params() {
    #[decorate(
        transform_params = transform_params,
        transform_result = transform_result,
        log_execution
    )]
    fn compute(x: i32, y: i32) -> i32 {
        x + y
    }

    let result = compute(1, 2);
    assert_eq!(result, 10); // ((1+1) + (2+1)) * 2 = 10
}

#[test]
fn test_param_transform() {
    #[decorate(
        transform_params = transform_params,
        log_execution
    )]
    fn add(x: i32, y: i32) -> i32 {
        x + y
    }

    assert_eq!(add(1, 2), 5); // (1+1) + (2+1) = 5
}

#[test]
fn test_result_transform() {
    #[decorate(
        transform_result = transform_result,
        log_execution
    )]
    fn add(x: i32, y: i32) -> i32 {
        x + y
    }

    assert_eq!(add(1, 2), 6); // (1 + 2) * 2 = 6
}

#[test]
fn test_both_transforms() {
    #[decorate(
        transform_params = double_params,
        transform_result = add_one_result,
        log_execution
    )]
    fn add(x: i32, y: i32) -> i32 {
        x + y
    }

    assert_eq!(add(1, 2), 7); // ((1*2) + (2*2)) + 1 = 7
}

#[test]
fn test_multiple_decorators() {
    #[decorate(
        transform_params = transform_params,
        transform_result = transform_result,
        log_execution
    )]
    fn add(x: i32, y: i32) -> i32 {
        x + y
    }

    CALL_COUNT.store(0, Ordering::SeqCst);
    let result = add(1, 2);
    assert_eq!(result, 10); // ((1+1) + (2+1)) * 2 = 10
    assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 1);
}

#[test]
fn test_transform_order() {
    #[decorate(
        pre = "println!(\"Starting\");",
        transform_params = transform_params,
        transform_result = transform_result,
        post = "println!(\"Finished\");",
        log_execution
    )]
    fn add(x: i32, y: i32) -> i32 {
        x + y
    }

    assert_eq!(add(1, 2), 10); // ((1+1) + (2+1)) * 2 = 10
}
