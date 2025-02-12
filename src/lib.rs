// src/lib.rs
extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, punctuated::Punctuated, spanned::Spanned, Error, ItemFn, Path, Token};

// Add parser for comma-separated decorator paths
struct DecoratorList {
    decorators: Punctuated<Path, Token![,]>,
}

impl Parse for DecoratorList {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(DecoratorList {
            decorators: Punctuated::parse_terminated(input)?,
        })
    }
}

// Helper function to create decorated error messages
fn create_error(span: proc_macro2::Span, message: &str, help: Option<&str>) -> Error {
    let mut err = Error::new(span, message);
    if let Some(help_msg) = help {
        err.combine(Error::new(span, help_msg));
    }
    err
}

/// Decorates a function with one or more wrappers that provide additional functionality.
///
/// # Arguments
///
/// * `decorator_paths` - Comma-separated list of decorator function paths
///
/// # Examples
///
/// Basic usage:
/// ```rust
/// use decorate_macro::decorate;
///
/// fn log_execution<F, R>(f: F) -> R where F: FnOnce() -> R {
///     println!("Starting");
///     let result = f();
///     println!("Ending");
///     result
/// }
///
/// #[decorate(log_execution)]
/// fn add(x: i32, y: i32) -> i32 {
///     x + y
/// }
/// ```
///
/// Multiple decorators:
/// ```rust
/// use decorate_macro::decorate;
///
/// fn validate<F, R>(f: F) -> R where F: FnOnce() -> R {
///     println!("Validating...");
///     f()
/// }
///
/// fn log_result<F, R: std::fmt::Debug>(f: F) -> R where F: FnOnce() -> R {
///     let result = f();
///     println!("Result: {:?}", result);
///     result
/// }
///
/// #[decorate(validate, log_result)]
/// fn multiply(x: i32, y: i32) -> i32 {
///     x * y
/// }
/// ```
///
/// Generic functions:
/// ```rust
/// use decorate_macro::decorate;
///
/// fn trace<F, R: std::fmt::Debug>(f: F) -> R where F: FnOnce() -> R {
///     println!("Entering function");
///     let result = f();
///     println!("Returning: {:?}", result);
///     result
/// }
///
/// #[decorate(trace)]
/// fn identity<T: std::fmt::Debug>(x: T) -> T {
///     x
/// }
/// ```
#[proc_macro_attribute]
pub fn decorate(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the list of decorator paths
    let decorator_list =
        match syn::parse::<DecoratorList>(attr) {
            Ok(list) if list.decorators.is_empty() => return TokenStream::from(
                create_error(
                    proc_macro2::Span::call_site(),
                    "No decorator paths provided",
                    Some(
                        "Expected at least one decorator function, e.g., #[decorate(my_decorator)]",
                    ),
                )
                .to_compile_error(),
            ),
            Ok(list) => list,
            Err(e) => return TokenStream::from(e.to_compile_error()),
        };

    let input_fn = match syn::parse::<ItemFn>(item) {
        Ok(f) => f,
        Err(e) => return TokenStream::from(e.to_compile_error()),
    };

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

    // Build nested decorator calls
    let mut decorated_body = quote! { #body };
    for decorator in decorator_list.decorators.iter().rev() {
        decorated_body = quote! {
            #decorator(|| #decorated_body)
        };
    }

    // Generate the decorated function
    let output = quote! {
        #vis #sig {
            #decorated_body
        }
    };

    output.into()
}
