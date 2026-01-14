//! Caching decorator with TTL and eviction support.

use std::any::Any;
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Cache entry with value and metadata.
struct CacheEntry {
    value: Box<dyn Any + Send + Sync>,
    created_at: Instant,
    last_accessed: Instant,
    access_count: u64,
}

/// Cache statistics.
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub size: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

type CacheMap = HashMap<String, CacheEntry>;

struct CacheState {
    entries: CacheMap,
    stats: CacheStats,
    max_size: usize,
}

static CACHE: LazyLock<RwLock<CacheState>> = LazyLock::new(|| {
    RwLock::new(CacheState {
        entries: HashMap::new(),
        stats: CacheStats::default(),
        max_size: 1000,
    })
});

/// Caches the result of a function with a TTL.
///
/// # Arguments
/// * `cache_key` - Unique key for this cached value
/// * `ttl` - Time-to-live for the cached value
/// * `f` - The function to execute on cache miss
///
/// # Example
///
/// ```rust,ignore
/// #[decorate(with_cache("user_123", Duration::from_secs(300)))]
/// fn fetch_user(id: u64) -> Result<User, Error> {
///     // Only called on cache miss
/// }
/// ```
pub fn with_cache<F, T, E>(cache_key: &str, ttl: Duration, f: F) -> Result<T, E>
where
    F: FnOnce() -> Result<T, E>,
    T: Clone + Send + Sync + 'static,
    E: std::fmt::Debug,
{
    let start = Instant::now();

    // Try to get from cache (read lock)
    {
        let cache = CACHE.read().unwrap_or_else(|p| p.into_inner());

        if let Some(entry) = cache.entries.get(cache_key) {
            if entry.created_at.elapsed() < ttl {
                if let Some(value) = entry.value.downcast_ref::<T>() {
                    info!(
                        key = %cache_key,
                        age_ms = %entry.created_at.elapsed().as_millis(),
                        access_count = %entry.access_count,
                        latency_us = %start.elapsed().as_micros(),
                        "💾 Cache hit"
                    );

                    // Update access stats (need write lock, but return value first)
                    let cloned = value.clone();
                    drop(cache);

                    // Update stats
                    if let Ok(mut cache) = CACHE.write() {
                        cache.stats.hits += 1;
                        if let Some(entry) = cache.entries.get_mut(cache_key) {
                            entry.last_accessed = Instant::now();
                            entry.access_count += 1;
                        }
                    }

                    return Ok(cloned);
                }
            } else {
                info!(
                    key = %cache_key,
                    age_ms = %entry.created_at.elapsed().as_millis(),
                    ttl_ms = %ttl.as_millis(),
                    "🔄 Cache expired"
                );
            }
        } else {
            info!(key = %cache_key, "🔍 Cache miss");
        }
    }

    // Cache miss - execute function
    let result = f();

    // Store in cache on success
    if let Ok(ref value) = result {
        let mut cache = CACHE.write().unwrap_or_else(|p| p.into_inner());
        cache.stats.misses += 1;

        // Evict if at capacity
        if cache.entries.len() >= cache.max_size {
            evict_lru(&mut cache);
        }

        let now = Instant::now();
        cache.entries.insert(
            cache_key.to_string(),
            CacheEntry {
                value: Box::new(value.clone()),
                created_at: now,
                last_accessed: now,
                access_count: 1,
            },
        );
        cache.stats.size = cache.entries.len();

        info!(
            key = %cache_key,
            ttl_ms = %ttl.as_millis(),
            cache_size = %cache.entries.len(),
            latency_ms = %start.elapsed().as_millis(),
            "📝 Cached result"
        );
    }

    result
}

/// Evicts the least recently used entry.
fn evict_lru(cache: &mut CacheState) {
    if let Some((key, _)) = cache
        .entries
        .iter()
        .min_by_key(|(_, entry)| entry.last_accessed)
        .map(|(k, v)| (k.clone(), v.last_accessed))
    {
        cache.entries.remove(&key);
        cache.stats.evictions += 1;
        warn!(key = %key, "🗑️ Evicted LRU entry");
    }
}

/// Invalidates a specific cache entry.
pub fn invalidate_cache(key: &str) {
    if let Ok(mut cache) = CACHE.write()
        && cache.entries.remove(key).is_some()
    {
        cache.stats.size = cache.entries.len();
        info!(key = %key, "🗑️ Cache entry invalidated");
    }
}

/// Invalidates all cache entries matching a prefix.
pub fn invalidate_cache_prefix(prefix: &str) {
    if let Ok(mut cache) = CACHE.write() {
        let keys_to_remove: Vec<_> = cache
            .entries
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();

        let count = keys_to_remove.len();
        for key in keys_to_remove {
            cache.entries.remove(&key);
        }
        cache.stats.size = cache.entries.len();

        info!(prefix = %prefix, count = %count, "🗑️ Cache entries invalidated by prefix");
    }
}

/// Clears the entire cache.
pub fn clear_cache() {
    if let Ok(mut cache) = CACHE.write() {
        let count = cache.entries.len();
        cache.entries.clear();
        cache.stats.size = 0;
        info!(count = %count, "🗑️ Cache cleared");
    }
}

/// Gets cache statistics.
pub fn get_cache_stats() -> CacheStats {
    CACHE
        .read()
        .map(|cache| cache.stats.clone())
        .unwrap_or_default()
}

/// Sets the maximum cache size.
pub fn set_cache_max_size(max_size: usize) {
    if let Ok(mut cache) = CACHE.write() {
        cache.max_size = max_size;

        // Evict if over new limit
        while cache.entries.len() > max_size {
            evict_lru(&mut cache);
        }

        info!(max_size = %max_size, "📊 Cache max size updated");
    }
}
