use decorate_macro::decorate;
use std::cell::RefCell;

thread_local! {
    static METHOD_CALLS: RefCell<Vec<&'static str>> = RefCell::new(Vec::new());
}

fn trace_method<F, R>(method_name: &'static str, f: F) -> R
where
    F: FnOnce() -> R,
{
    METHOD_CALLS.with(|calls| calls.borrow_mut().push(method_name));
    let result = f();
    METHOD_CALLS.with(|calls| calls.borrow_mut().push("end"));
    result
}

struct TestStruct {
    value: i32,
}

impl TestStruct {
    fn new(value: i32) -> Self {
        Self { value }
    }

    #[decorate(trace_method("get"))]
    fn get_value(&self) -> i32 {
        self.value
    }

    #[decorate(trace_method("set"))]
    fn set_value(&mut self, new_value: i32) {
        self.value = new_value;
    }

    #[decorate(trace_method("compute"))]
    fn compute<T>(&self, factor: T) -> T
    where
        T: std::ops::Mul<Output = T> + From<i32>,
    {
        T::from(self.value) * factor
    }
}

fn main() {
    let mut test = TestStruct::new(42);

    assert_eq!(test.get_value(), 42);
    test.set_value(10);
    assert_eq!(test.compute(5), 50);

    METHOD_CALLS.with(|calls| {
        assert_eq!(
            &*calls.borrow(),
            &["get", "end", "set", "end", "compute", "end"]
        );
    });
}
