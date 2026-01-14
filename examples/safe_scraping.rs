//! Example demonstrating professional decorator usage for a web scraping scenario.
//!
//! This example shows how to combine multiple decorators for:
//! - Performance measurement
//! - Retry logic
//! - Caching
//! - Rate limiting
//! - Input validation

use decorate_macro::decorate;
use decorators::{measure_time, rate_limit, with_cache, with_retry};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, warn};

pub mod decorators;

/// Product data structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Product {
    id: String,
    name: String,
    price: f64,
    stock: i32,
}

/// Validates a product ID before fetching.
fn validate_product_id<F, R>(id: &str, f: F) -> Result<R, String>
where
    F: FnOnce() -> Result<R, String>,
{
    // Validation rules
    if id.trim().is_empty() {
        return Err("Product ID cannot be empty".to_string());
    }
    if id.len() > 50 {
        return Err("Product ID too long (max 50 characters)".to_string());
    }
    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err("Product ID contains invalid characters".to_string());
    }

    f()
}

/// Mock database lookup with simulated latency and failures.
#[decorate(measure_time)]
fn get_mock_product(id: &str) -> Option<Product> {
    // Simulate occasional network errors
    if rand::random::<f64>() < 0.05 {
        warn!("🌐 Network error for product [{}]", id);
        return None;
    }

    // Simulate variable latency
    let delay = rand::random::<u64>() % 50;
    std::thread::sleep(Duration::from_millis(delay));

    // Mock product database
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
        Product {
            id: "789".to_string(),
            name: "Headphones".to_string(),
            price: 149.99,
            stock: 50,
        },
    ];

    let product = mock_data.into_iter().find(|p| p.id == id);
    if product.is_none() {
        info!("❓ Product not found [{}]", id);
    }
    product
}

/// Fetches a product with full decorator stack:
/// - Performance measurement
/// - Retry on failure  
/// - Caching with TTL
/// - Rate limiting
/// - Input validation
#[decorate(
    measure_time,
    with_retry(3),
    with_cache(id, Duration::from_secs(300)),
    rate_limit(500),
    validate_product_id(id)
)]
fn fetch_product(id: &&str) -> Result<Product, String> {
    info!("📦 Fetching product: {}", id);
    get_mock_product(id).ok_or_else(|| format!("Product not found: {}", id))
}

/// Batch fetches multiple products.
#[decorate(measure_time)]
fn fetch_products(ids: &[&str]) -> Vec<Result<Product, String>> {
    info!("📦 Batch fetching {} products", ids.len());
    ids.iter().map(fetch_product).collect()
}

fn main() {
    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .with_level(true)
        .init();

    println!("\n📦 Product Fetch Demo\n");
    println!("{}", "=".repeat(50));

    // Test with various product IDs
    let product_ids = vec![
        "123",                                                                // Valid - exists
        "456",                                                                // Valid - exists
        "789",                                                                // Valid - exists
        "999",        // Valid - doesn't exist
        "",           // Invalid - empty
        "abc-123",    // Valid format
        "invalid@id", // Invalid - special chars
        "very-very-very-long-product-id-that-exceeds-fifty-characters-limit", // Invalid - too long
    ];

    let results = fetch_products(&product_ids);

    println!("\n=== Results ===\n");
    for (id, result) in product_ids.iter().zip(results) {
        match result {
            Ok(product) => {
                println!(
                    "✅ [{}] {} - ${:.2} ({} in stock)",
                    product.id, product.name, product.price, product.stock
                );
            }
            Err(e) => {
                println!("❌ [{}] Error: {}", id, e);
            }
        }
    }

    // Show cache statistics
    println!("\n=== Cache Stats ===");
    let stats = decorators::get_cache_stats();
    println!(
        "Hits: {}, Misses: {}, Hit Rate: {:.1}%",
        stats.hits,
        stats.misses,
        stats.hit_rate() * 100.0
    );
}
