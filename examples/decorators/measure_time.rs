//! Performance measurement decorator with detailed metrics.

use std::panic::Location;
use std::time::Instant;
use tracing::{Level, info, warn};

/// Measures and logs execution time of a function.
///
/// # Features
/// - Automatic caller location detection
/// - Configurable warning threshold
/// - Structured logging with tracing
///
/// # Example
///
/// ```rust,ignore
/// #[decorate(measure_time)]
/// fn expensive_operation() -> Data {
///     // ...
/// }
/// ```
#[track_caller]
pub fn measure_time<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let location = Location::caller();
    let file = location
        .file()
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(location.file());
    let line = location.line();

    let start = Instant::now();
    let result = f();
    let elapsed = start.elapsed();

    // Warn if execution takes longer than 1 second
    if elapsed.as_secs() >= 1 {
        warn!(
            target: "perf",
            file = %file,
            line = %line,
            duration_ms = %elapsed.as_millis(),
            "⚠️  Slow execution: {:?}",
            elapsed
        );
    } else {
        info!(
            target: "perf",
            file = %file,
            line = %line,
            duration_us = %elapsed.as_micros(),
            "⏱️  Completed in {:?}",
            elapsed
        );
    }

    result
}

/// Measures execution time with a custom threshold for warnings.
///
/// # Arguments
/// * `threshold_ms` - Milliseconds threshold for warning logs
/// * `f` - The function to execute
#[track_caller]
pub fn measure_time_with_threshold<F, R>(threshold_ms: u64, f: F) -> R
where
    F: FnOnce() -> R,
{
    let location = Location::caller();
    let file = location
        .file()
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(location.file());

    let start = Instant::now();
    let result = f();
    let elapsed = start.elapsed();

    let level = if elapsed.as_millis() as u64 >= threshold_ms {
        Level::WARN
    } else {
        Level::INFO
    };

    match level {
        Level::WARN => warn!(
            target: "perf",
            file = %file,
            threshold_ms = %threshold_ms,
            actual_ms = %elapsed.as_millis(),
            "⚠️  Exceeded threshold: {:?}",
            elapsed
        ),
        _ => info!(
            target: "perf",
            file = %file,
            duration_us = %elapsed.as_micros(),
            "⏱️  Completed in {:?}",
            elapsed
        ),
    }

    result
}
