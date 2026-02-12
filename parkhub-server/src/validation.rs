//! Input Validation

use once_cell::sync::Lazy;
use regex::Regex;

/// Username regex (alphanumeric + underscore, 3-30 chars)
#[allow(dead_code)]
pub static USERNAME_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]{2,29}$").unwrap());

/// Custom validator for license plates
pub fn validate_license_plate(plate: &str) -> Result<(), validator::ValidationError> {
    let normalized = plate.to_uppercase().replace(['-', ' '], "");
    if normalized.len() < 2 || normalized.len() > 10 {
        return Err(validator::ValidationError::new("invalid_license_plate"));
    }
    // Allow letters (including German umlauts ÄÖÜ) and digits
    if !normalized.chars().all(|c| c.is_alphanumeric()) {
        return Err(validator::ValidationError::new("invalid_license_plate"));
    }
    Ok(())
}

/// Custom validator for booking duration
pub fn validate_booking_duration(minutes: i32) -> Result<(), validator::ValidationError> {
    if minutes < 15 {
        let mut err = validator::ValidationError::new("too_short");
        err.message = Some("Minimum booking duration is 15 minutes".into());
        return Err(err);
    }
    if minutes > 24 * 60 {
        let mut err = validator::ValidationError::new("too_long");
        err.message = Some("Maximum booking duration is 24 hours".into());
        return Err(err);
    }
    Ok(())
}

/// Custom validator for password strength
pub fn validate_password_strength(password: &str) -> Result<(), validator::ValidationError> {
    if password.len() < 8 {
        let mut err = validator::ValidationError::new("too_short");
        err.message = Some("Password must be at least 8 characters".into());
        return Err(err);
    }
    let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    if !has_lowercase || !has_uppercase || !has_digit {
        let mut err = validator::ValidationError::new("weak_password");
        err.message = Some("Password must contain lowercase, uppercase, and digit".into());
        return Err(err);
    }
    Ok(())
}
