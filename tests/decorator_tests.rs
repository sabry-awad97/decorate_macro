use decorate_macro::decorate;

fn test_decorator<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    println!("Before");
    let result = f();
    println!("After");
    result
}

#[test]
fn test_basic_decoration() {
    #[decorate(test_decorator)]
    fn add(x: i32, y: i32) -> i32 {
        x + y
    }

    assert_eq!(add(2, 3), 5);
}

#[test]
fn test_pub_decoration() {
    #[decorate(test_decorator)]
    pub fn multiply(x: i32, y: i32) -> i32 {
        x * y
    }

    assert_eq!(multiply(2, 3), 6);
}

#[test]
fn test_async_decoration() {
    #[decorate(test_decorator)]
    async fn async_fn(x: i32) -> i32 {
        x + 1
    }

    // Note: We're not actually running the async function since
    // that would require a runtime, but we verify it compiles
}
