# Examples

## Basic Decorators

### Logging Decorator

```rust
use decorate_macro::decorate;
use log::{info, warn};

fn log_execution<F, R>(f: F) -> R 
where 
    F: FnOnce() -> R 
{
    info!("Starting execution");
    let result = f();
    info!("Finished execution");
    result
}

#[decorate(log_execution)]
fn add(x: i32, y: i32) -> i32 {
    x + y
}
```

### Timing Decorator

```rust
use std::time::Instant;

fn measure_time<F, R>(f: F) -> R 
where 
    F: FnOnce() -> R 
{
    let start = Instant::now();
    let result = f();
    println!("Execution took: {:?}", start.elapsed());
    result
}

#[decorate(measure_time)]
fn expensive_calculation() -> u64 {
    // ... implementation ...
}
```

## Advanced Examples

### Retry Mechanism

```rust
fn with_retry<F, R>(attempts: u32, f: F) -> R 
where 
    F: Fn() -> R 
{
    let mut last_error = None;
    for _ in 0..attempts {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(&f)) {
            Ok(result) => return result,
            Err(e) => last_error = Some(e),
        }
    }
    panic!("Failed after {} attempts", attempts)
}

#[decorate(with_retry(3))]
fn fallible_operation() -> Result<(), Error> {
    // ... implementation ...
}
```

### Caching Results

```rust
use std::sync::Mutex;
use std::collections::HashMap;
use std::hash::Hash;

fn memoize<F, K, V>(cache: &Mutex<HashMap<K, V>>, key: K, f: F) -> V 
where 
    F: FnOnce() -> V,
    K: Hash + Eq,
    V: Clone,
{
    let mut cache = cache.lock().unwrap();
    if let Some(value) = cache.get(&key) {
        return value.clone();
    }
    
    let result = f();
    cache.insert(key, result.clone());
    result
}
```

For more examples, check the [tests directory](../tests/).
