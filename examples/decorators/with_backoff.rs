//! Exponential backoff decorator for resilient operations.

use std::thread;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

/// Backoff strategy configuration.
#[derive(Debug, Clone)]
pub struct BackoffConfig {
    /// Maximum number of attempts
    pub max_attempts: u32,
    /// Initial delay before first retry
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Base for exponential calculation (typically 2)
    pub base: f64,
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(60),
            base: 2.0,
        }
    }
}

/// Implements exponential backoff retry logic for Result-returning functions.
///
/// Delay increases exponentially: initial_delay * base^attempt
///
/// # Arguments
/// * `max_attempts` - Maximum number of attempts
/// * `initial_delay` - Initial delay before first retry
/// * `f` - The function to execute
///
/// # Example
///
/// ```rust,ignore
/// #[decorate(with_backoff(5, Duration::from_millis(100)))]
/// fn call_external_service() -> Result<Response, Error> {
///     // Retries with delays: 100ms, 200ms, 400ms, 800ms, 1600ms
/// }
/// ```
pub fn with_backoff<F, R, E>(max_attempts: u32, initial_delay: Duration, f: F) -> Result<R, E>
where
    F: Fn() -> Result<R, E>,
    E: std::fmt::Debug,
{
    let config = BackoffConfig {
        max_attempts,
        initial_delay,
        ..Default::default()
    };
    with_backoff_config(&config, f)
}

/// Exponential backoff with full configuration control.
pub fn with_backoff_config<F, R, E>(config: &BackoffConfig, f: F) -> Result<R, E>
where
    F: Fn() -> Result<R, E>,
    E: std::fmt::Debug,
{
    let start = Instant::now();

    for attempt in 1..=config.max_attempts {
        match f() {
            Ok(result) => {
                if attempt > 1 {
                    info!(
                        attempt = %attempt,
                        elapsed_ms = %start.elapsed().as_millis(),
                        "✅ Succeeded after {} attempts with backoff",
                        attempt
                    );
                }
                return Ok(result);
            }
            Err(e) => {
                warn!(
                    attempt = %attempt,
                    max_attempts = %config.max_attempts,
                    error = ?e,
                    "❌ Attempt {}/{} failed",
                    attempt,
                    config.max_attempts
                );

                if attempt < config.max_attempts {
                    let delay = calculate_backoff_delay(
                        attempt,
                        config.initial_delay,
                        config.max_delay,
                        config.base,
                    );

                    info!(
                        delay_ms = %delay.as_millis(),
                        next_attempt = %(attempt + 1),
                        "⏳ Backing off for {:?}",
                        delay
                    );
                    thread::sleep(delay);
                } else {
                    error!(
                        attempts = %config.max_attempts,
                        elapsed_ms = %start.elapsed().as_millis(),
                        "❌ All {} attempts failed",
                        config.max_attempts
                    );
                    return Err(e);
                }
            }
        }
    }

    unreachable!()
}

/// Calculates the backoff delay for a given attempt.
fn calculate_backoff_delay(
    attempt: u32,
    initial_delay: Duration,
    max_delay: Duration,
    base: f64,
) -> Duration {
    let delay_secs = initial_delay.as_secs_f64() * base.powi(attempt as i32 - 1);
    Duration::from_secs_f64(delay_secs.min(max_delay.as_secs_f64()))
}

/// Decorrelated jitter backoff (AWS-style).
///
/// Uses the formula: sleep = min(cap, random_between(base, sleep * 3))
/// This provides better distribution than simple exponential backoff.
///
/// # Arguments
/// * `max_attempts` - Maximum number of attempts
/// * `base_delay` - Base delay for calculations
/// * `cap` - Maximum delay cap
/// * `f` - The function to execute
pub fn with_decorrelated_jitter<F, R, E>(
    max_attempts: u32,
    base_delay: Duration,
    cap: Duration,
    f: F,
) -> Result<R, E>
where
    F: Fn() -> Result<R, E>,
    E: std::fmt::Debug,
{
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};

    let start = Instant::now();
    let mut sleep = base_delay;

    for attempt in 1..=max_attempts {
        match f() {
            Ok(result) => {
                if attempt > 1 {
                    info!(
                        attempt = %attempt,
                        elapsed_ms = %start.elapsed().as_millis(),
                        "✅ Succeeded with decorrelated jitter backoff"
                    );
                }
                return Ok(result);
            }
            Err(e) => {
                warn!(
                    attempt = %attempt,
                    max_attempts = %max_attempts,
                    error = ?e,
                    "❌ Attempt {}/{} failed",
                    attempt,
                    max_attempts
                );

                if attempt < max_attempts {
                    // Generate pseudo-random value
                    let mut hasher = RandomState::new().build_hasher();
                    hasher.write_u64(attempt as u64 + start.elapsed().as_nanos() as u64);
                    let random = hasher.finish();

                    // Calculate next sleep: random between base and sleep * 3
                    let min_sleep = base_delay.as_secs_f64();
                    let max_sleep = (sleep.as_secs_f64() * 3.0).min(cap.as_secs_f64());
                    let range = max_sleep - min_sleep;
                    let jittered = min_sleep + (random as f64 / u64::MAX as f64) * range;

                    sleep = Duration::from_secs_f64(jittered.min(cap.as_secs_f64()));

                    info!(
                        delay_ms = %sleep.as_millis(),
                        "⏳ Decorrelated jitter delay"
                    );
                    thread::sleep(sleep);
                } else {
                    error!(
                        attempts = %max_attempts,
                        elapsed_ms = %start.elapsed().as_millis(),
                        "❌ All {} attempts failed",
                        max_attempts
                    );
                    return Err(e);
                }
            }
        }
    }

    unreachable!()
}
