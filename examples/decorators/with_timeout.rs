//! Timeout decorator for bounding execution time.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tracing::{error, info};

/// Error type for timeout operations.
#[derive(Debug, Clone)]
pub enum TimeoutError<E> {
    /// The operation timed out
    Timeout { duration: Duration },
    /// The operation failed with an error
    Inner(E),
}

impl<E: std::fmt::Display> std::fmt::Display for TimeoutError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeoutError::Timeout { duration } => {
                write!(f, "Operation timed out after {:?}", duration)
            }
            TimeoutError::Inner(e) => write!(f, "{}", e),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for TimeoutError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TimeoutError::Inner(e) => Some(e),
            _ => None,
        }
    }
}

/// Executes a function with a timeout.
///
/// # Arguments
/// * `timeout_ms` - Maximum execution time in milliseconds
/// * `f` - The function to execute
///
/// # Returns
/// `Ok(R)` if completed within timeout, `Err(TimeoutError::Timeout)` otherwise
///
/// # Note
/// This spawns a new thread for the operation. For async code, use async timeout utilities.
///
/// # Example
///
/// ```rust,ignore
/// #[decorate(with_timeout(5000))]
/// fn slow_operation() -> Result<Data, TimeoutError<Error>> {
///     // Must complete within 5 seconds
/// }
/// ```
pub fn with_timeout<F, R>(timeout_ms: u64, f: F) -> Result<R, TimeoutError<String>>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let timeout = Duration::from_millis(timeout_ms);
    let (tx, rx) = mpsc::channel();

    info!(timeout_ms = %timeout_ms, "⏳ Starting operation with timeout");

    let handle = thread::spawn(move || {
        let result = f();
        let _ = tx.send(result);
    });

    match rx.recv_timeout(timeout) {
        Ok(result) => {
            info!("✅ Operation completed within timeout");
            // Wait for thread to finish (it should be done already)
            let _ = handle.join();
            Ok(result)
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            error!(
                timeout_ms = %timeout_ms,
                "⏰ Operation timed out"
            );
            // Note: The thread will continue running in the background
            // In production, consider using a cancellation mechanism
            Err(TimeoutError::Timeout { duration: timeout })
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            error!("❌ Operation thread panicked");
            Err(TimeoutError::Inner("Thread panicked".to_string()))
        }
    }
}

/// Executes a fallible function with a timeout.
///
/// # Arguments
/// * `timeout_ms` - Maximum execution time in milliseconds
/// * `f` - The function to execute
pub fn with_timeout_result<F, R, E>(timeout_ms: u64, f: F) -> Result<R, TimeoutError<E>>
where
    F: FnOnce() -> Result<R, E> + Send + 'static,
    R: Send + 'static,
    E: Send + 'static,
{
    let timeout = Duration::from_millis(timeout_ms);
    let (tx, rx) = mpsc::channel();

    info!(timeout_ms = %timeout_ms, "⏳ Starting fallible operation with timeout");

    let handle = thread::spawn(move || {
        let result = f();
        let _ = tx.send(result);
    });

    match rx.recv_timeout(timeout) {
        Ok(Ok(result)) => {
            info!("✅ Operation succeeded within timeout");
            let _ = handle.join();
            Ok(result)
        }
        Ok(Err(e)) => {
            info!("❌ Operation failed within timeout");
            let _ = handle.join();
            Err(TimeoutError::Inner(e))
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            error!(timeout_ms = %timeout_ms, "⏰ Operation timed out");
            Err(TimeoutError::Timeout { duration: timeout })
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            error!("❌ Operation thread panicked");
            // This is a bit awkward - we need an E but the thread panicked
            // In practice, you'd want a more sophisticated error type
            panic!("Operation thread panicked")
        }
    }
}
