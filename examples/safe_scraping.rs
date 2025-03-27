use decorate_macro::decorate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::{info, warn};

// Mock data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Product {
    id: String,
    name: String,
    price: f64,
    stock: i32,
}

// Global cache and rate limiter
static PRODUCT_CACHE: LazyLock<Mutex<HashMap<String, (Product, Instant)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static LAST_REQUEST: LazyLock<Mutex<Instant>> = LazyLock::new(|| Mutex::new(Instant::now()));

// Performance measurement decorator
fn measure_time<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let location = std::panic::Location::caller();
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

// Safe decorator with logging
fn safe_decorator<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let fn_name = std::any::type_name::<F>()
        .split("::")
        .last()
        .unwrap_or("unknown");
    info!("🚀 Starting: {}", fn_name);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
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

// Rate limiting decorator with mutex poison recovery
fn rate_limit<F, R>(delay_ms: u64, f: F) -> R
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
            std::thread::sleep(sleep_duration);
        } else {
            *last = Instant::now();
        }
    }
    f()
}

// Enhanced caching decorator with mutex poison recovery
fn with_cache<F>(cache_duration: Duration, id: &str, f: F) -> Result<Product, String>
where
    F: FnOnce() -> Result<Product, String>,
{
    let start = Instant::now();
    let cache = PRODUCT_CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if let Some((product, timestamp)) = cache.get(id) {
        if timestamp.elapsed() < cache_duration {
            info!("💾 Cache hit  [{}] ({:.2?})", id, start.elapsed());
            return Ok(product.clone());
        }
        info!("🔄 Cache expired [{}]", id);
    } else {
        info!("🔍 Cache miss [{}]", id);
    }

    drop(cache);

    match f() {
        Ok(result) => {
            let mut cache = PRODUCT_CACHE
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            info!("📝 Cached new data [{}] ({:.2?})", id, start.elapsed());
            cache.insert(id.to_string(), (result.clone(), Instant::now()));
            Ok(result)
        }
        Err(e) => Err(e),
    }
}

// Enhanced retry decorator with logging and timing
fn with_retry<F, R>(attempts: u32, f: F) -> R
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
                    std::thread::sleep(delay);
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

// Type alias for validation rule
type ValidationRule = (&'static dyn Fn(&str) -> bool, &'static str);

fn validate_product_id<F, R>(id: &str, f: F) -> Result<R, String>
where
    F: FnOnce() -> Result<R, String>,
{
    // Define validation rules with descriptive error messages
    let validation_rules: Vec<ValidationRule> = vec![
        (
            &|id: &str| !id.trim().is_empty(),
            "Product ID cannot be empty",
        ),
        (
            &|id: &str| id.len() <= 50,
            "Product ID too long (max 50 characters)",
        ),
        (
            &|id: &str| {
                id.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            },
            "Product ID contains invalid characters (only alphanumeric, '-' and '_' allowed)",
        ),
    ];

    // Apply all validation rules
    for (validator, error_msg) in validation_rules {
        if !validator(id) {
            return Err(error_msg.to_string());
        }
    }

    // If all validations pass, execute the wrapped function
    f()
}

// Mock database with more graceful error handling
#[decorate(measure_time)]
fn get_mock_product(id: &str) -> Option<Product> {
    if rand::random::<f64>() < 0.05 {
        warn!("🌐 Network error for product [{}]", id);
        return None;
    }

    let delay = rand::random::<u64>() % 50;
    std::thread::sleep(Duration::from_millis(delay));

    let mock_data = vec![
        Product {
            id: "123".to_string(),
            name: "Laptop".to_string(),
            price: 999.99,
            stock: 10,
        },
        Product {
            id: "456".to_string(),
            name: "Smartphone".to_string(),
            price: 599.99,
            stock: 15,
        },
    ];

    let product = mock_data.into_iter().find(|p| p.id == id);
    if product.is_none() {
        info!("❓ Product not found [{}]", id);
    }
    product
}

// Main scraping function with all decorators including validate_product_id
#[decorate(
    measure_time,
    safe_decorator,
    with_cache(Duration::from_secs(300), id),
    rate_limit(1000),
    validate_product_id(id)
)]
fn fetch_product(id: &str) -> Result<Product, String> {
    info!("Fetching product with ID: {}", id);
    get_mock_product(id).ok_or_else(|| format!("Product not found: {}", id))
}

// Batch fetching with retry, safety mechanisms, and timing
#[decorate(measure_time, safe_decorator)]
fn fetch_products(ids: &[&str]) -> Vec<Result<Product, String>> {
    info!("Batch fetching {} products", ids.len());
    ids.iter()
        .map(|&id| {
            // Modified to properly pass the id parameter
            with_retry(3, || fetch_product(id))
        })
        .collect()
}

fn main() {
    // Initialize logging with custom formatting
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false) // Hide target
        .with_thread_ids(false) // Hide thread IDs
        .with_thread_names(false) // Hide thread names
        .with_file(false) // Hide file names in the prefix
        .with_line_number(false) // Hide line numbers
        .with_level(true) // Show log levels
        .init();

    println!("\n📦 Starting product fetch operation\n");

    // Test with valid and invalid IDs
    let product_ids = vec![
        "123",
        "",
        "abc-123",
        "invalid@id",
        "very-very-very-long-product-id-that-exceeds-fifty-characters",
    ];
    let results = fetch_products(&product_ids);

    println!("\n=== Results ===");
    for (_id, result) in product_ids.iter().zip(results) {
        match result {
            Ok(product) => println!("✅ Found product: {:?}", product),
            Err(e) => println!("❌ Error: {}", e),
        }
    }
}
