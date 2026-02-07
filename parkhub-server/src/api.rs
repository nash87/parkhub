//! HTTP API Routes
//!
//! RESTful API for the parking system.

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use chrono::{Duration, Utc};
use metrics_exporter_prometheus::PrometheusHandle;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

use crate::metrics;
use crate::rate_limit::{EndpointRateLimiters, per_ip};
use std::net::IpAddr;
use crate::openapi::ApiDoc;
use crate::static_files;

use parkhub_common::{
    ApiResponse, AuthTokens, Booking, BookingStatus, BookingType,
    CreateBookingRequest, HandshakeRequest, HandshakeResponse, HomeofficeDay,
    HomeofficePattern, HomeofficeSettings, LoginRequest, LoginResponse, LotLayout,
    ParkingLot, ParkingSlot, RefreshTokenRequest, RegisterRequest, ServerStatus,
    SlotStatus, User, UserPreferences, UserRole, Vehicle, AdminStats,
    PROTOCOL_VERSION,
};

use crate::db::Session;
use crate::AppState;

type SharedState = Arc<RwLock<AppState>>;

/// User ID extracted from auth token
#[derive(Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
}

/// Create the API router with OpenAPI docs and metrics
pub fn create_router(state: SharedState) -> Router {
    let rate_limiters = Arc::new(EndpointRateLimiters::new());
    let metrics_handle = metrics::init_metrics();

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/health/live", get(liveness_check))
        .route("/health/ready", get(readiness_check))
        .route("/handshake", post(handshake))
        .route("/status", get(server_status))
        .route("/api/v1/auth/login", post(login))
        // Rate limiters are applied inside login/register handlers
        .route("/api/v1/auth/register", post(register))
        .route("/api/v1/auth/refresh", post(refresh_token));

    // Protected routes (auth required)
    let protected_routes = Router::new()
        .route("/api/v1/users/me", get(get_current_user))
        .route("/api/v1/users/:id", get(get_user))
        .route("/api/v1/lots", get(list_lots).post(create_lot))
        .route("/api/v1/lots/:id", get(get_lot))
        .route("/api/v1/lots/:id/slots", get(get_lot_slots))
        .route("/api/v1/lots/:id/layout", get(get_lot_layout).put(update_lot_layout))
        .route("/api/v1/bookings", get(list_bookings).post(create_booking))
        .route(
            "/api/v1/bookings/:id",
            get(get_booking).delete(cancel_booking),
        )
        .route("/api/v1/bookings/ical", get(export_ical))
        .route("/api/v1/bookings/:id/checkin", post(checkin_booking))
        .route("/api/v1/vehicles", get(list_vehicles).post(create_vehicle))
        .route("/api/v1/vehicles/:id", delete(delete_vehicle))
        .route("/api/v1/vehicles/:id/photo", post(upload_vehicle_photo))
        // Homeoffice routes
        .route("/api/v1/homeoffice", get(get_homeoffice_settings))
        .route("/api/v1/homeoffice/pattern", put(update_homeoffice_pattern))
        .route("/api/v1/homeoffice/days", post(add_homeoffice_day))
        .route("/api/v1/homeoffice/days/:id", delete(remove_homeoffice_day))
        // Admin routes
        .route("/api/v1/admin/users", get(admin_list_users))
        .route("/api/v1/admin/bookings", get(admin_list_bookings))
        .route("/api/v1/admin/stats", get(admin_stats))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let metrics_handle_clone = metrics_handle.clone();

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .route("/metrics", get(move || async move {
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
                metrics_handle_clone.render(),
            )
        }))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .fallback(static_files::static_handler)
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
}

// ═══════════════════════════════════════════════════════════════════════════════
// AUTH MIDDLEWARE
// ═══════════════════════════════════════════════════════════════════════════════

async fn auth_middleware(
    State(state): State<SharedState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<()>>)> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(h) if h.starts_with("Bearer ") => &h[7..],
        _ => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::error("UNAUTHORIZED", "Missing or invalid authorization header")),
            ));
        }
    };

    let state_guard = state.read().await;
    let session = match state_guard.db.get_session(token).await {
        Ok(Some(s)) if !s.is_expired() => s,
        _ => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::error("UNAUTHORIZED", "Invalid or expired token")),
            ));
        }
    };
    drop(state_guard);

    request.extensions_mut().insert(AuthUser {
        user_id: session.user_id,
    });

    Ok(next.run(request).await)
}

// ═══════════════════════════════════════════════════════════════════════════════
// HEALTH & DISCOVERY
// ═══════════════════════════════════════════════════════════════════════════════

async fn health_check() -> &'static str {
    "OK"
}

async fn liveness_check() -> StatusCode {
    StatusCode::OK
}

async fn readiness_check(State(state): State<SharedState>) -> impl IntoResponse {
    let state = state.read().await;
    match state.db.stats().await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"ready": true}))),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"ready": false, "reason": format!("Database unavailable: {}", e)})),
        ),
    }
}

async fn handshake(
    State(state): State<SharedState>,
    Json(request): Json<HandshakeRequest>,
) -> Json<ApiResponse<HandshakeResponse>> {
    let state = state.read().await;
    if request.protocol_version != PROTOCOL_VERSION {
        return Json(ApiResponse::error(
            "PROTOCOL_MISMATCH",
            format!("Protocol version mismatch: server={}, client={}", PROTOCOL_VERSION, request.protocol_version),
        ));
    }
    Json(ApiResponse::success(HandshakeResponse {
        server_name: state.config.server_name.clone(),
        server_version: env!("CARGO_PKG_VERSION").to_string(),
        protocol_version: PROTOCOL_VERSION.to_string(),
        requires_auth: true,
        certificate_fingerprint: String::new(),
    }))
}

async fn server_status(State(state): State<SharedState>) -> Json<ApiResponse<ServerStatus>> {
    let state = state.read().await;
    let db_stats = state.db.stats().await.unwrap_or_else(|_| crate::db::DatabaseStats {
        users: 0, bookings: 0, parking_lots: 0, slots: 0, sessions: 0, vehicles: 0,
    });
    Json(ApiResponse::success(ServerStatus {
        uptime_seconds: 0,
        connected_clients: 0,
        total_users: db_stats.users as u32,
        total_bookings: db_stats.bookings as u32,
        database_size_bytes: 0,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════════
// AUTHENTICATION
// ═══════════════════════════════════════════════════════════════════════════════

async fn login(
    State(state): State<SharedState>,
    Json(request): Json<LoginRequest>,
) -> (StatusCode, Json<ApiResponse<LoginResponse>>) {
    let state_guard = state.read().await;

    let user = match state_guard.db.get_user_by_username(&request.username).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            match state_guard.db.get_user_by_email(&request.username).await {
                Ok(Some(u)) => u,
                _ => return (StatusCode::UNAUTHORIZED, Json(ApiResponse::error("INVALID_CREDENTIALS", "Invalid username or password"))),
            }
        }
        Err(e) => {
            tracing::error!("Database error during login: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Internal server error")));
        }
    };

    if !verify_password(&request.password, &user.password_hash) {
        return (StatusCode::UNAUTHORIZED, Json(ApiResponse::error("INVALID_CREDENTIALS", "Invalid username or password")));
    }

    if !user.is_active {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::error("ACCOUNT_DISABLED", "This account has been disabled")));
    }

    let session = Session::new(user.id, 24);
    let access_token = Uuid::new_v4().to_string();

    if let Err(e) = state_guard.db.save_session(&access_token, &session).await {
        tracing::error!("Failed to save session: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to create session")));
    }

    let mut response_user = user.clone();
    response_user.password_hash = String::new();

    (
        StatusCode::OK,
        Json(ApiResponse::success(LoginResponse {
            user: response_user,
            tokens: AuthTokens {
                access_token,
                refresh_token: session.refresh_token,
                expires_at: session.expires_at,
                token_type: "Bearer".to_string(),
            },
        })),
    )
}

async fn register(
    State(state): State<SharedState>,
    Json(request): Json<RegisterRequest>,
) -> (StatusCode, Json<ApiResponse<LoginResponse>>) {
    let state_guard = state.read().await;

    if let Ok(Some(_)) = state_guard.db.get_user_by_email(&request.email).await {
        return (StatusCode::CONFLICT, Json(ApiResponse::error("EMAIL_EXISTS", "An account with this email already exists")));
    }

    let username = request.email.split('@').next().unwrap_or("user").to_string();
    let mut final_username = username.clone();
    let mut counter = 1;
    while let Ok(Some(_)) = state_guard.db.get_user_by_username(&final_username).await {
        final_username = format!("{}{}", username, counter);
        counter += 1;
    }

    let password_hash = hash_password(&request.password);
    let now = Utc::now();
    let user = User {
        id: Uuid::new_v4(),
        username: final_username,
        email: request.email,
        password_hash,
        name: request.name,
        picture: None,
        phone: None,
        role: UserRole::User,
        created_at: now,
        updated_at: now,
        last_login: Some(now),
        preferences: UserPreferences::default(),
        is_active: true,
    };

    if let Err(e) = state_guard.db.save_user(&user).await {
        tracing::error!("Failed to save user: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to create account")));
    }

    let session = Session::new(user.id, 24);
    let access_token = Uuid::new_v4().to_string();

    if let Err(e) = state_guard.db.save_session(&access_token, &session).await {
        tracing::error!("Failed to save session: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to create session")));
    }

    let mut response_user = user.clone();
    response_user.password_hash = String::new();

    (
        StatusCode::CREATED,
        Json(ApiResponse::success(LoginResponse {
            user: response_user,
            tokens: AuthTokens {
                access_token,
                refresh_token: session.refresh_token,
                expires_at: session.expires_at,
                token_type: "Bearer".to_string(),
            },
        })),
    )
}

async fn refresh_token(
    State(_state): State<SharedState>,
    Json(_request): Json<RefreshTokenRequest>,
) -> (StatusCode, Json<ApiResponse<AuthTokens>>) {
    (StatusCode::NOT_IMPLEMENTED, Json(ApiResponse::error("NOT_IMPLEMENTED", "Token refresh not yet fully implemented")))
}

// ═══════════════════════════════════════════════════════════════════════════════
// USERS
// ═══════════════════════════════════════════════════════════════════════════════

async fn get_current_user(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<ApiResponse<User>>) {
    let state = state.read().await;
    match state.db.get_user(&auth_user.user_id.to_string()).await {
        Ok(Some(mut user)) => {
            user.password_hash = String::new();
            (StatusCode::OK, Json(ApiResponse::success(user)))
        }
        Ok(None) => (StatusCode::NOT_FOUND, Json(ApiResponse::error("NOT_FOUND", "User not found"))),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Internal server error")))
        }
    }
}

async fn get_user(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<ApiResponse<User>>) {
    let state = state.read().await;
    match state.db.get_user(&id).await {
        Ok(Some(mut user)) => {
            user.password_hash = String::new();
            (StatusCode::OK, Json(ApiResponse::success(user)))
        }
        Ok(None) => (StatusCode::NOT_FOUND, Json(ApiResponse::error("NOT_FOUND", "User not found"))),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Internal server error")))
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PARKING LOTS
// ═══════════════════════════════════════════════════════════════════════════════

async fn list_lots(State(state): State<SharedState>) -> Json<ApiResponse<Vec<ParkingLot>>> {
    let state = state.read().await;
    match state.db.list_parking_lots().await {
        Ok(lots) => Json(ApiResponse::success(lots)),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            Json(ApiResponse::error("SERVER_ERROR", "Failed to list parking lots"))
        }
    }
}

async fn create_lot(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(lot): Json<ParkingLot>,
) -> (StatusCode, Json<ApiResponse<ParkingLot>>) {
    let state_guard = state.read().await;
    let user = match state_guard.db.get_user(&auth_user.user_id.to_string()).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Access denied"))),
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Admin access required")));
    }
    if let Err(e) = state_guard.db.save_parking_lot(&lot).await {
        tracing::error!("Failed to save parking lot: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to create parking lot")));
    }
    (StatusCode::CREATED, Json(ApiResponse::success(lot)))
}

async fn get_lot(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<ApiResponse<ParkingLot>>) {
    let state = state.read().await;
    match state.db.get_parking_lot(&id).await {
        Ok(Some(lot)) => (StatusCode::OK, Json(ApiResponse::success(lot))),
        Ok(None) => (StatusCode::NOT_FOUND, Json(ApiResponse::error("NOT_FOUND", "Parking lot not found"))),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Internal server error")))
        }
    }
}

async fn get_lot_slots(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<Vec<ParkingSlot>>> {
    let state = state.read().await;
    match state.db.list_slots_by_lot(&id).await {
        Ok(slots) => Json(ApiResponse::success(slots)),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            Json(ApiResponse::error("SERVER_ERROR", "Failed to list slots"))
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// LOT LAYOUT
// ═══════════════════════════════════════════════════════════════════════════════

async fn get_lot_layout(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<ApiResponse<Option<LotLayout>>>) {
    let state = state.read().await;
    // First check the lot's own layout field
    match state.db.get_parking_lot(&id).await {
        Ok(Some(lot)) => {
            if lot.layout.is_some() {
                return (StatusCode::OK, Json(ApiResponse::success(lot.layout)));
            }
        }
        Ok(None) => return (StatusCode::NOT_FOUND, Json(ApiResponse::error("NOT_FOUND", "Parking lot not found"))),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Internal server error")));
        }
    }
    // Fall back to separate layout table
    match state.db.get_lot_layout(&id).await {
        Ok(layout) => (StatusCode::OK, Json(ApiResponse::success(layout))),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Internal server error")))
        }
    }
}

async fn update_lot_layout(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(layout): Json<LotLayout>,
) -> (StatusCode, Json<ApiResponse<LotLayout>>) {
    let state_guard = state.read().await;
    // Check admin
    let user = match state_guard.db.get_user(&auth_user.user_id.to_string()).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Access denied"))),
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Admin access required")));
    }
    // Update lot layout field
    match state_guard.db.get_parking_lot(&id).await {
        Ok(Some(mut lot)) => {
            lot.layout = Some(layout.clone());
            lot.updated_at = Utc::now();
            if let Err(e) = state_guard.db.save_parking_lot(&lot).await {
                tracing::error!("Failed to save parking lot: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to update layout")));
            }
        }
        Ok(None) => return (StatusCode::NOT_FOUND, Json(ApiResponse::error("NOT_FOUND", "Parking lot not found"))),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Internal server error")));
        }
    }
    // Also save to separate layout table
    let _ = state_guard.db.save_lot_layout(&id, &layout).await;
    (StatusCode::OK, Json(ApiResponse::success(layout)))
}

// ═══════════════════════════════════════════════════════════════════════════════
// BOOKINGS
// ═══════════════════════════════════════════════════════════════════════════════

async fn list_bookings(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Json<ApiResponse<Vec<Booking>>> {
    let state = state.read().await;
    match state.db.list_bookings_by_user(&auth_user.user_id.to_string()).await {
        Ok(bookings) => Json(ApiResponse::success(bookings)),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            Json(ApiResponse::error("SERVER_ERROR", "Failed to list bookings"))
        }
    }
}

async fn create_booking(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateBookingRequest>,
) -> (StatusCode, Json<ApiResponse<Booking>>) {
    let state_guard = state.read().await;

    // Check if slot exists and is available
    let slot = match state_guard.db.get_parking_slot(&req.slot_id.to_string()).await {
        Ok(Some(s)) => s,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(ApiResponse::error("NOT_FOUND", "Slot not found"))),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Internal server error")));
        }
    };

    if slot.status != SlotStatus::Available {
        return (StatusCode::CONFLICT, Json(ApiResponse::error("SLOT_UNAVAILABLE", "This slot is not available")));
    }

    // Get lot name
    let lot_name = match state_guard.db.get_parking_lot(&req.lot_id.to_string()).await {
        Ok(Some(lot)) => Some(lot.name),
        _ => None,
    };

    // Get vehicle plate
    let vehicle_plate = if let Some(plate) = &req.license_plate {
        Some(plate.clone())
    } else if let Some(vid) = &req.vehicle_id {
        match state_guard.db.get_vehicle(&vid.to_string()).await {
            Ok(Some(v)) => Some(v.plate),
            _ => None,
        }
    } else {
        None
    };

    // Calculate end time
    let end_time = if let Some(end) = req.end_time {
        end
    } else if let Some(mins) = req.duration_minutes {
        req.start_time + Duration::minutes(mins as i64)
    } else {
        req.start_time + Duration::hours(1) // Default 1 hour
    };

    let now = Utc::now();
    let booking = Booking {
        id: Uuid::new_v4(),
        user_id: auth_user.user_id,
        lot_id: req.lot_id,
        slot_id: req.slot_id,
        booking_type: req.booking_type.unwrap_or_default(),
        dauer_interval: req.dauer_interval,
        lot_name,
        slot_number: Some(slot.slot_number.clone()),
        vehicle_plate,
        start_time: req.start_time,
        end_time,
        status: BookingStatus::Confirmed,
        created_at: now,
        updated_at: now,
        notes: req.notes,
    };

    if let Err(e) = state_guard.db.save_booking(&booking).await {
        tracing::error!("Failed to save booking: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to create booking")));
    }

    // Update slot status
    let mut updated_slot = slot;
    updated_slot.status = SlotStatus::Reserved;
    let _ = state_guard.db.save_parking_slot(&updated_slot).await;

    (StatusCode::CREATED, Json(ApiResponse::success(booking)))
}

async fn get_booking(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> (StatusCode, Json<ApiResponse<Booking>>) {
    let state = state.read().await;
    match state.db.get_booking(&id).await {
        Ok(Some(booking)) => {
            if booking.user_id != auth_user.user_id {
                return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Access denied")));
            }
            (StatusCode::OK, Json(ApiResponse::success(booking)))
        }
        Ok(None) => (StatusCode::NOT_FOUND, Json(ApiResponse::error("NOT_FOUND", "Booking not found"))),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Internal server error")))
        }
    }
}

async fn cancel_booking(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let state_guard = state.read().await;
    let booking = match state_guard.db.get_booking(&id).await {
        Ok(Some(b)) => b,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(ApiResponse::error("NOT_FOUND", "Booking not found"))),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Internal server error")));
        }
    };

    if booking.user_id != auth_user.user_id {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Access denied")));
    }

    let mut updated_booking = booking.clone();
    updated_booking.status = BookingStatus::Cancelled;
    updated_booking.updated_at = Utc::now();

    if let Err(e) = state_guard.db.save_booking(&updated_booking).await {
        tracing::error!("Failed to update booking: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to cancel booking")));
    }

    // Free up the slot
    if let Ok(Some(mut slot)) = state_guard.db.get_parking_slot(&booking.slot_id.to_string()).await {
        slot.status = SlotStatus::Available;
        let _ = state_guard.db.save_parking_slot(&slot).await;
    }

    (StatusCode::OK, Json(ApiResponse::success(())))
}

// ═══════════════════════════════════════════════════════════════════════════════
// VEHICLES
// ═══════════════════════════════════════════════════════════════════════════════

async fn list_vehicles(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Json<ApiResponse<Vec<Vehicle>>> {
    let state = state.read().await;
    match state.db.list_vehicles_by_user(&auth_user.user_id.to_string()).await {
        Ok(vehicles) => Json(ApiResponse::success(vehicles)),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            Json(ApiResponse::error("SERVER_ERROR", "Failed to list vehicles"))
        }
    }
}

async fn create_vehicle(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(mut vehicle): Json<Vehicle>,
) -> (StatusCode, Json<ApiResponse<Vehicle>>) {
    vehicle.user_id = auth_user.user_id;
    vehicle.id = Uuid::new_v4();
    vehicle.created_at = Utc::now();

    let state_guard = state.read().await;
    if let Err(e) = state_guard.db.save_vehicle(&vehicle).await {
        tracing::error!("Failed to save vehicle: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to create vehicle")));
    }
    (StatusCode::CREATED, Json(ApiResponse::success(vehicle)))
}

async fn delete_vehicle(
    State(_state): State<SharedState>,
    Extension(_auth_user): Extension<AuthUser>,
    Path(_id): Path<String>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    (StatusCode::OK, Json(ApiResponse::success(())))
}

async fn upload_vehicle_photo(
    State(_state): State<SharedState>,
    Extension(_auth_user): Extension<AuthUser>,
    Path(_id): Path<String>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    // TODO: Implement file upload
    (StatusCode::OK, Json(ApiResponse::success(serde_json::json!({"photo_url": null}))))
}

// ═══════════════════════════════════════════════════════════════════════════════
// HOMEOFFICE
// ═══════════════════════════════════════════════════════════════════════════════

async fn get_homeoffice_settings(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<ApiResponse<HomeofficeSettings>>) {
    let state = state.read().await;
    match state.db.get_homeoffice_settings(&auth_user.user_id.to_string()).await {
        Ok(Some(settings)) => (StatusCode::OK, Json(ApiResponse::success(settings))),
        Ok(None) => {
            // Return default empty settings
            let settings = HomeofficeSettings {
                user_id: auth_user.user_id,
                pattern: HomeofficePattern { weekdays: vec![] },
                single_days: vec![],
            };
            (StatusCode::OK, Json(ApiResponse::success(settings)))
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Internal server error")))
        }
    }
}

async fn update_homeoffice_pattern(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(pattern): Json<HomeofficePattern>,
) -> (StatusCode, Json<ApiResponse<HomeofficeSettings>>) {
    let state_guard = state.read().await;
    let mut settings = match state_guard.db.get_homeoffice_settings(&auth_user.user_id.to_string()).await {
        Ok(Some(s)) => s,
        _ => HomeofficeSettings {
            user_id: auth_user.user_id,
            pattern: HomeofficePattern { weekdays: vec![] },
            single_days: vec![],
        },
    };
    settings.pattern = pattern;
    if let Err(e) = state_guard.db.save_homeoffice_settings(&settings).await {
        tracing::error!("Failed to save homeoffice settings: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to save settings")));
    }
    (StatusCode::OK, Json(ApiResponse::success(settings)))
}

async fn add_homeoffice_day(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(day): Json<HomeofficeDay>,
) -> (StatusCode, Json<ApiResponse<HomeofficeSettings>>) {
    let state_guard = state.read().await;
    let mut settings = match state_guard.db.get_homeoffice_settings(&auth_user.user_id.to_string()).await {
        Ok(Some(s)) => s,
        _ => HomeofficeSettings {
            user_id: auth_user.user_id,
            pattern: HomeofficePattern { weekdays: vec![] },
            single_days: vec![],
        },
    };
    settings.single_days.push(day);
    if let Err(e) = state_guard.db.save_homeoffice_settings(&settings).await {
        tracing::error!("Failed to save homeoffice settings: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to save settings")));
    }
    (StatusCode::CREATED, Json(ApiResponse::success(settings)))
}

async fn remove_homeoffice_day(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(day_id): Path<String>,
) -> (StatusCode, Json<ApiResponse<HomeofficeSettings>>) {
    let state_guard = state.read().await;
    let mut settings = match state_guard.db.get_homeoffice_settings(&auth_user.user_id.to_string()).await {
        Ok(Some(s)) => s,
        _ => return (StatusCode::NOT_FOUND, Json(ApiResponse::error("NOT_FOUND", "No homeoffice settings found"))),
    };
    settings.single_days.retain(|d| d.id != day_id);
    if let Err(e) = state_guard.db.save_homeoffice_settings(&settings).await {
        tracing::error!("Failed to save homeoffice settings: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to save settings")));
    }
    (StatusCode::OK, Json(ApiResponse::success(settings)))
}

// ═══════════════════════════════════════════════════════════════════════════════
// ADMIN
// ═══════════════════════════════════════════════════════════════════════════════

async fn admin_list_users(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<ApiResponse<Vec<User>>>) {
    let state_guard = state.read().await;
    let user = match state_guard.db.get_user(&auth_user.user_id.to_string()).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Access denied"))),
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Admin access required")));
    }
    match state_guard.db.list_users().await {
        Ok(mut users) => {
            for u in &mut users {
                u.password_hash = String::new();
            }
            (StatusCode::OK, Json(ApiResponse::success(users)))
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to list users")))
        }
    }
}

async fn admin_list_bookings(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<ApiResponse<Vec<Booking>>>) {
    let state_guard = state.read().await;
    let user = match state_guard.db.get_user(&auth_user.user_id.to_string()).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Access denied"))),
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Admin access required")));
    }
    match state_guard.db.list_bookings().await {
        Ok(bookings) => (StatusCode::OK, Json(ApiResponse::success(bookings))),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to list bookings")))
        }
    }
}

async fn admin_stats(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<ApiResponse<AdminStats>>) {
    let state_guard = state.read().await;
    let user = match state_guard.db.get_user(&auth_user.user_id.to_string()).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Access denied"))),
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Admin access required")));
    }
    let db_stats = state_guard.db.stats().await.unwrap_or_default();
    let bookings = state_guard.db.list_bookings().await.unwrap_or_default();
    let now = Utc::now();
    let active_bookings = bookings.iter().filter(|b| b.status == BookingStatus::Active || b.status == BookingStatus::Confirmed).count();
    let today = now.date_naive();
    let bookings_today = bookings.iter().filter(|b| b.created_at.date_naive() == today).count();

    (StatusCode::OK, Json(ApiResponse::success(AdminStats {
        total_users: db_stats.users as i32,
        total_bookings: db_stats.bookings as i32,
        total_lots: db_stats.parking_lots as i32,
        active_bookings: active_bookings as i32,
        bookings_today: bookings_today as i32,
    })))
}


// ═══════════════════════════════════════════════════════════════════════════════
// ICAL EXPORT
// ═══════════════════════════════════════════════════════════════════════════════

async fn export_ical(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
) -> impl IntoResponse {
    let state = state.read().await;
    let bookings = match state.db.list_bookings_by_user(&auth_user.user_id.to_string()).await {
        Ok(b) => b,
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CONTENT_TYPE, "text/plain")],
                "Internal server error".to_string(),
            );
        }
    };

    let mut ical = String::from("BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//ParkHub//Bookings//EN\r\nCALSCALE:GREGORIAN\r\n");

    for booking in &bookings {
        if booking.status == BookingStatus::Cancelled {
            continue;
        }
        let dtstart = booking.start_time.format("%Y%m%dT%H%M%SZ");
        let dtend = booking.end_time.format("%Y%m%dT%H%M%SZ");
        let created = booking.created_at.format("%Y%m%dT%H%M%SZ");
        let summary = format!("Parking: {}", booking.slot_number.as_deref().unwrap_or("Unknown"));
        let location = booking.lot_name.as_deref().unwrap_or("Parking Lot");
        let description = format!("Booking ID: {}\\nStatus: {:?}", booking.id, booking.status);

        ical.push_str(&format!(
            "BEGIN:VEVENT\r\nUID:{}@parkhub\r\nDTSTAMP:{}\r\nDTSTART:{}\r\nDTEND:{}\r\nSUMMARY:{}\r\nLOCATION:{}\r\nDESCRIPTION:{}\r\nEND:VEVENT\r\n",
            booking.id, created, dtstart, dtend, summary, location, description
        ));
    }

    ical.push_str("END:VCALENDAR\r\n");

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/calendar; charset=utf-8"),
        ],
        ical,
    )
}

// ═══════════════════════════════════════════════════════════════════════════════
// CHECK-IN
// ═══════════════════════════════════════════════════════════════════════════════

async fn checkin_booking(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> (StatusCode, Json<ApiResponse<Booking>>) {
    let state_guard = state.read().await;
    let booking = match state_guard.db.get_booking(&id).await {
        Ok(Some(b)) => b,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(ApiResponse::error("NOT_FOUND", "Booking not found"))),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Internal server error")));
        }
    };

    if booking.user_id != auth_user.user_id {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Access denied")));
    }

    if booking.status != BookingStatus::Confirmed {
        return (StatusCode::CONFLICT, Json(ApiResponse::error("INVALID_STATUS", "Booking is not in confirmed status")));
    }

    let mut updated_booking = booking;
    updated_booking.status = BookingStatus::Active;
    updated_booking.updated_at = Utc::now();

    if let Err(e) = state_guard.db.save_booking(&updated_booking).await {
        tracing::error!("Failed to update booking: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to check in")));
    }

    // Update slot to occupied
    if let Ok(Some(mut slot)) = state_guard.db.get_parking_slot(&updated_booking.slot_id.to_string()).await {
        slot.status = SlotStatus::Occupied;
        let _ = state_guard.db.save_parking_slot(&slot).await;
    }

    (StatusCode::OK, Json(ApiResponse::success(updated_booking)))
}

// ═══════════════════════════════════════════════════════════════════════════════
// PASSWORD UTILITIES
// ═══════════════════════════════════════════════════════════════════════════════

fn hash_password(password: &str) -> String {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string()
}

fn verify_password(password: &str, hash: &str) -> bool {
    use argon2::{
        password_hash::{PasswordHash, PasswordVerifier},
        Argon2,
    };
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}
