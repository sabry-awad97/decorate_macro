use decorate_macro::decorate;

#[decorate()]
fn test_function(x: i32) -> i32 {
    x + 1
}

fn main() {}
