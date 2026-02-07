//! Data Models
//!
//! All shared data structures for the ParkHub system.
//! Simplified for company parking use case (no pricing/payment).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════════════════════
// USER & AUTHENTICATION MODELS
// ═══════════════════════════════════════════════════════════════════════════════

/// User account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub picture: Option<String>,
    pub phone: Option<String>,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub preferences: UserPreferences,
    pub is_active: bool,
}

/// User role for access control
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    #[default]
    User,
    Premium,
    Admin,
    SuperAdmin,
}

/// User preferences stored on server
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserPreferences {
    pub default_duration_minutes: Option<i32>,
    pub favorite_slots: Vec<String>,
    pub notifications_enabled: bool,
    pub email_reminders: bool,
    pub language: String,
    pub theme: String,
}

/// Authentication tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub token_type: String,
}

// ═══════════════════════════════════════════════════════════════════════════════
// PARKING LOT MODELS
// ═══════════════════════════════════════════════════════════════════════════════

/// Parking lot information (simplified for company parking)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParkingLot {
    pub id: Uuid,
    pub name: String,
    pub address: String,
    pub total_slots: i32,
    pub available_slots: i32,
    pub layout: Option<LotLayout>,
    pub status: LotStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Lot layout for the visual editor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotLayout {
    pub rows: Vec<LotRow>,
    pub road_label: Option<String>,
}

/// A row of parking slots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotRow {
    pub id: String,
    pub label: Option<String>,
    pub side: RowSide,
    pub slots: Vec<SlotConfig>,
}

/// Which side of the road a row is on
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RowSide {
    Top,
    Bottom,
}

/// Slot configuration within a layout row
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotConfig {
    pub id: String,
    pub number: String,
    pub status: SlotStatus,
    pub vehicle_plate: Option<String>,
    pub homeoffice_user: Option<String>,
}

/// Individual parking slot (for slot-level API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParkingSlot {
    pub id: Uuid,
    pub lot_id: Uuid,
    pub slot_number: String,
    pub status: SlotStatus,
    pub current_booking: Option<SlotBookingInfo>,
}

/// Slot availability status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SlotStatus {
    #[default]
    Available,
    Occupied,
    Reserved,
    Maintenance,
    Disabled,
    HomeOffice,
}

/// Brief booking info for slot display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotBookingInfo {
    pub booking_id: Uuid,
    pub user_id: Uuid,
    pub license_plate: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub is_own_booking: bool,
}

/// Lot operational status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LotStatus {
    #[default]
    Open,
    Closed,
    Full,
    Maintenance,
}

// ═══════════════════════════════════════════════════════════════════════════════
// BOOKING MODELS
// ═══════════════════════════════════════════════════════════════════════════════

/// Full booking information (simplified, no pricing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Booking {
    pub id: Uuid,
    pub user_id: Uuid,
    pub lot_id: Uuid,
    pub slot_id: Uuid,
    pub booking_type: BookingType,
    pub dauer_interval: Option<DauerInterval>,
    pub lot_name: Option<String>,
    pub slot_number: Option<String>,
    pub vehicle_plate: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub status: BookingStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub notes: Option<String>,
}

/// Booking type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum BookingType {
    #[default]
    Einmalig,
    Mehrtaegig,
    Dauer,
}

/// Interval for recurring (Dauer) bookings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DauerInterval {
    Weekly,
    Monthly,
}

/// Booking status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum BookingStatus {
    #[default]
    Pending,
    Confirmed,
    Active,
    Completed,
    Cancelled,
    Expired,
    NoShow,
}

/// Request to create a booking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBookingRequest {
    pub lot_id: Uuid,
    pub slot_id: Uuid,
    pub booking_type: Option<BookingType>,
    pub dauer_interval: Option<DauerInterval>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub vehicle_id: Option<Uuid>,
    pub license_plate: Option<String>,
    pub notes: Option<String>,
}

/// Booking history filters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BookingFilters {
    pub status: Option<BookingStatus>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub lot_id: Option<Uuid>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// VEHICLE MODELS
// ═══════════════════════════════════════════════════════════════════════════════

/// Vehicle information (simplified)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vehicle {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plate: String,
    pub make: Option<String>,
    pub model: Option<String>,
    pub color: Option<String>,
    pub is_default: bool,
    pub photo_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// HOMEOFFICE MODELS
// ═══════════════════════════════════════════════════════════════════════════════

/// Homeoffice settings for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeofficeSettings {
    pub user_id: Uuid,
    pub pattern: HomeofficePattern,
    pub single_days: Vec<HomeofficeDay>,
}

/// Recurring homeoffice pattern
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HomeofficePattern {
    /// Weekdays for homeoffice: 0=Mon, 1=Tue, ... 4=Fri
    pub weekdays: Vec<u8>,
}

/// A single homeoffice day entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeofficeDay {
    pub id: String,
    /// ISO date string (YYYY-MM-DD)
    pub date: String,
    pub reason: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// NOTIFICATION MODELS
// ═══════════════════════════════════════════════════════════════════════════════

/// User notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub read: bool,
    pub created_at: DateTime<Utc>,
}

/// Notification type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    BookingConfirmed,
    BookingReminder,
    BookingExpiring,
    BookingCancelled,
    SystemMessage,
}

// ═══════════════════════════════════════════════════════════════════════════════
// STATISTICS MODELS
// ═══════════════════════════════════════════════════════════════════════════════

/// Admin statistics overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminStats {
    pub total_users: i32,
    pub total_bookings: i32,
    pub total_lots: i32,
    pub active_bookings: i32,
    pub bookings_today: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_role_default() {
        let role = UserRole::default();
        assert_eq!(role, UserRole::User);
    }

    #[test]
    fn test_user_role_serialization() {
        assert_eq!(serde_json::to_string(&UserRole::User).unwrap(), "\"user\"");
        assert_eq!(serde_json::to_string(&UserRole::Admin).unwrap(), "\"admin\"");
    }

    #[test]
    fn test_slot_status_default() {
        let status = SlotStatus::default();
        assert_eq!(status, SlotStatus::Available);
    }

    #[test]
    fn test_slot_status_serialization() {
        assert_eq!(serde_json::to_string(&SlotStatus::Available).unwrap(), "\"available\"");
        assert_eq!(serde_json::to_string(&SlotStatus::HomeOffice).unwrap(), "\"home_office\"");
    }

    #[test]
    fn test_booking_status_default() {
        let status = BookingStatus::default();
        assert_eq!(status, BookingStatus::Pending);
    }

    #[test]
    fn test_booking_type_serialization() {
        assert_eq!(serde_json::to_string(&BookingType::Einmalig).unwrap(), "\"einmalig\"");
        assert_eq!(serde_json::to_string(&BookingType::Dauer).unwrap(), "\"dauer\"");
    }

    #[test]
    fn test_booking_filters_default() {
        let filters = BookingFilters::default();
        assert!(filters.status.is_none());
        assert!(filters.lot_id.is_none());
    }

    #[test]
    fn test_user_preferences_default() {
        let prefs = UserPreferences::default();
        assert!(!prefs.notifications_enabled);
        assert!(prefs.favorite_slots.is_empty());
    }
}
