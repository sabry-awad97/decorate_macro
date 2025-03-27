use std::sync::LazyLock;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use tracing::info;

static LAST_REQUEST: LazyLock<Mutex<Instant>> = LazyLock::new(|| Mutex::new(Instant::now()));

/// Rate limiting decorator with mutex poison recovery
pub fn rate_limit<F, R>(delay_ms: u64, f: F) -> R
where
    F: FnOnce() -> R,
{
    {
        let mut last = LAST_REQUEST
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let elapsed = last.elapsed();
        let delay = Duration::from_millis(delay_ms);

        if elapsed < delay {
            let sleep_duration = delay - elapsed;
            info!("⏳ Rate limit: sleeping for {:.2?}", sleep_duration);
            *last = Instant::now() + sleep_duration;
            drop(last);
            thread::sleep(sleep_duration);
        } else {
            *last = Instant::now();
        }
    }
    f()
}
