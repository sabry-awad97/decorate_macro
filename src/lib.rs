// src/lib.rs
extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Error, ItemFn, Path};

// Helper function to create decorated error messages
fn create_error(span: proc_macro2::Span, message: &str, help: Option<&str>) -> Error {
    let mut err = Error::new(span, message);
    if let Some(help_msg) = help {
        err.combine(Error::new(span, help_msg));
    }
    err
}

/// Decorates a function with a wrapper that provides additional functionality.
///
/// # Arguments
///
/// * `decorator_path` - Path to the decorator function that will wrap the original function
///
/// # Returns
///
/// * `TokenStream` - The modified function implementation
///
/// # Example
///
/// ```rust
/// use decorate_macro::decorate;
///
/// fn log_execution<F, R>(f: F) -> R
/// where
///     F: FnOnce() -> R,
/// {
///     println!("Starting execution");
///     let result = f();
///     println!("Finished execution");
///     result
/// }
///
/// #[decorate(log_execution)]
/// fn add(x: i32, y: i32) -> i32 {
///     x + y
/// }
/// ```
///
/// # Example with generics
///
/// ```rust
/// use decorate_macro::decorate;
///
/// fn log_execution<F, R>(f: F) -> R
/// where
///     F: FnOnce() -> R,
/// {
///     println!("Starting execution");
///     let result = f();
///     println!("Finished execution");
///     result
/// }
///
/// #[decorate(log_execution)]
/// fn identity<T: std::fmt::Debug>(x: T) -> T {
///     println!("Value: {:?}", x);
///     x
/// }
/// ```
#[proc_macro_attribute]
pub fn decorate(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse decorator path with improved error message
    let decorator_path = match syn::parse::<Path>(attr.clone()) {
        Ok(path) => path,
        Err(_) => {
            return TokenStream::from(
                create_error(
                    proc_macro2::Span::call_site(),
                    "Invalid decorator path",
                    Some("Expected a function path, e.g., #[decorate(my_decorator)]"),
                )
                .to_compile_error(),
            )
        }
    };

    // Parse function with detailed error
    let input_fn = match syn::parse::<ItemFn>(item.clone()) {
        Ok(f) => f,
        Err(_) => {
            return TokenStream::from(
                create_error(
                    proc_macro2::Span::call_site(),
                    "Invalid function definition",
                    Some("The decorate attribute can only be applied to functions"),
                )
                .to_compile_error(),
            )
        }
    };

    // Validate decorator path exists in scope
    if decorator_path.segments.is_empty() {
        return TokenStream::from(
            create_error(
                decorator_path.span(),
                "Empty decorator path",
                Some("Specify a valid decorator function name"),
            )
            .to_compile_error(),
        );
    }

    // Validate function signature
    if input_fn.sig.constness.is_some() {
        return TokenStream::from(
            create_error(
                input_fn.sig.constness.span(),
                "Cannot decorate const functions",
                Some("The decorate attribute cannot be used with const functions"),
            )
            .to_compile_error(),
        );
    }

    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let body = &input_fn.block;

    // Generate the decorated function
    let output = quote! {
        #vis #sig {
            #decorator_path(|| #body)
        }
    };

    output.into()
}
