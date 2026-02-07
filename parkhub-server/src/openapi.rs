//! OpenAPI Documentation

use utoipa::OpenApi;

use crate::{
    error::{ApiError, FieldError},
    health::{ComponentHealth, HealthResponse, HealthStatus, ReadyResponse},
    jwt::TokenPair,
    requests::*,
};

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    info(
        title = "ParkHub API",
        version = "1.0.0",
        description = "Open source parking lot management system API",
        license(name = "MIT", url = "https://opensource.org/licenses/MIT"),
        contact(name = "ParkHub", url = "https://github.com/nash87/parkhub")
    ),
    servers((url = "/api/v1", description = "API v1")),
    tags(
        (name = "Authentication", description = "User authentication endpoints"),
        (name = "Users", description = "User management"),
        (name = "Bookings", description = "Parking bookings"),
        (name = "Lots", description = "Parking lots and slots"),
        (name = "Vehicles", description = "User vehicles"),
        (name = "Health", description = "Health check endpoints"),
        (name = "Monitoring", description = "Metrics and monitoring"),
        (name = "Admin", description = "Administrative endpoints")
    ),
    components(
        schemas(
            ApiError, FieldError,
            LoginRequest, RegisterRequest, ChangePasswordRequest, RefreshTokenRequest, TokenPair,
            CreateBookingRequest, ExtendBookingRequest, UpdateBookingRequest, BookingFiltersParams,
            VehicleRequest,
            UpdateProfileRequest, UpdatePreferencesRequest,
            CreateParkingLotRequest, UpdateParkingLotRequest,
            PaginationParams,
            HealthResponse, HealthStatus, ComponentHealth, ReadyResponse,
        )
    ),
    paths()
)]
pub struct ApiDoc;
