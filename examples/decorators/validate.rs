//! Input validation decorator for defensive programming.

use tracing::{error, info};

/// Validation rule definition.
pub struct ValidationRule<T> {
    /// The validation predicate
    pub check: fn(&T) -> bool,
    /// Error message if validation fails
    pub message: &'static str,
}

impl<T> ValidationRule<T> {
    pub const fn new(check: fn(&T) -> bool, message: &'static str) -> Self {
        Self { check, message }
    }
}

/// Validates input against a set of rules before executing the function.
///
/// # Arguments
/// * `input` - The value to validate
/// * `rules` - Slice of validation rules to apply
/// * `f` - The function to execute if validation passes
///
/// # Returns
/// `Ok(R)` if validation passes and function succeeds, `Err` with validation message otherwise
///
/// # Example
///
/// ```rust,ignore
/// const ID_RULES: &[ValidationRule<u64>] = &[
///     ValidationRule::new(|id| *id > 0, "ID must be positive"),
///     ValidationRule::new(|id| *id < 1_000_000, "ID must be less than 1M"),
/// ];
///
/// #[decorate(validate_input(id, ID_RULES))]
/// fn get_user(id: u64) -> Result<User, String> {
///     // ...
/// }
/// ```
pub fn validate_input<T, F, R>(input: &T, rules: &[ValidationRule<T>], f: F) -> Result<R, String>
where
    F: FnOnce() -> Result<R, String>,
{
    info!("🔍 Validating input against {} rules", rules.len());

    for (i, rule) in rules.iter().enumerate() {
        if !(rule.check)(input) {
            error!(
                rule_index = %i,
                message = %rule.message,
                "❌ Validation failed"
            );
            return Err(rule.message.to_string());
        }
    }

    info!("✅ All validations passed");
    f()
}

/// Common validation rules for strings.
pub mod string_rules {
    use super::ValidationRule;

    pub const NOT_EMPTY: ValidationRule<String> =
        ValidationRule::new(|s| !s.trim().is_empty(), "String cannot be empty");

    pub const ALPHANUMERIC: ValidationRule<String> = ValidationRule::new(
        |s| s.chars().all(|c| c.is_alphanumeric()),
        "String must be alphanumeric",
    );

    pub const NO_WHITESPACE: ValidationRule<String> = ValidationRule::new(
        |s| !s.chars().any(|c| c.is_whitespace()),
        "String cannot contain whitespace",
    );
}

/// Common validation rules for numbers.
pub mod number_rules {
    use super::ValidationRule;

    pub const POSITIVE_I32: ValidationRule<i32> =
        ValidationRule::new(|n| *n > 0, "Number must be positive");

    pub const NON_NEGATIVE_I32: ValidationRule<i32> =
        ValidationRule::new(|n| *n >= 0, "Number cannot be negative");

    pub const POSITIVE_U64: ValidationRule<u64> =
        ValidationRule::new(|n| *n > 0, "Number must be positive");
}

/// Validates that a value is present (Some) before executing.
pub fn require_some<T, F, R>(value: &Option<T>, field_name: &str, f: F) -> Result<R, String>
where
    F: FnOnce() -> Result<R, String>,
{
    match value {
        Some(_) => {
            info!(field = %field_name, "✅ Required field present");
            f()
        }
        None => {
            error!(field = %field_name, "❌ Required field missing");
            Err(format!("Required field '{}' is missing", field_name))
        }
    }
}
