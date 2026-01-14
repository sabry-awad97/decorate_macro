//! Function call tracing decorator for debugging and observability.

use std::panic::Location;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tracing::{Level, info, span};

static CALL_ID: AtomicU64 = AtomicU64::new(0);

/// Traces function calls with entry/exit logging and unique call IDs.
///
/// # Features
/// - Unique call ID for correlating logs
/// - Entry and exit logging
/// - Execution time measurement
/// - Caller location tracking
///
/// # Example
///
/// ```rust,ignore
/// #[decorate(trace_calls)]
/// fn process_order(order_id: u64) -> Result<(), Error> {
///     // Logs: "→ Entering process_order [call_id=1]"
///     // ... execution ...
///     // Logs: "← Exiting process_order [call_id=1] (took 42ms)"
/// }
/// ```
#[track_caller]
pub fn trace_calls<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let call_id = CALL_ID.fetch_add(1, Ordering::Relaxed);
    let location = Location::caller();
    let file = location
        .file()
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(location.file());
    let line = location.line();

    let span = span!(
        Level::INFO,
        "fn_call",
        call_id = %call_id,
        file = %file,
        line = %line
    );
    let _guard = span.enter();

    info!(
        call_id = %call_id,
        "→ Entering function"
    );

    let start = Instant::now();
    let result = f();
    let elapsed = start.elapsed();

    info!(
        call_id = %call_id,
        duration_ms = %elapsed.as_millis(),
        "← Exiting function"
    );

    result
}

/// Traces function calls with a custom operation name.
///
/// # Arguments
/// * `operation` - Name to identify this operation in logs
/// * `f` - The function to execute
#[track_caller]
pub fn trace_calls_named<F, R>(operation: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let call_id = CALL_ID.fetch_add(1, Ordering::Relaxed);
    let location = Location::caller();

    let span = span!(
        Level::INFO,
        "operation",
        name = %operation,
        call_id = %call_id,
        file = %location.file(),
        line = %location.line()
    );
    let _guard = span.enter();

    info!(
        operation = %operation,
        call_id = %call_id,
        "→ Starting operation"
    );

    let start = Instant::now();
    let result = f();
    let elapsed = start.elapsed();

    info!(
        operation = %operation,
        call_id = %call_id,
        duration_ms = %elapsed.as_millis(),
        "← Completed operation"
    );

    result
}
