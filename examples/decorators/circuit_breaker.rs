//! Circuit breaker pattern implementation for fault tolerance.
//!
//! Prevents cascading failures by temporarily blocking calls to a failing service.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation - requests pass through
    Closed,
    /// Failure threshold exceeded - requests are blocked
    Open,
    /// Testing if service recovered - limited requests allowed
    HalfOpen,
}

#[derive(Debug)]
struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
}

impl CircuitBreaker {
    fn new(failure_threshold: u32, success_threshold: u32, timeout: Duration) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            failure_threshold,
            success_threshold,
            timeout,
        }
    }

    fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.timeout {
                        info!("🔄 Circuit breaker transitioning to half-open");
                        self.state = CircuitState::HalfOpen;
                        self.success_count = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    true
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    fn record_success(&mut self) {
        match self.state {
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.success_threshold {
                    info!(
                        "✅ Circuit breaker closed after {} successes",
                        self.success_count
                    );
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                }
            }
            CircuitState::Closed => {
                self.failure_count = 0; // Reset on success
            }
            _ => {}
        }
    }

    fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());

        match self.state {
            CircuitState::Closed => {
                if self.failure_count >= self.failure_threshold {
                    error!(
                        "🔴 Circuit breaker opened after {} failures",
                        self.failure_count
                    );
                    self.state = CircuitState::Open;
                }
            }
            CircuitState::HalfOpen => {
                warn!("🔴 Circuit breaker re-opened after failure in half-open state");
                self.state = CircuitState::Open;
                self.success_count = 0;
            }
            _ => {}
        }
    }
}

type CircuitBreakerMap = HashMap<String, CircuitBreaker>;

static CIRCUIT_BREAKERS: LazyLock<Mutex<CircuitBreakerMap>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Circuit breaker decorator for fault tolerance.
///
/// # Arguments
/// * `name` - Unique identifier for this circuit breaker
/// * `failure_threshold` - Number of failures before opening the circuit
/// * `success_threshold` - Number of successes in half-open state before closing
/// * `timeout_secs` - Seconds to wait before transitioning from open to half-open
/// * `f` - The function to execute
///
/// # Returns
/// `Ok(R)` on success, `Err(E)` on failure or if circuit is open
///
/// # Example
///
/// ```rust,ignore
/// #[decorate(circuit_breaker("api", 5, 2, 30))]
/// fn call_external_api() -> Result<Response, Error> {
///     // ...
/// }
/// ```
pub fn circuit_breaker<F, R, E>(
    name: &str,
    failure_threshold: u32,
    success_threshold: u32,
    timeout_secs: u64,
    f: F,
) -> Result<R, E>
where
    F: FnOnce() -> Result<R, E>,
    E: std::fmt::Debug + From<String>,
{
    let mut breakers = CIRCUIT_BREAKERS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let breaker = breakers.entry(name.to_string()).or_insert_with(|| {
        CircuitBreaker::new(
            failure_threshold,
            success_threshold,
            Duration::from_secs(timeout_secs),
        )
    });

    if !breaker.can_execute() {
        warn!(
            circuit = %name,
            state = ?breaker.state,
            "🚫 Circuit breaker is open, rejecting request"
        );
        return Err(E::from(format!("Circuit breaker '{}' is open", name)));
    }

    let state_before = breaker.state;
    drop(breakers); // Release lock during execution

    let result = f();

    let mut breakers = CIRCUIT_BREAKERS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if let Some(breaker) = breakers.get_mut(name) {
        match &result {
            Ok(_) => {
                breaker.record_success();
                if state_before == CircuitState::HalfOpen {
                    info!(circuit = %name, "✅ Success in half-open state");
                }
            }
            Err(e) => {
                breaker.record_failure();
                warn!(
                    circuit = %name,
                    error = ?e,
                    failures = %breaker.failure_count,
                    "❌ Failure recorded"
                );
            }
        }
    }

    result
}

/// Gets the current state of a circuit breaker.
pub fn get_circuit_state(name: &str) -> Option<CircuitState> {
    CIRCUIT_BREAKERS
        .lock()
        .ok()
        .and_then(|breakers| breakers.get(name).map(|b| b.state))
}

/// Resets a circuit breaker to closed state.
pub fn reset_circuit(name: &str) {
    if let Ok(mut breakers) = CIRCUIT_BREAKERS.lock() {
        if let Some(breaker) = breakers.get_mut(name) {
            breaker.state = CircuitState::Closed;
            breaker.failure_count = 0;
            breaker.success_count = 0;
            info!(circuit = %name, "🔄 Circuit breaker reset");
        }
    }
}
