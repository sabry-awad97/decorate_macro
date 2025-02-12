# Decorate Macro

A powerful and flexible procedural macro for Rust that enables Python-style function decoration with advanced features like parameter transformation, result transformation, and execution hooks.

[![Crates.io](https://img.shields.io/crates/v/decorate_macro.svg)](https://crates.io/crates/decorate_macro)
[![Documentation](https://docs.rs/decorate_macro/badge.svg)](https://docs.rs/decorate_macro)
[![License](https://img.shields.io/crates/l/decorate_macro.svg)](LICENSE)

## Features

- 🚀 Function decoration with minimal boilerplate
- 🔄 Parameter and result transformation
- ⚡ Pre and post-execution hooks
- 🛡️ Support for async functions and methods
- 🧩 Multiple decorators with composition
- 🎯 Generic function support
- 🔒 Type-safe transformations

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
decorate_macro = "0.1.0"
```

## Quick Start

```rust
use decorate_macro::decorate;

// Basic logging decorator
fn log_execution<F, R>(f: F) -> R 
where 
    F: FnOnce() -> R,
{
    println!("Starting execution");
    let result = f();
    println!("Finished execution");
    result
}

// Apply decorator to function
#[decorate(log_execution)]
fn add(x: i32, y: i32) -> i32 {
    x + y
}

fn main() {
    assert_eq!(add(2, 3), 5);
}
```

## Advanced Usage

### Parameter Transformation

```rust
fn transform_params(x: i32, y: i32) -> (i32, i32) {
    (x + 1, y + 1)
}

#[decorate(
    transform_params = transform_params,
    log_execution
)]
fn compute(x: i32, y: i32) -> i32 {
    x + y
}
```

### Result Transformation

```rust
fn transform_result(x: i32) -> i32 {
    x * 2
}

#[decorate(
    transform_result = transform_result,
    log_execution
)]
fn compute(x: i32, y: i32) -> i32 {
    x + y
}
```

### Pre/Post Execution Hooks

```rust
#[decorate(
    pre = "println!(\"Starting computation\");",
    post = "println!(\"Computation finished\");",
    log_execution
)]
fn compute(x: i32, y: i32) -> i32 {
    x + y
}
```

### Multiple Decorators

```rust
#[decorate(validate, log_execution, retry(3))]
fn critical_operation() -> Result<(), Error> {
    // ... implementation ...
}
```

## Documentation

For detailed documentation and more examples, please visit:

- [API Documentation](https://docs.rs/decorate_macro)
- [Examples Directory](examples/)
- [Advanced Usage Guide](docs/advanced.md)

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for guidelines.

To contribute:

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -am 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Testing

Run the test suite:

```bash
cargo test
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by Python's decorator pattern
- Built with [syn](https://crates.io/crates/syn) and [quote](https://crates.io/crates/quote)
- Thanks to all contributors!
