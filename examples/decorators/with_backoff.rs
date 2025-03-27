use std::thread;
use std::time::Duration;
use tracing::{error, info, warn};

/// Implements exponential backoff retry logic
pub fn with_backoff<F, R, E>(max_attempts: u32, initial_delay: Duration, f: F) -> Result<R, E>
where
    F: Fn() -> Result<R, E>,
    E: std::fmt::Debug,
{
    let mut delay = initial_delay;

    for attempt in 1..=max_attempts {
        match f() {
            Ok(result) => {
                if attempt > 1 {
                    info!("✅ Succeeded after {} attempts", attempt);
                }
                return Ok(result);
            }
            Err(e) => {
                warn!("❌ Attempt {}/{} failed: {:?}", attempt, max_attempts, e);
                if attempt < max_attempts {
                    info!("⏳ Waiting {:?} before next attempt", delay);
                    thread::sleep(delay);
                    delay *= 2; // Exponential backoff
                } else {
                    error!("❌ All {} attempts failed", max_attempts);
                    return Err(e);
                }
            }
        }
    }
    unreachable!()
}
