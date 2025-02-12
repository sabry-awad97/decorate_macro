// src/lib.rs
extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::Parse, punctuated::Punctuated, spanned::Spanned, Error, Expr, ItemFn, Path, Token,
};

// Parser for a single decorator with optional arguments
struct DecoratorCall {
    path: Path,
    args: Option<Punctuated<Expr, Token![,]>>,
}

impl Parse for DecoratorCall {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let path = input.parse()?;
        let args = if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            Some(Punctuated::parse_terminated(&content)?)
        } else {
            None
        };
        Ok(DecoratorCall { path, args })
    }
}

// Parser for comma-separated decorators
struct DecoratorList {
    decorators: Punctuated<DecoratorCall, Token![,]>,
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
///
/// Decorator with arguments:
/// ```rust
/// use decorate_macro::decorate;
///
/// fn with_retry<F, R>(attempts: u32, f: F) -> R
/// where
///     F: Fn() -> R,  // Changed from FnOnce to Fn
/// {
///     let mut last_error = None;
///     for _ in 0..attempts {
///         match std::panic::catch_unwind(std::panic::AssertUnwindSafe(&f)) {
///             Ok(result) => return result,
///             Err(e) => last_error = Some(e),
///         }
///     }
///     panic!("Failed after {} attempts", attempts)
/// }
///
/// #[decorate(with_retry(3))]
/// fn fallible_operation() -> i32 {
///     42
/// }
/// ```
#[proc_macro_attribute]
pub fn decorate(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the list of decorator calls
    let decorator_list = match syn::parse::<DecoratorList>(attr) {
        Ok(list) if list.decorators.is_empty() => {
            return TokenStream::from(
                create_error(
                    proc_macro2::Span::call_site(),
                    "No decorator paths provided",
                    Some("Expected at least one decorator function"),
                )
                .to_compile_error(),
            )
        }
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

    // Build nested decorator calls with arguments
    let mut decorated_body = quote! { #body };
    for decorator in decorator_list.decorators.iter().rev() {
        let path = &decorator.path;
        decorated_body = if let Some(args) = &decorator.args {
            quote! {
                #path(#args, || #decorated_body)
            }
        } else {
            quote! {
                #path(|| #decorated_body)
            }
        };
    }

    let output = quote! {
        #vis #sig {
            #decorated_body
        }
    };

    output.into()
}
