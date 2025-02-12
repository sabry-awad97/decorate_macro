use decorate_macro::decorate;

fn test_decorator<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    f()
}

#[decorate(test_decorator)]
fn generic_function<T>(x: T) -> T {
    x
}

fn main() {}
