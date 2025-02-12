use decorate_macro::decorate;
use log::{info, warn};

/// A decorator that logs function execution and handles potential panics
fn safe_decorator<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    info!("Starting function execution");
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    match result {
        Ok(value) => {
            info!("Function completed successfully");
            value
        }
        Err(e) => {
            warn!("Function panicked: {:?}", e);
            panic!("Function execution failed");
        }
    }
}

/// Example function using the decorator
#[decorate(safe_decorator)]
fn calculate_value(x: i32) -> i32 {
    if x < 0 {
        panic!("Input must be non-negative");
    }
    x * 2
}

fn main() {
    // Initialize logging
    env_logger::init();

    // Example usage
    match std::panic::catch_unwind(|| calculate_value(5)) {
        Ok(value) => println!("Result: {}", value),
        Err(_) => eprintln!("Calculation failed"),
    }
}
