use std::thread;
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Enhanced retry decorator with logging and timing
pub fn with_retry<F, R>(attempts: u32, f: F) -> R
where
    F: Fn() -> R,
{
    let start = Instant::now();
    let mut last_error = None;

    for attempt in 1..=attempts {
        info!("🔄 Attempt {}/{}", attempt, attempts);
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(&f)) {
            Ok(result) => {
                info!("✅ Attempt {} succeeded ({:.2?})", attempt, start.elapsed());
                return result;
            }
            Err(e) => {
                warn!("❌ Attempt {}/{} failed: {:?}", attempt, attempts, e);
                last_error = Some(e);
                if attempt < attempts {
                    let delay = Duration::from_millis(100 * attempt as u64);
                    info!("⏳ Waiting {:.2?} before next attempt", delay);
                    thread::sleep(delay);
                }
            }
        }
    }

    panic!(
        "❌ Failed after {} attempts ({:.2?}). Last error: {:?}",
        attempts,
        start.elapsed(),
        last_error
    );
}
