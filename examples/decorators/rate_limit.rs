//! Rate limiting decorator to control execution frequency.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Rate limiter state for a single key.
#[derive(Debug)]
struct RateLimiterState {
    last_request: Instant,
    request_count: u64,
}

type RateLimiterMap = HashMap<String, RateLimiterState>;

static RATE_LIMITERS: LazyLock<Mutex<RateLimiterMap>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Rate limits function calls by enforcing a minimum delay between executions.
///
/// # Arguments
/// * `delay_ms` - Minimum milliseconds between executions
/// * `f` - The function to execute
///
/// # Behavior
/// If called before the delay has elapsed, the decorator will sleep until
/// the delay period has passed, then execute the function.
///
/// # Example
///
/// ```rust,ignore
/// #[decorate(rate_limit(1000))]  // Max 1 call per second
/// fn call_api() -> Response {
///     // ...
/// }
/// ```
pub fn rate_limit<F, R>(delay_ms: u64, f: F) -> R
where
    F: FnOnce() -> R,
{
    rate_limit_keyed("default", delay_ms, f)
}

/// Rate limits with a custom key for independent rate limiting.
///
/// Different keys maintain separate rate limit counters, allowing
/// fine-grained control over different operations.
///
/// # Arguments
/// * `key` - Unique identifier for this rate limit group
/// * `delay_ms` - Minimum milliseconds between executions
/// * `f` - The function to execute
///
/// # Example
///
/// ```rust,ignore
/// #[decorate(rate_limit_keyed("api_v1", 100))]
/// fn call_api_v1() -> Response { }
///
/// #[decorate(rate_limit_keyed("api_v2", 200))]
/// fn call_api_v2() -> Response { }
/// ```
pub fn rate_limit_keyed<F, R>(key: &str, delay_ms: u64, f: F) -> R
where
    F: FnOnce() -> R,
{
    let delay = Duration::from_millis(delay_ms);
    let now = Instant::now();

    let sleep_duration = {
        let mut limiters = RATE_LIMITERS
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let state = limiters.entry(key.to_string()).or_insert_with(|| {
            RateLimiterState {
                last_request: now - delay, // Allow immediate first request
                request_count: 0,
            }
        });

        let elapsed = now.duration_since(state.last_request);

        if elapsed < delay {
            let sleep_time = delay - elapsed;
            state.last_request = now + sleep_time;
            state.request_count += 1;
            Some(sleep_time)
        } else {
            state.last_request = now;
            state.request_count += 1;
            None
        }
    };

    if let Some(sleep_time) = sleep_duration {
        warn!(
            key = %key,
            sleep_ms = %sleep_time.as_millis(),
            "⏳ Rate limited - sleeping"
        );
        thread::sleep(sleep_time);
    } else {
        info!(key = %key, "✅ Rate limit passed");
    }

    f()
}

/// Token bucket rate limiter for burst-tolerant rate limiting.
///
/// Allows bursts up to `bucket_size` requests, then enforces the rate limit.
///
/// # Arguments
/// * `key` - Unique identifier for this rate limit group
/// * `tokens_per_second` - Rate at which tokens are replenished
/// * `bucket_size` - Maximum tokens (burst capacity)
/// * `f` - The function to execute
///
/// # Returns
/// `Some(R)` if a token was available, `None` if rate limited
pub fn rate_limit_token_bucket<F, R>(
    key: &str,
    tokens_per_second: f64,
    bucket_size: u32,
    f: F,
) -> Option<R>
where
    F: FnOnce() -> R,
{
    #[derive(Debug)]
    struct TokenBucket {
        tokens: f64,
        last_update: Instant,
        tokens_per_second: f64,
        bucket_size: u32,
    }

    static TOKEN_BUCKETS: LazyLock<Mutex<HashMap<String, TokenBucket>>> =
        LazyLock::new(|| Mutex::new(HashMap::new()));

    let mut buckets = TOKEN_BUCKETS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let now = Instant::now();
    let bucket = buckets
        .entry(key.to_string())
        .or_insert_with(|| TokenBucket {
            tokens: bucket_size as f64,
            last_update: now,
            tokens_per_second,
            bucket_size,
        });

    // Replenish tokens based on elapsed time
    let elapsed = now.duration_since(bucket.last_update).as_secs_f64();
    bucket.tokens =
        (bucket.tokens + elapsed * bucket.tokens_per_second).min(bucket.bucket_size as f64);
    bucket.last_update = now;

    if bucket.tokens >= 1.0 {
        bucket.tokens -= 1.0;
        info!(
            key = %key,
            remaining_tokens = %bucket.tokens as u32,
            "✅ Token acquired"
        );
        drop(buckets);
        Some(f())
    } else {
        warn!(
            key = %key,
            tokens = %bucket.tokens,
            "🚫 Rate limited - no tokens available"
        );
        None
    }
}

/// Gets rate limit statistics for a key.
pub fn get_rate_limit_stats(key: &str) -> Option<u64> {
    RATE_LIMITERS
        .lock()
        .ok()
        .and_then(|limiters| limiters.get(key).map(|s| s.request_count))
}

/// Resets rate limit state for a key.
pub fn reset_rate_limit(key: &str) {
    if let Ok(mut limiters) = RATE_LIMITERS.lock() {
        limiters.remove(key);
        info!(key = %key, "🔄 Rate limit reset");
    }
}
