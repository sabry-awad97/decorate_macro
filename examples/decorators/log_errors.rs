//! Error logging decorator for Result-returning functions.

use std::panic::Location;
use tracing::{error, info, warn};

/// Logs errors from Result-returning functions without modifying the result.
///
/// # Features
/// - Logs errors at ERROR level with context
/// - Logs success at DEBUG level
/// - Preserves the original Result unchanged
/// - Includes caller location
///
/// # Example
///
/// ```rust,ignore
/// #[decorate(log_errors)]
/// fn fetch_data(id: u64) -> Result<Data, DbError> {
///     // Errors are automatically logged with context
/// }
/// ```
#[track_caller]
pub fn log_errors<F, R, E>(f: F) -> Result<R, E>
where
    F: FnOnce() -> Result<R, E>,
    E: std::fmt::Debug,
{
    let location = Location::caller();
    let file = location
        .file()
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(location.file());
    let line = location.line();

    let result = f();

    match &result {
        Ok(_) => {
            info!(
                file = %file,
                line = %line,
                "✅ Operation succeeded"
            );
        }
        Err(e) => {
            error!(
                file = %file,
                line = %line,
                error = ?e,
                "❌ Operation failed"
            );
        }
    }

    result
}

/// Logs errors with a custom operation name for better context.
///
/// # Arguments
/// * `operation` - Name to identify this operation in logs
/// * `f` - The function to execute
#[track_caller]
pub fn log_errors_named<F, R, E>(operation: &str, f: F) -> Result<R, E>
where
    F: FnOnce() -> Result<R, E>,
    E: std::fmt::Debug,
{
    let location = Location::caller();

    let result = f();

    match &result {
        Ok(_) => {
            info!(
                operation = %operation,
                file = %location.file(),
                line = %location.line(),
                "✅ {} succeeded", operation
            );
        }
        Err(e) => {
            error!(
                operation = %operation,
                file = %location.file(),
                line = %location.line(),
                error = ?e,
                "❌ {} failed", operation
            );
        }
    }

    result
}

/// Logs errors and converts them to a different error type.
///
/// Useful for adding context when propagating errors up the call stack.
///
/// # Arguments
/// * `context` - Additional context to include in the error
/// * `f` - The function to execute
#[track_caller]
pub fn log_errors_with_context<F, R, E>(context: &str, f: F) -> Result<R, String>
where
    F: FnOnce() -> Result<R, E>,
    E: std::fmt::Debug + std::fmt::Display,
{
    let location = Location::caller();

    match f() {
        Ok(value) => {
            info!(
                context = %context,
                "✅ Operation succeeded"
            );
            Ok(value)
        }
        Err(e) => {
            let error_msg = format!("{}: {}", context, e);
            error!(
                context = %context,
                file = %location.file(),
                line = %location.line(),
                original_error = %e,
                "❌ {}", error_msg
            );
            Err(error_msg)
        }
    }
}

/// Logs warnings for recoverable errors, errors for fatal ones.
///
/// # Arguments
/// * `is_recoverable` - Function to determine if an error is recoverable
/// * `f` - The function to execute
#[track_caller]
pub fn log_errors_classified<F, R, E, C>(is_recoverable: C, f: F) -> Result<R, E>
where
    F: FnOnce() -> Result<R, E>,
    E: std::fmt::Debug,
    C: Fn(&E) -> bool,
{
    let location = Location::caller();
    let file = location
        .file()
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(location.file());

    let result = f();

    if let Err(ref e) = result {
        if is_recoverable(e) {
            warn!(
                file = %file,
                line = %location.line(),
                error = ?e,
                "⚠️ Recoverable error occurred"
            );
        } else {
            error!(
                file = %file,
                line = %location.line(),
                error = ?e,
                "❌ Fatal error occurred"
            );
        }
    }

    result
}
