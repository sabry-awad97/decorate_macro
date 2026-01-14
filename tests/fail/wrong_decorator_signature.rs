use decorate_macro::decorate;

// This decorator has the wrong signature - it takes no closure parameter
fn bad_decorator() -> i32 {
    42
}

#[decorate(bad_decorator)]
fn test_function() -> i32 {
    1
}

fn main() {}
