use decorate_macro::decorate;

fn test_decorator<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    f()
}

#[decorate(test_decorator)]
fn normal_function(x: i32) -> i32 {
    x + 1
}

fn main() {
    assert_eq!(normal_function(5), 6);
}
