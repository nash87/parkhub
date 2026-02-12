//! Request DTOs with Validation
//!
//! Defines API request payloads with validation for OpenAPI documentation.
//! Some of these are used for schema generation only.

use chrono::{DateTime, Utc};
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::validation::{
    validate_booking_duration, validate_license_plate, validate_password_strength,
};

/// Login request (OpenAPI schema)
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(length(min = 3, max = 100))]
    pub username: String,
    #[validate(length(min = 1))]
    pub password: String,
}

/// Registration request (OpenAPI schema)
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 30))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(custom(function = "validate_password_strength"))]
    pub password: String,
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub phone: Option<String>,
}

/// Password change request (OpenAPI schema)
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    #[validate(custom(function = "validate_password_strength"))]
    pub new_password: String,
}

/// Token refresh request (OpenAPI schema)
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// Create booking request (OpenAPI schema)
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateBookingRequest {
    pub lot_id: Uuid,
    pub slot_id: Uuid,
    pub start_time: DateTime<Utc>,
    #[validate(custom(function = "validate_booking_duration"))]
    pub duration_minutes: i32,
    pub vehicle_id: Option<Uuid>,
    #[validate(custom(function = "validate_license_plate"))]
    pub license_plate: Option<String>,
    #[validate(length(max = 500))]
    pub notes: Option<String>,
}

/// Extend booking request (OpenAPI schema)
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ExtendBookingRequest {
    #[validate(range(min = 15, max = 480))]
    pub additional_minutes: i32,
}

/// Update booking request (OpenAPI schema)
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateBookingRequest {
    pub start_time: Option<DateTime<Utc>>,
    #[validate(custom(function = "validate_booking_duration"))]
    pub duration_minutes: Option<i32>,
    #[validate(length(max = 500))]
    pub notes: Option<String>,
}

/// Vehicle request (OpenAPI schema)
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct VehicleRequest {
    #[validate(custom(function = "validate_license_plate"))]
    pub license_plate: String,
    pub make: Option<String>,
    pub model: Option<String>,
    pub color: Option<String>,
}

/// Update user profile request (OpenAPI schema)
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateProfileRequest {
    pub name: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    pub phone: Option<String>,
    pub picture: Option<String>,
}

/// Update user preferences request (OpenAPI schema)
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdatePreferencesRequest {
    pub default_duration_minutes: Option<i32>,
    pub notifications_enabled: Option<bool>,
    pub email_reminders: Option<bool>,
    pub language: Option<String>,
    pub theme: Option<String>,
}

/// Create parking lot request (OpenAPI schema)
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateParkingLotRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1, max = 500))]
    pub address: String,
    pub latitude: f64,
    pub longitude: f64,
    #[validate(range(min = 1, max = 10000))]
    pub total_slots: i32,
}

/// Update parking lot request (OpenAPI schema)
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateParkingLotRequest {
    pub name: Option<String>,
    pub address: Option<String>,
    pub status: Option<String>,
}

/// Pagination parameters (OpenAPI schema)
#[derive(Debug, Deserialize, Validate, ToSchema, Default)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_per_page")]
    pub per_page: i32,
}

fn default_page() -> i32 {
    1
}
fn default_per_page() -> i32 {
    20
}

/// Booking list filters (OpenAPI schema)
#[derive(Debug, Deserialize, Validate, ToSchema, Default)]
pub struct BookingFiltersParams {
    pub status: Option<String>,
    pub lot_id: Option<Uuid>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    #[serde(flatten)]
    #[validate(nested)]
    pub pagination: PaginationParams,
}
