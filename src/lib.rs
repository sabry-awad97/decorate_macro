// src/lib.rs
extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, ItemFn, Path};

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
#[proc_macro_attribute]
pub fn decorate(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse inputs or return error messages
    let decorator_path = match syn::parse::<Path>(attr) {
        Ok(path) => path,
        Err(e) => return TokenStream::from(e.to_compile_error()),
    };

    let input_fn = match syn::parse::<ItemFn>(item) {
        Ok(f) => f,
        Err(e) => return TokenStream::from(e.to_compile_error()),
    };

    // Validate function attributes
    if !input_fn.sig.generics.params.is_empty() {
        return TokenStream::from(
            Error::new_spanned(
                input_fn.sig.generics,
                "Decorated functions cannot have generic parameters",
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
