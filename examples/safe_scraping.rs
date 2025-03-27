use decorate_macro::decorate;
use decorators::{measure_time, rate_limit, safe_decorator, with_cache, with_retry};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, warn};

pub mod decorators;

// Mock data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Product {
    id: String,
    name: String,
    price: f64,
    stock: i32,
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
    with_retry(3),
    with_cache(id, Duration::from_secs(300)),
    rate_limit(1000),
    validate_product_id(id)
)]
fn fetch_product(id: &&str) -> Result<Product, String> {
    info!("Fetching product with ID: {}", id);
    get_mock_product(id).ok_or_else(|| format!("Product not found: {}", id))
}

// Batch fetching with retry, safety mechanisms, and timing
#[decorate(measure_time, safe_decorator)]
fn fetch_products(ids: &[&str]) -> Vec<Result<Product, String>> {
    info!("Batch fetching {} products", ids.len());
    ids.iter().map(fetch_product).collect()
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
