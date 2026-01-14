//! Panic-safe decorator for graceful error handling.

use std::any::Any;
use std::panic::{self, AssertUnwindSafe};
use tracing::{error, info, warn};

/// Result type for panic-safe operations.
#[derive(Debug)]
pub enum SafeResult<T> {
    /// Operation completed successfully
    Ok(T),
    /// Operation panicked
    Panicked(String),
}

impl<T> SafeResult<T> {
    pub fn is_ok(&self) -> bool {
        matches!(self, SafeResult::Ok(_))
    }

    pub fn is_panicked(&self) -> bool {
        matches!(self, SafeResult::Panicked(_))
    }

    pub fn unwrap(self) -> T {
        match self {
            SafeResult::Ok(v) => v,
            SafeResult::Panicked(msg) => panic!("Called unwrap on panicked result: {}", msg),
        }
    }

    pub fn unwrap_or(self, default: T) -> T {
        match self {
            SafeResult::Ok(v) => v,
            SafeResult::Panicked(_) => default,
        }
    }

    pub fn ok(self) -> Option<T> {
        match self {
            SafeResult::Ok(v) => Some(v),
            SafeResult::Panicked(_) => None,
        }
    }
}

/// Catches panics and converts them to a SafeResult.
///
/// # Example
///
/// ```rust,ignore
/// #[decorate(safe_decorator)]
/// fn risky_operation() -> SafeResult<Data> {
///     // Panics are caught and converted to SafeResult::Panicked
/// }
/// ```
pub fn safe_decorator<F, R>(f: F) -> SafeResult<R>
where
    F: FnOnce() -> R,
{
    info!("🛡️ Executing in panic-safe context");

    match panic::catch_unwind(AssertUnwindSafe(f)) {
        Ok(value) => {
            info!("✅ Operation completed successfully");
            SafeResult::Ok(value)
        }
        Err(e) => {
            let panic_msg = extract_panic_message(&e);
            error!(
                panic_message = %panic_msg,
                "❌ Operation panicked"
            );
            SafeResult::Panicked(panic_msg)
        }
    }
}

/// Catches panics and re-panics with a custom message.
///
/// Useful for adding context to panics without losing the original error.
pub fn safe_with_context<F, R>(context: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    info!(context = %context, "🛡️ Executing with panic context");

    match panic::catch_unwind(AssertUnwindSafe(f)) {
        Ok(value) => {
            info!(context = %context, "✅ Operation completed");
            value
        }
        Err(e) => {
            let panic_msg = extract_panic_message(&e);
            error!(
                context = %context,
                panic_message = %panic_msg,
                "❌ Operation panicked"
            );
            panic!("{}: {}", context, panic_msg);
        }
    }
}

/// Catches panics and returns a Result instead.
///
/// # Example
///
/// ```rust,ignore
/// #[decorate(safe_to_result)]
/// fn risky() -> Result<Data, String> {
///     // Panics become Err(String)
/// }
/// ```
pub fn safe_to_result<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce() -> R,
{
    info!("🛡️ Executing with panic-to-result conversion");

    match panic::catch_unwind(AssertUnwindSafe(f)) {
        Ok(value) => {
            info!("✅ Operation completed successfully");
            Ok(value)
        }
        Err(e) => {
            let panic_msg = extract_panic_message(&e);
            error!(panic_message = %panic_msg, "❌ Operation panicked");
            Err(panic_msg)
        }
    }
}

/// Catches panics and returns a default value.
///
/// # Arguments
/// * `default` - Value to return if the function panics
/// * `f` - The function to execute
pub fn safe_with_default<F, R>(default: R, f: F) -> R
where
    F: FnOnce() -> R,
{
    info!("🛡️ Executing with default fallback");

    match panic::catch_unwind(AssertUnwindSafe(f)) {
        Ok(value) => {
            info!("✅ Operation completed successfully");
            value
        }
        Err(e) => {
            let panic_msg = extract_panic_message(&e);
            warn!(
                panic_message = %panic_msg,
                "⚠️ Operation panicked, returning default"
            );
            default
        }
    }
}

/// Catches panics and calls a fallback function.
///
/// # Arguments
/// * `fallback` - Function to call if the main function panics
/// * `f` - The function to execute
pub fn safe_with_fallback<F, G, R>(fallback: G, f: F) -> R
where
    F: FnOnce() -> R,
    G: FnOnce(String) -> R,
{
    info!("🛡️ Executing with fallback handler");

    match panic::catch_unwind(AssertUnwindSafe(f)) {
        Ok(value) => {
            info!("✅ Operation completed successfully");
            value
        }
        Err(e) => {
            let panic_msg = extract_panic_message(&e);
            warn!(
                panic_message = %panic_msg,
                "⚠️ Operation panicked, calling fallback"
            );
            fallback(panic_msg)
        }
    }
}

/// Extracts a human-readable message from a panic payload.
fn extract_panic_message(payload: &Box<dyn Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "Unknown panic".to_string()
    }
}

/// Sets a custom panic hook that logs panics with tracing.
pub fn install_panic_logger() {
    panic::set_hook(Box::new(|info| {
        let payload = info.payload();
        let msg = if let Some(s) = payload.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()));

        error!(
            message = %msg,
            location = ?location,
            "🔥 PANIC"
        );
    }));
}
