use std::any::Any;
use std::clone::Clone;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::info;

type CacheEntry = (Box<dyn Any + Send + Sync>, Instant);
type CacheMap = HashMap<String, CacheEntry>;

// Generic cache storage
static CACHE: LazyLock<Mutex<CacheMap>> = LazyLock::new(|| Mutex::new(HashMap::new()));

// Generic caching decorator with mutex poison recovery
pub fn with_cache<F, T, E>(cache_key: &str, ttl: Duration, f: F) -> Result<T, E>
where
    F: FnOnce() -> Result<T, E>,
    T: Clone + Send + Sync + 'static,
    E: std::fmt::Debug,
{
    let start = Instant::now();
    let cache = CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if let Some((cached_value, timestamp)) = cache.get(cache_key) {
        if timestamp.elapsed() < ttl
            && let Some(value) = cached_value.downcast_ref::<T>()
        {
            info!(
                "💾 Cache hit for key: [{}] ({:.2?})",
                cache_key,
                start.elapsed()
            );
            return Ok(value.clone());
        }
        info!("🔄 Cache expired for key: [{}]", cache_key);
    } else {
        info!("🔍 Cache miss [{}]", cache_key);
    }

    drop(cache);

    match f() {
        Ok(result) => {
            let mut cache = CACHE
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            info!(
                "📝 Cached new data [{}] ({:.2?})",
                cache_key,
                start.elapsed()
            );
            cache.insert(
                cache_key.to_string(),
                (Box::new(result.clone()), Instant::now()),
            );
            Ok(result)
        }
        Err(e) => Err(e),
    }
}
