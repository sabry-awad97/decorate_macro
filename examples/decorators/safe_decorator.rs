// Safe decorator with logging
use std::panic::catch_unwind;
use tracing::{info, warn};

pub fn safe_decorator<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let fn_name = std::any::type_name::<F>()
        .split("::")
        .last()
        .unwrap_or("unknown");
    info!("🚀 Starting: {}", fn_name);
    let result = catch_unwind(std::panic::AssertUnwindSafe(f));
    match result {
        Ok(value) => {
            info!("✅ Success: {}", fn_name);
            value
        }
        Err(e) => {
            warn!("❌ Failed: {} - {:?}", fn_name, e);
            panic!("Function execution failed");
        }
    }
}
