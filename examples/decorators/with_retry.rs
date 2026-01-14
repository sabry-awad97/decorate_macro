//! Retry decorator with configurable strategies.

use std::thread;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

/// Retry configuration options.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for exponential backoff (1.0 = constant delay)
    pub backoff_multiplier: f64,
    /// Whether to add jitter to delays
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryConfig {
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            ..Default::default()
        }
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    pub fn with_max_delay(mut self, max_delay: Duration) -> Self {
        self.max_delay = max_delay;
        self
    }

    pub fn with_backoff(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    pub fn without_jitter(mut self) -> Self {
        self.jitter = false;
        self
    }
}

/// Retries a function on panic with configurable attempts.
///
/// # Arguments
/// * `attempts` - Maximum number of attempts
/// * `f` - The function to execute (must be `Fn` for multiple calls)
///
/// # Panics
/// Panics if all attempts fail, with the last error.
///
/// # Example
///
/// ```rust,ignore
/// #[decorate(with_retry(3))]
/// fn unreliable_operation() -> Data {
///     // Will retry up to 3 times on panic
/// }
/// ```
pub fn with_retry<F, R>(attempts: u32, f: F) -> R
where
    F: Fn() -> R,
{
    let config = RetryConfig::new(attempts);
    with_retry_config(&config, f)
}

/// Retries with full configuration control.
///
/// # Arguments
/// * `config` - Retry configuration
/// * `f` - The function to execute
pub fn with_retry_config<F, R>(config: &RetryConfig, f: F) -> R
where
    F: Fn() -> R,
{
    let start = Instant::now();
    let mut delay = config.initial_delay;
    let mut last_error = None;

    for attempt in 1..=config.max_attempts {
        info!(
            attempt = %attempt,
            max_attempts = %config.max_attempts,
            "🔄 Attempt {}/{}",
            attempt,
            config.max_attempts
        );

        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(&f)) {
            Ok(result) => {
                if attempt > 1 {
                    info!(
                        attempt = %attempt,
                        elapsed_ms = %start.elapsed().as_millis(),
                        "✅ Succeeded after {} attempts",
                        attempt
                    );
                }
                return result;
            }
            Err(e) => {
                warn!(
                    attempt = %attempt,
                    max_attempts = %config.max_attempts,
                    error = ?e,
                    "❌ Attempt {} failed",
                    attempt
                );
                last_error = Some(e);

                if attempt < config.max_attempts {
                    let actual_delay = if config.jitter {
                        add_jitter(delay)
                    } else {
                        delay
                    };

                    info!(
                        delay_ms = %actual_delay.as_millis(),
                        "⏳ Waiting before retry"
                    );
                    thread::sleep(actual_delay);

                    // Apply backoff
                    delay = Duration::from_secs_f64(
                        (delay.as_secs_f64() * config.backoff_multiplier)
                            .min(config.max_delay.as_secs_f64()),
                    );
                }
            }
        }
    }

    error!(
        attempts = %config.max_attempts,
        elapsed_ms = %start.elapsed().as_millis(),
        "❌ All {} attempts failed",
        config.max_attempts
    );

    panic!(
        "Failed after {} attempts ({:.2?}). Last error: {:?}",
        config.max_attempts,
        start.elapsed(),
        last_error
    );
}

/// Retries a Result-returning function.
///
/// # Arguments
/// * `attempts` - Maximum number of attempts
/// * `f` - The function to execute
///
/// # Returns
/// `Ok(R)` on success, `Err(E)` with the last error if all attempts fail
pub fn with_retry_result<F, R, E>(attempts: u32, f: F) -> Result<R, E>
where
    F: Fn() -> Result<R, E>,
    E: std::fmt::Debug,
{
    let config = RetryConfig::new(attempts);
    with_retry_result_config(&config, f)
}

/// Retries a Result-returning function with full configuration.
pub fn with_retry_result_config<F, R, E>(config: &RetryConfig, f: F) -> Result<R, E>
where
    F: Fn() -> Result<R, E>,
    E: std::fmt::Debug,
{
    let start = Instant::now();
    let mut delay = config.initial_delay;
    let mut last_error = None;

    for attempt in 1..=config.max_attempts {
        info!(
            attempt = %attempt,
            max_attempts = %config.max_attempts,
            "🔄 Attempt {}/{}",
            attempt,
            config.max_attempts
        );

        match f() {
            Ok(result) => {
                if attempt > 1 {
                    info!(
                        attempt = %attempt,
                        elapsed_ms = %start.elapsed().as_millis(),
                        "✅ Succeeded after {} attempts",
                        attempt
                    );
                }
                return Ok(result);
            }
            Err(e) => {
                warn!(
                    attempt = %attempt,
                    error = ?e,
                    "❌ Attempt {} failed",
                    attempt
                );
                last_error = Some(e);

                if attempt < config.max_attempts {
                    let actual_delay = if config.jitter {
                        add_jitter(delay)
                    } else {
                        delay
                    };

                    info!(delay_ms = %actual_delay.as_millis(), "⏳ Waiting before retry");
                    thread::sleep(actual_delay);

                    delay = Duration::from_secs_f64(
                        (delay.as_secs_f64() * config.backoff_multiplier)
                            .min(config.max_delay.as_secs_f64()),
                    );
                }
            }
        }
    }

    error!(
        attempts = %config.max_attempts,
        elapsed_ms = %start.elapsed().as_millis(),
        "❌ All {} attempts failed",
        config.max_attempts
    );

    Err(last_error.unwrap())
}

/// Adds random jitter to a duration (±25%).
fn add_jitter(duration: Duration) -> Duration {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};

    // Simple pseudo-random jitter using hasher
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u64(duration.as_nanos() as u64);
    let random = hasher.finish();

    let jitter_factor = 0.75 + (random % 500) as f64 / 1000.0; // 0.75 to 1.25
    Duration::from_secs_f64(duration.as_secs_f64() * jitter_factor)
}
