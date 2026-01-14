# Error Reference

This document lists all possible errors that can occur when using the `decorate` macro.

## Compilation Errors

### Invalid Decorator Format

```rust
#[decorate()]  // Error: no decorator paths provided
fn function() {}
```

**Solution**: Provide at least one decorator function.

### Const Function Decoration

```rust
#[decorate(log_execution)]
const fn constant() {}  // Error: cannot decorate const functions
```

**Solution**: Remove the `const` keyword or use a regular function.

### Invalid Parameter Transformation

```rust
fn wrong_transform(x: i32) -> i32 { x }

#[decorate(transform_params = wrong_transform)]
fn two_params(x: i32, y: i32) {}  // Error: Parameter count mismatch
```

**Solution**: Ensure transformer function accepts and returns the correct number of parameters.

### Wrong Decorator Signature

The macro now provides clear error messages when decorator signatures don't match:

```rust
// Decorator takes no closure parameter
fn bad_decorator() -> i32 { 42 }

#[decorate(bad_decorator)]  // Error: this function takes 0 arguments but 1 argument was supplied
fn test() -> i32 { 1 }
```

**Error points directly to `bad_decorator`** with a clear message about argument count.

### Wrong Return Type

```rust
fn bad_return<F>(f: F) -> String
where F: FnOnce() -> i32 {
    let _ = f();
    "wrong".to_string()
}

#[decorate(bad_return)]  // Error: mismatched types - expected `i32`, found `String`
fn test() -> i32 { 42 }
```

**Error points to the decorator** and shows the type mismatch clearly.

### Invalid Self Path

```rust
impl MyStruct {
    #[decorate("invalid.path")]  // Error: path must start with 'self'
    fn method(&self) {}
}
```

**Solution**: Self-referencing paths must start with `self`, e.g., `"self.logger.log"`.

## Runtime Errors

### Panic in Decorators

If a decorator panics, it will propagate unless explicitly caught:

```rust
fn panicking_decorator<F, R>(f: F) -> R
where
    F: FnOnce() -> R
{
    panic!("Decorator failed");
}
```

**Solution**: Use `catch_unwind` in decorators that might panic:

```rust
fn safe_decorator<F, R>(f: F) -> R
where
    F: FnOnce() -> R
{
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f())) {
        Ok(result) => result,
        Err(_) => panic!("Function execution failed"),
    }
}
```

## Best Practices

1. Always validate decorator inputs
2. Handle potential panics in decorators
3. Use type-safe transformations
4. Document decorator behavior
5. Test decorated functions thoroughly
6. Follow the expected decorator signature:
   ```rust
   fn decorator<F, R>(f: F) -> R
   where
       F: FnOnce() -> R,
   {
       f()
   }
   ```
