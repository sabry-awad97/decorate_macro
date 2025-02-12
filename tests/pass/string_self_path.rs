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

    fn log<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
        R: std::fmt::Debug,
    {
        println!("{}: Starting function", self.prefix);
        let result = f();
        println!("{}: Result = {:?}", self.prefix, result);
        result
    }

    // Add static logging method
    fn static_log<F, R>(prefix: &str, f: F) -> R
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
    logger: Logger,
    value: i32,
}

impl Counter {
    fn new() -> Self {
        Self {
            logger: Logger::new("Counter"),
            value: 0,
        }
    }

    // Using string literal for self path
    #[decorate("self.logger.log")]
    fn increment(&mut self) -> i32 {
        self.value += 1;
        self.value
    }

    // Multiple decorators with string literals
    #[decorate("self.logger.log", Logger::static_log("Static"))]
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
