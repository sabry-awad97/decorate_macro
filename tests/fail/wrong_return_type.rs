use decorate_macro::decorate;

// This decorator returns the wrong type - returns String instead of the closure's return type
fn bad_return_decorator<F>(f: F) -> String
where
    F: FnOnce() -> i32,
{
    let _ = f();
    "wrong".to_string()
}

#[decorate(bad_return_decorator)]
fn test_function() -> i32 {
    42
}

fn main() {}
