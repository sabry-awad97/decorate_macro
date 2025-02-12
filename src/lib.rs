// src/lib.rs
extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::Parse, punctuated::Punctuated, spanned::Spanned, Error, Expr, ItemFn, Path, Token,
};

// Parser for self path from string literal
fn parse_self_path(s: &str) -> syn::Result<syn::Expr> {
    let segments: Vec<&str> = s.split('.').collect();
    if segments.is_empty() || segments[0] != "self" {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            "Path must start with 'self'",
        ));
    }

    let mut expr = syn::parse_str::<syn::Expr>("self")?;
    for segment in segments.iter().skip(1) {
        let expr_tokens = expr.to_token_stream().to_string();
        expr = syn::parse_str(&format!("({}).{}", expr_tokens, segment))?;
    }
    Ok(expr)
}

// Parser for a single decorator with optional arguments
struct DecoratorCall {
    path: Either<Path, syn::Expr>,
    args: Option<Punctuated<Expr, Token![,]>>,
}

// Update Either enum
enum Either<A, B> {
    Left(A),
    Right(B),
}

impl Parse for DecoratorCall {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::LitStr) {
            let path_str: syn::LitStr = input.parse()?;
            let path_expr = parse_self_path(&path_str.value())?;

            let args = if input.peek(syn::token::Paren) {
                let content;
                syn::parenthesized!(content in input);
                Some(Punctuated::parse_terminated(&content)?)
            } else {
                None
            };

            return Ok(DecoratorCall {
                path: Either::Right(path_expr),
                args,
            });
        }

        let path = input.parse()?;
        let args = if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            Some(Punctuated::parse_terminated(&content)?)
        } else {
            None
        };
        Ok(DecoratorCall {
            path: Either::Left(path),
            args,
        })
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
///
/// Using with struct methods:
/// ```rust
/// use decorate_macro::decorate;
///
/// fn log_access<F, R>(f: F) -> R
/// where
///     F: FnOnce() -> R,
/// {
///     println!("Accessing method");
///     let result = f();
///     println!("Access complete");
///     result
/// }
///
/// struct Counter {
///     value: i32,
/// }
///
/// impl Counter {
///     #[decorate(log_access)]
///     pub fn increment(&mut self) -> i32 {
///         self.value += 1;
///         self.value
///     }
///
///     #[decorate(log_access)]
///     pub fn get_value(&self) -> i32 {
///         self.value
///     }
/// }
/// ```
///
/// With multiple decorators on methods:
/// ```rust
/// use decorate_macro::decorate;
/// use std::time::Instant;
///
/// fn validate_positive<F, R>(f: F) -> R
/// where
///     F: FnOnce() -> R,
///     R: PartialOrd + Default,
/// {
///     let result = f();
///     if result > R::default() {
///         result
///     } else {
///         R::default()
///     }
/// }
///
/// fn measure_time<F, R>(f: F) -> R
/// where
///     F: FnOnce() -> R,
/// {
///     let start = Instant::now();
///     let result = f();
///     println!("Execution took: {:?}", start.elapsed());
///     result
/// }
///
/// struct Calculator {
///     base: f64,
/// }
///
/// impl Calculator {
///     #[decorate(validate_positive, measure_time)]
///     pub fn compute(&self, factor: f64) -> f64 {
///         self.base * factor
///     }
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
        let decorator_expr = match &decorator.path {
            Either::Left(path) => quote!(#path),
            Either::Right(expr) => quote!(#expr),
        };

        decorated_body = if let Some(args) = &decorator.args {
            quote! {
                #decorator_expr(#args, || #decorated_body)
            }
        } else {
            quote! {
                #decorator_expr(|| #decorated_body)
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
