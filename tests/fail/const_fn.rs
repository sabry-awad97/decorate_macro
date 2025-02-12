use decorate_macro::decorate;

fn test_decorator<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    f()
}

#[decorate(test_decorator)]
const fn constant_fn(x: i32) -> i32 {
    x + 1
}

fn main() {}
