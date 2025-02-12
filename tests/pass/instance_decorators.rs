use decorate_macro::decorate;

struct Logger {
    prefix: String,
}

impl Logger {
    fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
        }
    }

    // Static decorator function that takes logger as first argument
    fn with_logging<F, R>(prefix: &str, f: F) -> R
    where
        F: FnOnce() -> R,
        R: std::fmt::Debug,
    {
        println!("{}: Starting function", prefix);
        let result = f();
        println!("{}: Result = {:?}", prefix, result);
        result
    }
}

struct Counter {
    value: i32,
}

impl Counter {
    fn new() -> Self {
        Self { value: 0 }
    }

    // Using static decorator method
    #[decorate(Logger::with_logging("Counter"))]
    fn increment(&mut self) -> i32 {
        self.value += 1;
        self.value
    }

    // Using static decorator method
    #[decorate(Logger::with_logging("Counter"))]
    fn get_value(&self) -> i32 {
        self.value
    }
}

fn main() {
    let mut counter = Counter::new();
    assert_eq!(counter.increment(), 1);
    assert_eq!(counter.get_value(), 1);
    assert_eq!(counter.increment(), 2);
}
