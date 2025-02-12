use decorate_macro::decorate;

#[decorate({invalid_option = "test"}, log_execution)]
fn test_function() -> i32 {
    42
}

fn main() {}
