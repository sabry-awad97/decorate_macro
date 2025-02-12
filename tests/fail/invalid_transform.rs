use decorate_macro::decorate;

fn wrong_params(x: i32) -> i32 {
    x + 1
}

#[decorate(transform_params = wrong_params)]
fn add(x: i32, y: i32) -> i32 {
    x + y
}

fn main() {}
