# Advanced Usage Guide

## Table of Contents

- [Parameter Transformation](#parameter-transformation)
- [Result Transformation](#result-transformation)
- [Execution Hooks](#execution-hooks)
- [Multiple Decorators](#multiple-decorators)
- [Generic Functions](#generic-functions)
- [Struct Methods](#struct-methods)

## Parameter Transformation

Transform function parameters before execution:

```rust
fn validate_params(x: i32, y: i32) -> (i32, i32) {
    (x.max(0), y.max(0))  // Ensure non-negative inputs
}

#[decorate(transform_params = validate_params)]
fn divide(x: i32, y: i32) -> f64 {
    x as f64 / y as f64
}
```

## Result Transformation

Transform the function's return value:

```rust
fn round_result(x: f64) -> f64 {
    (x * 100.0).round() / 100.0  // Round to 2 decimal places
}

#[decorate(transform_result = round_result)]
fn calculate_pi() -> f64 {
    std::f64::consts::PI
}
```

## Execution Hooks

Add pre and post execution code:

```rust
#[decorate(
    pre = "log::info!(\"Starting computation\");",
    post = "log::info!(\"Computation finished\");",
)]
fn expensive_operation() -> Result<(), Error> {
    // ... implementation ...
}
```

## Multiple Decorators

Combine multiple decorators:

```rust
#[decorate(
    transform_params = validate_params,
    transform_result = round_result,
    retry(3),
    log_execution
)]
fn complex_calculation(x: i32, y: i32) -> f64 {
    // ... implementation ...
}
```

## Generic Functions

Decorate generic functions:

```rust
fn log_type<T: std::fmt::Debug>(x: T) -> T {
    println!("Value: {:?}", x);
    x
}

#[decorate(transform_params = log_type)]
fn process<T: std::fmt::Debug>(value: T) -> T {
    // ... implementation ...
}
```

## Struct Methods

Decorate struct methods:

```rust
impl Calculator {
    #[decorate(
        pre = "self.validate();",
        post = "self.update_history();",
        transform_result = round_result
    )]
    pub fn compute(&self, x: f64) -> f64 {
        // ... implementation ...
    }
}
```

For more examples, check the [examples directory](../examples/).
