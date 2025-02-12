use decorate_macro::decorate;

fn test_decorator<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    println!("Before");
    let result = f();
    println!("After");
    result
}

#[test]
fn test_basic_decoration() {
    #[decorate(test_decorator)]
    fn add(x: i32, y: i32) -> i32 {
        x + y
    }

    assert_eq!(add(2, 3), 5);
}

#[test]
fn test_pub_decoration() {
    #[decorate(test_decorator)]
    pub fn multiply(x: i32, y: i32) -> i32 {
        x * y
    }

    assert_eq!(multiply(2, 3), 6);
}

#[test]
fn test_async_decoration() {
    #[decorate(test_decorator)]
    async fn async_fn(x: i32) -> i32 {
        x + 1
    }

    // Note: We're not actually running the async function since
    // that would require a runtime, but we verify it compiles
}

#[test]
fn test_generic_decoration() {
    #[decorate(test_decorator)]
    fn generic_fn<T: std::fmt::Display>(x: T) -> String {
        format!("Value: {}", x)
    }

    assert_eq!(generic_fn(42), "Value: 42");
    assert_eq!(generic_fn("hello"), "Value: hello");
}

#[test]
fn test_generic_with_where_clause() {
    #[decorate(test_decorator)]
    fn bounded_fn<T>(x: T) -> T
    where
        T: std::fmt::Debug + Clone,
    {
        println!("Debug: {:?}", x);
        x.clone()
    }

    let value = bounded_fn(vec![1, 2, 3]);
    assert_eq!(value, vec![1, 2, 3]);
}

#[test]
fn test_struct_method_decoration() {
    struct TestStruct(i32);

    impl TestStruct {
        #[decorate(test_decorator)]
        fn get_value(&self) -> i32 {
            self.0
        }
    }

    let test = TestStruct(42);
    assert_eq!(test.get_value(), 42);
}

#[test]
fn test_mut_method_decoration() {
    struct TestStruct(i32);

    impl TestStruct {
        #[decorate(test_decorator)]
        fn increment(&mut self) -> i32 {
            self.0 += 1;
            self.0
        }
    }

    let mut test = TestStruct(0);
    assert_eq!(test.increment(), 1);
    assert_eq!(test.increment(), 2);
}
