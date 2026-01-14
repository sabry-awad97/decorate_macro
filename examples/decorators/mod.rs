//! Professional decorator implementations for the `decorate_macro` crate.
//!
//! This module provides production-ready decorators covering common cross-cutting concerns:
//!
//! - **Observability**: `measure_time`, `trace_calls`, `log_errors`
//! - **Resilience**: `with_retry`, `with_backoff`, `with_timeout`, `circuit_breaker`
//! - **Performance**: `with_cache`, `rate_limit`, `debounce`
//! - **Safety**: `safe_decorator`, `validate_input`
//!
//! # Example
//!
//! ```rust,ignore
//! use decorate_macro::decorate;
//! use decorators::{measure_time, with_retry, with_cache};
//!
//! #[decorate(measure_time, with_retry(3), with_cache("user", Duration::from_secs(60)))]
//! fn fetch_user(id: u64) -> Result<User, Error> {
//!     // ...
//! }
//! ```

mod circuit_breaker;
mod debounce;
mod log_errors;
mod measure_time;
mod rate_limit;
mod safe_decorator;
mod trace_calls;
mod validate;
mod with_backoff;
mod with_cache;
mod with_retry;
mod with_timeout;

pub use circuit_breaker::{CircuitState, circuit_breaker};
pub use debounce::debounce;
pub use log_errors::log_errors;
pub use measure_time::measure_time;
pub use rate_limit::rate_limit;
pub use safe_decorator::safe_decorator;
pub use trace_calls::trace_calls;
pub use validate::validate_input;
pub use with_backoff::with_backoff;
pub use with_cache::{CacheStats, get_cache_stats, with_cache};
pub use with_retry::with_retry;
pub use with_timeout::with_timeout;
