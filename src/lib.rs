//! A procedural macro for Python-style function decoration in Rust.
//!
//! This crate provides the `#[decorate]` attribute macro that allows wrapping
//! functions with decorator functions, supporting features like parameter
//! transformation, result transformation, and pre/post execution hooks.

extern crate proc_macro;

use either::Either;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::{
    Error, Expr, FnArg, Ident, ItemFn, Pat, Path, Result, Token, parse::Parse,
    punctuated::Punctuated, spanned::Spanned,
};

// ============================================================================
// Error Messages (centralized for consistency and i18n potential)
// ============================================================================

mod error_messages {
    pub const NO_DECORATORS: &str = "no decorator paths provided";
    pub const CONST_FN_NOT_SUPPORTED: &str = "cannot decorate const functions";
    pub const CONST_FN_HELP: &str = "remove the `const` keyword or use a regular function";
    pub const SELF_PATH_MUST_START_WITH_SELF: &str = "path must start with 'self'";
    pub const SELF_PATH_EMPTY_SEGMENT: &str = "path contains empty segment";
    pub const SELF_PATH_INVALID_SEGMENT: &str = "path segment must be a valid identifier";
    pub const UNKNOWN_CONFIG_OPTION: &str = "unknown configuration option";
    pub const UNKNOWN_CONFIG_HELP: &str =
        "valid options are: pre, post, transform_params, transform_result";
}

// ============================================================================
// Configuration for decorator behavior
// ============================================================================

/// Configuration options that modify how a decorator is applied.
///
/// These options allow fine-grained control over the decoration process,
/// including parameter/result transformation and execution hooks.
#[derive(Default)]
struct DecoratorConfig {
    /// Code to execute before the decorated function body.
    pre_code: Option<Expr>,
    /// Code to execute after the decorated function body.
    post_code: Option<Expr>,
    /// Function to transform parameters before execution.
    transform_params: Option<Path>,
    /// Function to transform the result after execution.
    transform_result: Option<Path>,
}

impl DecoratorConfig {
    /// Returns `true` if any configuration option is set.
    #[inline]
    fn has_any(&self) -> bool {
        self.pre_code.is_some()
            || self.post_code.is_some()
            || self.transform_params.is_some()
            || self.transform_result.is_some()
    }
}

// ============================================================================
// Decorator Call Parser
// ============================================================================

/// Represents a single decorator invocation with optional configuration and arguments.
///
/// A decorator call can be:
/// - A simple path: `my_decorator`
/// - A path with arguments: `my_decorator(arg1, arg2)`
/// - A string path for self references: `"self.field.decorator"`
/// - Any of the above with configuration: `transform_params = foo, my_decorator`
struct DecoratorCall {
    /// Optional configuration for this decorator.
    config: Option<DecoratorConfig>,
    /// The decorator path - either a regular path or a self-referencing expression.
    path: Either<Path, Expr>,
    /// Optional arguments to pass to the decorator.
    args: Option<Punctuated<Expr, Token![,]>>,
}

impl Parse for DecoratorCall {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let mut config = DecoratorConfig::default();

        // Parse configuration options (key = value pairs)
        while input.peek(Ident) && input.peek2(Token![=]) {
            let key: Ident = input.parse()?;
            let key_span = key.span();
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "pre" => config.pre_code = Some(input.parse()?),
                "post" => config.post_code = Some(input.parse()?),
                "transform_params" => config.transform_params = Some(input.parse()?),
                "transform_result" => config.transform_result = Some(input.parse()?),
                _ => {
                    return Err(create_error_with_help(
                        key_span,
                        error_messages::UNKNOWN_CONFIG_OPTION,
                        error_messages::UNKNOWN_CONFIG_HELP,
                    ));
                }
            }

            // Consume optional trailing comma
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        // Parse decorator path (either string literal for self-paths or regular path)
        let path = if input.peek(syn::LitStr) {
            let path_str: syn::LitStr = input.parse()?;
            Either::Right(parse_self_path(&path_str.value(), path_str.span())?)
        } else {
            Either::Left(input.parse()?)
        };

        // Parse optional arguments in parentheses
        let args = if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            Some(Punctuated::parse_terminated(&content)?)
        } else {
            None
        };

        Ok(DecoratorCall {
            config: if config.has_any() { Some(config) } else { None },
            path,
            args,
        })
    }
}

// ============================================================================
// Decorator List Parser
// ============================================================================

/// A comma-separated list of decorator calls.
struct DecoratorList {
    decorators: Punctuated<DecoratorCall, Token![,]>,
}

impl Parse for DecoratorList {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        Ok(DecoratorList {
            decorators: Punctuated::parse_terminated(input)?,
        })
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Creates an error with an additional help message.
fn create_error_with_help(span: Span, message: &str, help: &str) -> Error {
    let mut err = Error::new(span, message);
    err.combine(Error::new(span, format!("help: {}", help)));
    err
}

/// Validates that a string is a valid Rust identifier.
fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();

    // First character must be alphabetic or underscore
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' => {}
        _ => return false,
    }

    // Remaining characters must be alphanumeric or underscore
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// Parses a self-referencing path string into an expression.
///
/// # Arguments
/// * `s` - The path string, e.g., "self.field.method"
/// * `span` - The span to use for error reporting
///
/// # Returns
/// A `syn::Expr` representing the parsed path.
///
/// # Errors
/// Returns an error if:
/// - The path doesn't start with "self"
/// - The path contains empty segments
/// - Any segment is not a valid identifier
fn parse_self_path(s: &str, span: Span) -> Result<Expr> {
    let segments: Vec<&str> = s.split('.').collect();

    // Validate path starts with "self"
    if segments.is_empty() || segments[0] != "self" {
        return Err(Error::new(
            span,
            error_messages::SELF_PATH_MUST_START_WITH_SELF,
        ));
    }

    // Validate all segments are non-empty and valid identifiers
    for (i, segment) in segments.iter().enumerate() {
        if segment.is_empty() {
            return Err(Error::new(span, error_messages::SELF_PATH_EMPTY_SEGMENT));
        }
        // Skip "self" validation as it's a keyword
        if i > 0 && !is_valid_identifier(segment) {
            return Err(Error::new(
                span,
                format!(
                    "{}: '{}'",
                    error_messages::SELF_PATH_INVALID_SEGMENT,
                    segment
                ),
            ));
        }
    }

    // Build the expression using syn's AST directly (safer than string parsing)
    let self_ident = Ident::new("self", span);
    let mut expr: Expr = syn::parse_quote_spanned!(span=> #self_ident);

    for segment in segments.iter().skip(1) {
        let field_ident = Ident::new(segment, span);
        expr = syn::parse_quote_spanned!(span=> (#expr).#field_ident);
    }

    Ok(expr)
}

/// Extracts parameter names from function arguments.
///
/// Only extracts simple identifier patterns; complex patterns (tuples, etc.)
/// are skipped with a warning in the generated code.
fn extract_param_names(inputs: &Punctuated<FnArg, Token![,]>) -> Vec<&Ident> {
    inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Typed(pat_type) => {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    Some(&pat_ident.ident)
                } else {
                    // Complex patterns (tuples, etc.) are not supported for transformation
                    None
                }
            }
            FnArg::Receiver(_) => None, // Skip self parameters
        })
        .collect()
}

/// Generates the decorated function body by applying all decorators in reverse order.
fn generate_decorated_body(
    decorators: &Punctuated<DecoratorCall, Token![,]>,
    original_body: &syn::Block,
    fn_inputs: &Punctuated<FnArg, Token![,]>,
    is_async: bool,
) -> proc_macro2::TokenStream {
    let mut decorated_body = quote! { #original_body };

    // Apply decorators in reverse order (innermost first)
    for decorator in decorators.iter().rev() {
        // Apply configuration transformations
        if let Some(config) = &decorator.config {
            decorated_body = apply_config_transformations(config, decorated_body, fn_inputs);
        }

        // Wrap with decorator call
        let decorator_expr = match &decorator.path {
            Either::Left(path) => quote!(#path),
            Either::Right(expr) => quote!(#expr),
        };

        decorated_body =
            generate_decorator_wrapper(&decorator_expr, &decorator.args, decorated_body, is_async);
    }

    decorated_body
}

/// Applies configuration transformations (pre/post code, param/result transforms).
fn apply_config_transformations(
    config: &DecoratorConfig,
    mut body: proc_macro2::TokenStream,
    fn_inputs: &Punctuated<FnArg, Token![,]>,
) -> proc_macro2::TokenStream {
    // Apply parameter transformation
    if let Some(transform) = &config.transform_params {
        let param_names = extract_param_names(fn_inputs);
        if !param_names.is_empty() {
            body = quote! {
                {
                    let (#(#param_names),*) = #transform(#(#param_names),*);
                    #body
                }
            };
        }
    }

    // Apply pre-execution code
    if let Some(pre) = &config.pre_code {
        body = quote! {
            {
                #pre;
                #body
            }
        };
    }

    // Apply post-execution code
    if let Some(post) = &config.post_code {
        body = quote! {
            {
                let __decorate_result = #body;
                #post;
                __decorate_result
            }
        };
    }

    // Apply result transformation
    if let Some(transform) = &config.transform_result {
        body = quote! {
            {
                let __decorate_result = #body;
                #transform(__decorate_result)
            }
        };
    }

    body
}

/// Generates the decorator wrapper call.
fn generate_decorator_wrapper(
    decorator_expr: &proc_macro2::TokenStream,
    args: &Option<Punctuated<Expr, Token![,]>>,
    body: proc_macro2::TokenStream,
    is_async: bool,
) -> proc_macro2::TokenStream {
    if is_async {
        if let Some(args) = args {
            quote! {
                #decorator_expr(#args, || async { #body })
            }
        } else {
            quote! {
                #decorator_expr(|| async { #body })
            }
        }
    } else if let Some(args) = args {
        quote! {
            #decorator_expr(#args, || #body)
        }
    } else {
        quote! {
            #decorator_expr(|| #body)
        }
    }
}

// ============================================================================
// Main Macro Implementation
// ============================================================================

/// Decorates a function with one or more wrappers that provide additional functionality.
///
/// # Arguments
///
/// * `decorator_paths` - Comma-separated list of decorator function paths
///
/// # Configuration Options
///
/// * `pre = <expr>` - Code to execute before the function body
/// * `post = <expr>` - Code to execute after the function body
/// * `transform_params = <path>` - Function to transform parameters
/// * `transform_result = <path>` - Function to transform the result
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
    // Parse the decorator list
    let decorator_list = match syn::parse::<DecoratorList>(attr) {
        Ok(list) if list.decorators.is_empty() => {
            return Error::new(Span::call_site(), error_messages::NO_DECORATORS)
                .to_compile_error()
                .into();
        }
        Ok(list) => list,
        Err(e) => return e.to_compile_error().into(),
    };

    // Parse the function
    let input_fn = match syn::parse::<ItemFn>(item) {
        Ok(f) => f,
        Err(e) => return e.to_compile_error().into(),
    };

    // Validate: const functions cannot be decorated
    if let Some(const_token) = &input_fn.sig.constness {
        return create_error_with_help(
            const_token.span(),
            error_messages::CONST_FN_NOT_SUPPORTED,
            error_messages::CONST_FN_HELP,
        )
        .to_compile_error()
        .into();
    }

    let is_async = input_fn.sig.asyncness.is_some();
    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let body = &input_fn.block;
    let attrs = &input_fn.attrs;

    // Generate the decorated body
    let decorated_body =
        generate_decorated_body(&decorator_list.decorators, body, &sig.inputs, is_async);

    // Generate the final output
    let output = if is_async {
        quote_spanned! {sig.span()=>
            #(#attrs)*
            #vis #sig {
                #decorated_body.await
            }
        }
    } else {
        quote_spanned! {sig.span()=>
            #(#attrs)*
            #vis #sig {
                #decorated_body
            }
        }
    };

    output.into()
}
