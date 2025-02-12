use decorate_macro::decorate;

#[decorate(123)] // Invalid decorator path
fn test_function(x: i32) -> i32 {
    x
}

fn main() {}
