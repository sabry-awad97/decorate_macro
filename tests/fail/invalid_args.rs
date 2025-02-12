use decorate_macro::decorate;

fn test_decorator<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    f()
}

#[decorate(test_decorator(invalid,))]
fn test_function(x: i32) -> i32 {
    x + 1
}

fn main() {}
