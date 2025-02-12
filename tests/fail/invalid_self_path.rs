use decorate_macro::decorate;

struct Test {
    value: i32,
}

impl Test {
    #[decorate("invalid.path")] // Should fail - doesn't start with 'self'
    fn test(&self) -> i32 {
        self.value
    }
}

fn main() {}
