//! Shared range-validation helpers used by all domain validators.

/// Push an error if `value` is outside `[min, max]` (integer).
pub(crate) fn validate_range(errors: &mut Vec<String>, name: &str, value: u32, min: u32, max: u32) {
    if value < min || value > max {
        errors.push(format!("{name} = {value} is out of range [{min}, {max}]"));
    }
}

/// Push an error if `value` is outside `[min, max]` (float).
pub(crate) fn validate_range_f64(
    errors: &mut Vec<String>,
    name: &str,
    value: f64,
    min: f64,
    max: f64,
) {
    if value < min || value > max {
        errors.push(format!("{name} = {value} is out of range [{min}, {max}]"));
    }
}
