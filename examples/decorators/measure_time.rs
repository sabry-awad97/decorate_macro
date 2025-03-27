// Performance measurement decorator
use std::time::Instant;
use tracing::info;
use std::panic::Location;

pub fn measure_time<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let location = Location::caller();
    let start = Instant::now();
    let result = f();
    info!(
        "⏱️  [{:>20}] Took {:>10?}",
        location
            .file()
            .split('\\')
            .last()
            .unwrap_or(location.file()),
        start.elapsed()
    );
    result
}
