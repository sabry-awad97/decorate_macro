use decorate_macro::decorate;
use std::fmt::Debug;

fn test_decorator<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    println!("Decorating generic function");
    f()
}

// Test with a simple generic function
#[decorate(test_decorator)]
fn identity<T>(x: T) -> T {
    x
}

// Test with bounded generic parameters
#[decorate(test_decorator)]
fn print_and_return<T: Debug>(x: T) -> T {
    println!("Value: {:?}", x);
    x
}

// Test with multiple generic parameters and where clause
#[decorate(test_decorator)]
fn combine<T, U>(t: T, u: U) -> String
where
    T: Debug,
    U: Debug,
{
    format!("{:?}{:?}", t, u)
}

fn main() {
    // Test generic functions
    assert_eq!(identity(42), 42);
    assert_eq!(identity("hello"), "hello");

    print_and_return(123);
    print_and_return("test");

    let result = combine(42, " is the answer");
    assert_eq!(result, "42\" is the answer\"");
}
