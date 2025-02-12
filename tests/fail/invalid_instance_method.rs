use decorate_macro::decorate;

struct Test {
    value: i32,
}

impl Test {
    #[decorate(self.invalid..method)] // Invalid syntax
    fn test(&self) -> i32 {
        self.value
    }
}

fn main() {}
