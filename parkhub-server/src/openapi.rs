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

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use utoipa::OpenApi;

    use super::ApiDoc;

    fn snapshot_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("docs")
            .join("openapi.snapshot.json")
    }

    #[test]
    fn openapi_snapshot_is_in_sync() {
        let generated = ApiDoc::openapi()
            .to_pretty_json()
            .expect("openapi json serialization should succeed");
        let path = snapshot_path();

        if std::env::var("UPDATE_OPENAPI_SNAPSHOT").as_deref() == Ok("1") {
            fs::write(&path, format!("{}\n", generated)).expect("snapshot write should succeed");
            return;
        }

        let existing = fs::read_to_string(&path).expect(
            "OpenAPI snapshot missing. Generate it with: UPDATE_OPENAPI_SNAPSHOT=1 cargo test -p parkhub-server openapi_snapshot_is_in_sync",
        );

        assert_eq!(
            existing,
            format!("{}\n", generated),
            "OpenAPI snapshot changed. Re-generate with: UPDATE_OPENAPI_SNAPSHOT=1 cargo test -p parkhub-server openapi_snapshot_is_in_sync"
        );
    }
}
