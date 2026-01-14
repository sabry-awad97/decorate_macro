//! Debounce decorator to prevent rapid repeated calls.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};
use tracing::{info, warn};

type DebounceMap = HashMap<String, Instant>;

static DEBOUNCE_STATE: LazyLock<Mutex<DebounceMap>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// Debounces function calls, preventing execution if called too frequently.
///
/// Unlike rate limiting which delays execution, debouncing skips the call entirely
/// if it's within the debounce window.
///
/// # Arguments
/// * `key` - Unique identifier for this debounce group
/// * `window_ms` - Minimum milliseconds between executions
/// * `f` - The function to execute
///
/// # Returns
/// `Some(R)` if executed, `None` if debounced
///
/// # Example
///
/// ```rust,ignore
/// #[decorate(debounce("save", 1000))]
/// fn auto_save() -> Option<()> {
///     // Only executes if 1 second has passed since last call
/// }
/// ```
pub fn debounce<F, R>(key: &str, window_ms: u64, f: F) -> Option<R>
where
    F: FnOnce() -> R,
{
    let window = Duration::from_millis(window_ms);
    let now = Instant::now();

    let mut state = DEBOUNCE_STATE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if let Some(last_call) = state.get(key) {
        let elapsed = now.duration_since(*last_call);
        if elapsed < window {
            warn!(
                key = %key,
                remaining_ms = %(window - elapsed).as_millis(),
                "🚫 Debounced - too soon since last call"
            );
            return None;
        }
    }

    state.insert(key.to_string(), now);
    drop(state); // Release lock before execution

    info!(key = %key, "✅ Executing debounced function");
    Some(f())
}

/// Debounces with a default value returned when debounced.
///
/// # Arguments
/// * `key` - Unique identifier for this debounce group
/// * `window_ms` - Minimum milliseconds between executions
/// * `default` - Value to return if debounced
/// * `f` - The function to execute
pub fn debounce_with_default<F, R>(key: &str, window_ms: u64, default: R, f: F) -> R
where
    F: FnOnce() -> R,
{
    debounce(key, window_ms, f).unwrap_or(default)
}

/// Resets the debounce state for a key, allowing immediate execution.
pub fn reset_debounce(key: &str) {
    if let Ok(mut state) = DEBOUNCE_STATE.lock() {
        state.remove(key);
        info!(key = %key, "🔄 Debounce state reset");
    }
}

/// Clears all debounce state.
pub fn clear_all_debounce() {
    if let Ok(mut state) = DEBOUNCE_STATE.lock() {
        state.clear();
        info!("🔄 All debounce state cleared");
    }
}
