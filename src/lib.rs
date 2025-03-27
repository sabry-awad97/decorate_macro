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

// Define a configuration structure
struct Config {
    pre_code: Option<syn::Expr>,
    post_code: Option<syn::Expr>,
    transform_params: Option<syn::Path>,
    transform_result: Option<syn::Path>,
}

// Parser for a single decorator with optional arguments
struct DecoratorCall {
    config: Option<Config>,
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
        let mut config = Config {
            pre_code: None,
            post_code: None,
            transform_params: None,
            transform_result: None,
        };

        // Parse config options if present
        while input.peek(syn::Ident) && input.peek2(Token![=]) {
            let key: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "pre" => config.pre_code = Some(input.parse()?),
                "post" => config.post_code = Some(input.parse()?),
                "transform_params" => config.transform_params = Some(input.parse()?),
                "transform_result" => config.transform_result = Some(input.parse()?),
                _ => return Err(Error::new(key.span(), "Unknown config option")),
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        // Parse decorator path or string
        let path = if input.peek(syn::LitStr) {
            let path_str: syn::LitStr = input.parse()?;
            Either::Right(parse_self_path(&path_str.value())?)
        } else {
            Either::Left(input.parse()?)
        };

        // Parse optional arguments
        let args = if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            Some(Punctuated::parse_terminated(&content)?)
        } else {
            None
        };

        Ok(DecoratorCall {
            config: if config.pre_code.is_some()
                || config.post_code.is_some()
                || config.transform_params.is_some()
                || config.transform_result.is_some()
            {
                Some(config)
            } else {
                None
            },
            path,
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

    // Check for const functions first
    if input_fn.sig.constness.is_some() {
        let const_span = input_fn.sig.constness.span();
        let mut error = Error::new(const_span, "Cannot decorate const functions");
        error.combine(Error::new(
            const_span,
            "The decorate attribute cannot be used with const functions",
        ));
        return TokenStream::from(error.to_compile_error());
    }

    // Check if the function is async
    let is_async = input_fn.sig.asyncness.is_some();

    // Remove the validation check since we handle parameter transformation
    // directly in the code generation phase

    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let body = &input_fn.block;

    // Build nested decorator calls with arguments
    let mut decorated_body = quote! { #body };

    // If the function is async, we need to box the future
    if is_async {
        decorated_body = quote! {
            Box::pin(async move { #decorated_body })
        };
    }

    for decorator in decorator_list.decorators.iter().rev() {
        if let Some(config) = &decorator.config {
            // Add parameter transformation
            if let Some(transform) = &config.transform_params {
                let param_names: Vec<_> = input_fn
                    .sig
                    .inputs
                    .iter()
                    .filter_map(|arg| match arg {
                        syn::FnArg::Typed(pat_type) => {
                            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                                Some(&pat_ident.ident)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    })
                    .collect();

                if !param_names.is_empty() {
                    decorated_body = quote! {
                        {
                            let (#(#param_names),*) = #transform(#(#param_names),*);
                            #decorated_body
                        }
                    };
                }
            }

            // Add pre-code
            if let Some(pre) = &config.pre_code {
                decorated_body = quote! {
                    {
                        #pre;
                        #decorated_body
                    }
                };
            }

            // Add post-code
            if let Some(post) = &config.post_code {
                decorated_body = quote! {
                    {
                        let result = #decorated_body;
                        #post;
                        result
                    }
                };
            }

            // Add result transformation
            if let Some(transform) = &config.transform_result {
                decorated_body = quote! {
                    {
                        let result = #decorated_body;
                        #transform(result)
                    }
                };
            }
        }

        let decorator_expr = match &decorator.path {
            Either::Left(path) => quote!(#path),
            Either::Right(expr) => quote!(#expr),
        };

        decorated_body = if is_async {
            if let Some(args) = &decorator.args {
                quote! { #decorator_expr(#args, || #decorated_body).await }
            } else {
                quote! { #decorator_expr(|| #decorated_body).await }
            }
        } else if let Some(args) = &decorator.args {
            quote! { #decorator_expr(#args, || #decorated_body) }
        } else {
            quote! { #decorator_expr(|| #decorated_body) }
        };
    }

    let output = if is_async {
        quote! {
            #vis #sig {
                use std::future::Future;
                use std::pin::Pin;
                use std::boxed::Box;
                #decorated_body
            }
        }
    } else {
        quote! {
            #vis #sig {
                #decorated_body
            }
        }
    };

    output.into()
}
