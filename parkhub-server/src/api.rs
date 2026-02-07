//! HTTP API Routes
//!
//! RESTful API for the parking system.

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post, put},
    Extension, Json, Router,
    extract::Multipart,
};
use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};
use base64::Engine;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

use crate::audit::{AuditEntry, AuditEventType};
use axum::http::HeaderMap;
use crate::metrics;
use crate::openapi::ApiDoc;
use crate::rate_limit::EndpointRateLimiters;
use crate::static_files;

use parkhub_common::{
    ApiResponse, AuthTokens, Booking, BookingStatus,
    CreateBookingRequest, HandshakeRequest, HandshakeResponse, HomeofficeDay,
    HomeofficePattern, HomeofficeSettings, LoginRequest,
    ParkingLot, ParkingSlot, RefreshTokenRequest, RegisterRequest, ServerStatus,
    LotStatus, SlotStatus, User, UserRole, Vehicle, AdminStats,
    WaitlistEntry, PushSubscription, RecurrenceRule,
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

// ═══════════════════════════════════════════════════════════════════════════════
// RESPONSE DTOs (exclude password_hash)
// ═══════════════════════════════════════════════════════════════════════════════

/// User response DTO - never exposes password_hash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub name: String,
    pub picture: Option<String>,
    pub phone: Option<String>,
    pub role: UserRole,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
    pub preferences: parkhub_common::UserPreferences,
    pub is_active: bool,
    pub department: Option<String>,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            email: u.email,
            name: u.name,
            picture: u.picture,
            phone: u.phone,
            role: u.role,
            created_at: u.created_at,
            updated_at: u.updated_at,
            last_login: u.last_login,
            preferences: u.preferences,
            is_active: u.is_active,
            department: u.department,
        }
    }
}

/// Login response with safe user DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeLoginResponse {
    pub user: UserResponse,
    pub tokens: AuthTokens,
}

// ═══════════════════════════════════════════════════════════════════════════════
// REQUEST DTOs
// ═══════════════════════════════════════════════════════════════════════════════

/// Create vehicle request - client only provides these fields
#[derive(Debug, Deserialize)]
pub struct CreateVehicleRequest {
    pub license_plate: String,
    pub make: Option<String>,
    pub model: Option<String>,
    pub color: Option<String>,
}

/// Push subscription request - client only provides these fields
#[derive(Debug, Deserialize)]
pub struct CreatePushSubscriptionRequest {
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
}

/// Update booking request
#[derive(Debug, Deserialize)]
pub struct UpdateBookingRequest {
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub notes: Option<String>,
    pub vehicle_id: Option<Uuid>,
    pub license_plate: Option<String>,
}

/// Create the API router with OpenAPI docs and metrics
pub fn create_router(state: SharedState) -> Router {
    let _rate_limiters = Arc::new(EndpointRateLimiters::new());
    let metrics_handle = metrics::init_metrics();

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/health/live", get(liveness_check))
        .route("/health/ready", get(readiness_check))
        .route("/handshake", post(handshake))
        .route("/status", get(server_status))
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/register", post(register))
        .route("/api/v1/auth/refresh", post(refresh_token))
        .route("/api/v1/privacy", get(get_privacy_policy))
        .route("/api/v1/about", get(get_about))
        .route("/api/v1/help", get(get_help))
        .route("/api/v1/setup/change-password", post(setup_change_password))
        .route("/api/v1/setup/status", get(get_setup_status))
        .route("/api/v1/branding", get(get_branding_public))
        .route("/api/v1/branding/logo", get(serve_branding_logo));

    // Protected routes (auth required)
    let protected_routes = Router::new()
        .route("/api/v1/users/me", get(get_current_user))
        .route("/api/v1/users/:id", get(get_user))
        .route("/api/v1/lots", get(list_lots).post(create_lot))
        .route("/api/v1/lots/:id", get(get_lot))
        .route("/api/v1/lots/:id/slots", get(get_lot_slots))
        .route("/api/v1/lots/:id/layout", get(get_lot_layout).put(update_lot_layout))
        .route("/api/v1/bookings", get(list_bookings).post(create_booking))
        .route("/api/v1/bookings/:id", get(get_booking).delete(cancel_booking).patch(update_booking))
        .route("/api/v1/bookings/ical", get(export_ical))
        .route("/api/v1/bookings/:id/checkin", post(checkin_booking))
        .route("/api/v1/vehicles", get(list_vehicles).post(create_vehicle))
        .route("/api/v1/vehicles/:id", delete(delete_vehicle))
        .route("/api/v1/vehicles/:id/photo", post(upload_vehicle_photo))
        .route("/api/v1/homeoffice", get(get_homeoffice_settings).put(update_homeoffice_settings))
        .route("/api/v1/homeoffice/pattern", put(update_homeoffice_pattern))
        .route("/api/v1/homeoffice/days", post(add_homeoffice_day))
        .route("/api/v1/homeoffice/days/:id", delete(remove_homeoffice_day))
        .route("/api/v1/admin/users", get(admin_list_users).post(admin_create_user))
        .route("/api/v1/admin/bookings", get(admin_list_bookings))
        .route("/api/v1/admin/stats", get(admin_stats))
        .route("/api/v1/admin/reports", get(admin_reports))
        .route("/api/v1/lots/:lot_id/slots/:slot_id/qr", get(slot_qr_code))
        .route("/api/v1/lots/:id/waitlist", get(get_waitlist).post(join_waitlist))
        .route("/api/v1/push/subscribe", post(push_subscribe))
        .route("/api/v1/admin/users/:id", patch(admin_update_user))
        .route("/api/v1/admin/slots/:id", patch(admin_update_slot))
        .route("/api/v1/admin/lots/:id", delete(admin_delete_lot))
        .route("/api/v1/users/me/export", get(export_user_data))
        .route("/api/v1/users/me/delete", delete(delete_own_account))
        .route("/api/v1/users/me/password", patch(change_password))
        .route("/api/v1/setup/complete", post(complete_setup))
        .route("/api/v1/admin/branding", get(get_branding_admin).put(update_branding))
        .route("/api/v1/admin/branding/logo", post(upload_branding_logo))
        .route("/api/v1/admin/reset", post(admin_reset_database))
        .route("/api/v1/lots/:lot_id/slots/:slot_id", put(admin_update_slot_properties))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let metrics_handle_clone = metrics_handle.clone();

    // API catch-all: return 404 JSON for unknown /api/* routes
    let api_fallback = Router::new()
        .route("/api/*rest", get(api_not_found).post(api_not_found).put(api_not_found).delete(api_not_found).patch(api_not_found));

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
        .merge(SwaggerUi::new("/swagger-ui").url("/api/openapi.json", ApiDoc::openapi()))
        .merge(api_fallback)
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
// API CATCH-ALL (returns JSON 404 for /api/* routes)
// ═══════════════════════════════════════════════════════════════════════════════

async fn api_not_found() -> (StatusCode, Json<ApiResponse<()>>) {
    (StatusCode::NOT_FOUND, Json(ApiResponse::error("NOT_FOUND", "API endpoint not found")))
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
    let db_stats = state.db.stats().await.unwrap_or_default();
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
) -> (StatusCode, Json<ApiResponse<SafeLoginResponse>>) {
    let state_guard = state.read().await;

    let user = match state_guard.db.get_user_by_username(&request.username).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            match state_guard.db.get_user_by_email(&request.username).await {
                Ok(Some(u)) => u,
                _ => {
                    AuditEntry::new(AuditEventType::LoginFailed)
                        .details(serde_json::json!({"username": &request.username}))
                        .error("Invalid credentials")
                        .log();
                    return (StatusCode::UNAUTHORIZED, Json(ApiResponse::error("INVALID_CREDENTIALS", "Invalid username or password")));
                }
            }
        }
        Err(e) => {
            tracing::error!("Database error during login: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Internal server error")));
        }
    };

    if !verify_password(&request.password, &user.password_hash) {
        AuditEntry::new(AuditEventType::LoginFailed)
            .user(user.id, &user.username)
            .error("Invalid password")
            .log();
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

    AuditEntry::new(AuditEventType::LoginSuccess)
        .user(user.id, &user.username)
        .log();

    metrics::record_auth_event("login", true);

    (
        StatusCode::OK,
        Json(ApiResponse::success(SafeLoginResponse {
            user: UserResponse::from(user),
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
) -> (StatusCode, Json<ApiResponse<SafeLoginResponse>>) {
    let state_guard = state.read().await;

    if let Ok(Some(_)) = state_guard.db.get_user_by_email(&request.email).await {
        return (StatusCode::CONFLICT, Json(ApiResponse::error("EMAIL_EXISTS", "An account with this email already exists")));
    }

    if let Ok(Some(_)) = state_guard.db.get_user_by_username(&request.username).await {
        return (StatusCode::CONFLICT, Json(ApiResponse::error("USERNAME_EXISTS", "This username is already taken")));
    }

    let password_hash = hash_password(&request.password);
    let now = Utc::now();
    let user = User {
        id: Uuid::new_v4(),
        username: request.username,
        email: request.email,
        password_hash,
        name: request.name,
        picture: None,
        phone: None,
        role: UserRole::User,
        created_at: now,
        updated_at: now,
        last_login: Some(now),
        preferences: parkhub_common::UserPreferences::default(),
        is_active: true,
        department: None,
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

    AuditEntry::new(AuditEventType::UserCreated)
        .user(user.id, &user.username)
        .log();

    metrics::record_auth_event("register", true);

    (
        StatusCode::CREATED,
        Json(ApiResponse::success(SafeLoginResponse {
            user: UserResponse::from(user),
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
) -> (StatusCode, Json<ApiResponse<UserResponse>>) {
    let state = state.read().await;
    match state.db.get_user(&auth_user.user_id.to_string()).await {
        Ok(Some(user)) => (StatusCode::OK, Json(ApiResponse::success(UserResponse::from(user)))),
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
) -> (StatusCode, Json<ApiResponse<UserResponse>>) {
    let state = state.read().await;
    match state.db.get_user(&id).await {
        Ok(Some(user)) => (StatusCode::OK, Json(ApiResponse::success(UserResponse::from(user)))),
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


/// Sync ParkingSlot records from a LotLayout
async fn sync_slots_from_layout(db: &crate::db::Database, lot_id: &uuid::Uuid, layout: &parkhub_common::LotLayout) {
    for row in &layout.rows {
        for slot_config in &row.slots {
            let slot_id = uuid::Uuid::parse_str(&slot_config.id).unwrap_or_else(|_| uuid::Uuid::new_v4());
            let slot = ParkingSlot {
                id: slot_id,
                lot_id: *lot_id,
                slot_number: slot_config.number.clone(),
                status: slot_config.status.clone(),
                current_booking: None,
                reserved_for_department: None,
            };
            if let Err(e) = db.save_parking_slot(&slot).await {
                tracing::warn!("Failed to sync slot {}: {}", slot_config.number, e);
            }
        }
    }
}

async fn create_lot(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateLotRequest>,
) -> (StatusCode, Json<ApiResponse<ParkingLot>>) {
    let state_guard = state.read().await;
    let user = match state_guard.db.get_user(&auth_user.user_id.to_string()).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Access denied"))),
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Admin access required")));
    }
    let now = Utc::now();
    let lot = ParkingLot {
        id: Uuid::new_v4(),
        name: req.name,
        address: req.address,
        total_slots: req.total_slots,
        available_slots: req.total_slots,
        layout: None,
        status: req.status.unwrap_or(LotStatus::Open),
        created_at: now,
        updated_at: now,
    };
    if let Err(e) = state_guard.db.save_parking_lot(&lot).await {
        tracing::error!("Failed to save parking lot: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to create parking lot")));
    }
    // Auto-create ParkingSlot records
    if lot.total_slots > 0 {
        for i in 1..=lot.total_slots {
            let slot = ParkingSlot {
                id: Uuid::new_v4(),
                lot_id: lot.id,
                slot_number: format!("{}", i),
                status: SlotStatus::Available,
                current_booking: None,
                reserved_for_department: None,
            };
            if let Err(e) = state_guard.db.save_parking_slot(&slot).await {
                tracing::warn!("Failed to create slot {}: {}", i, e);
            }
        }
    }
    AuditEntry::new(AuditEventType::LotCreated)
        .user(auth_user.user_id, &user.username)
        .resource("lot", &lot.id.to_string())
        .log();
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
) -> (StatusCode, Json<ApiResponse<Option<parkhub_common::LotLayout>>>) {
    let state = state.read().await;
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
    Json(layout): Json<parkhub_common::LotLayout>,
) -> (StatusCode, Json<ApiResponse<parkhub_common::LotLayout>>) {
    let state_guard = state.read().await;
    let user = match state_guard.db.get_user(&auth_user.user_id.to_string()).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Access denied"))),
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Admin access required")));
    }
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
    let _ = state_guard.db.save_lot_layout(&id, &layout).await;
    // Sync ParkingSlot records from updated layout
    if let Ok(lot_uuid) = uuid::Uuid::parse_str(&id) {
        sync_slots_from_layout(&state_guard.db, &lot_uuid, &layout).await;
    }
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

    if let Some(ref dept) = slot.reserved_for_department {
        let user = match state_guard.db.get_user(&auth_user.user_id.to_string()).await {
            Ok(Some(u)) => u,
            _ => return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Access denied"))),
        };
        let user_dept = user.department.as_deref().unwrap_or("");
        if user_dept != dept && user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
            return (StatusCode::FORBIDDEN, Json(ApiResponse::error("DEPARTMENT_RESTRICTED",
                format!("This slot is reserved for department: {}", dept))));
        }
    }

    let lot_name = match state_guard.db.get_parking_lot(&req.lot_id.to_string()).await {
        Ok(Some(lot)) => Some(lot.name),
        _ => None,
    };

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

    let end_time = if let Some(end) = req.end_time {
        end
    } else if let Some(mins) = req.duration_minutes {
        req.start_time + chrono::Duration::minutes(mins as i64)
    } else {
        req.start_time + chrono::Duration::hours(1)
    };

    let now = Utc::now();
    let booking = Booking {
        id: Uuid::new_v4(),
        user_id: auth_user.user_id,
        lot_id: req.lot_id,
        slot_id: req.slot_id,
        booking_type: req.booking_type.unwrap_or_default(),
        dauer_interval: req.dauer_interval,
        lot_name: lot_name.clone(),
        slot_number: Some(slot.slot_number.clone()),
        vehicle_plate: vehicle_plate.clone(),
        start_time: req.start_time,
        end_time,
        status: BookingStatus::Confirmed,
        created_at: now,
        updated_at: now,
        notes: req.notes.clone(),
        recurrence: req.recurrence.clone(),
        checked_in_at: None,
    };

    if let Err(e) = state_guard.db.save_booking(&booking).await {
        tracing::error!("Failed to save booking: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to create booking")));
    }

    let mut updated_slot = slot;
    updated_slot.status = SlotStatus::Reserved;
    let _ = state_guard.db.save_parking_slot(&updated_slot).await;

    // Audit log
    AuditEntry::new(AuditEventType::BookingCreated)
        .user(auth_user.user_id, "")
        .resource("booking", &booking.id.to_string())
        .log();
    metrics::record_booking_event("created");

    // Email confirmation
    if let Some(ref email_svc) = state_guard.email {
        if let Ok(Some(user)) = state_guard.db.get_user(&auth_user.user_id.to_string()).await {
            let _ = email_svc.send_booking_confirmation(
                &user.email, &user.name,
                booking.slot_number.as_deref().unwrap_or("?"),
                booking.lot_name.as_deref().unwrap_or("Lot"),
                &booking.start_time.to_rfc3339(), &booking.end_time.to_rfc3339(),
            ).await;
        }
    }

    // Expand recurring bookings
    if let Some(ref recurrence) = booking.recurrence {
        let start_date = booking.start_time.date_naive();
        let end_date = chrono::NaiveDate::parse_from_str(&recurrence.until, "%Y-%m-%d")
            .unwrap_or(start_date + chrono::Duration::days(90));
        let duration = booking.end_time - booking.start_time;
        let time_of_day = booking.start_time.time();
        let mut current = start_date + chrono::Duration::days(1);
        while current <= end_date {
            let weekday = current.weekday().num_days_from_monday() as u8;
            if recurrence.weekdays.contains(&weekday) {
                let cs = current.and_time(time_of_day).and_utc();
                let ce = cs + duration;
                let child = Booking {
                    id: Uuid::new_v4(), user_id: auth_user.user_id,
                    lot_id: booking.lot_id, slot_id: booking.slot_id,
                    booking_type: booking.booking_type.clone(),
                    dauer_interval: booking.dauer_interval.clone(),
                    lot_name: booking.lot_name.clone(),
                    slot_number: booking.slot_number.clone(),
                    vehicle_plate: booking.vehicle_plate.clone(),
                    start_time: cs, end_time: ce,
                    status: BookingStatus::Confirmed,
                    created_at: now, updated_at: now,
                    notes: booking.notes.clone(),
                    recurrence: Some(RecurrenceRule {
                        weekdays: recurrence.weekdays.clone(),
                        until: recurrence.until.clone(),
                        parent_id: Some(booking.id),
                    }),
                    checked_in_at: None,
                };
                let _ = state_guard.db.save_booking(&child).await;
            }
            current += chrono::Duration::days(1);
        }
    }

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

/// PATCH /api/v1/bookings/:id - Update a booking
async fn update_booking(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(req): Json<UpdateBookingRequest>,
) -> (StatusCode, Json<ApiResponse<Booking>>) {
    let state_guard = state.read().await;
    let mut booking = match state_guard.db.get_booking(&id).await {
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

    if booking.status == BookingStatus::Cancelled || booking.status == BookingStatus::Completed {
        return (StatusCode::CONFLICT, Json(ApiResponse::error("BOOKING_NOT_MODIFIABLE", "Cannot modify a cancelled or completed booking")));
    }

    if let Some(start) = req.start_time {
        booking.start_time = start;
    }
    if let Some(end) = req.end_time {
        booking.end_time = end;
    }
    if let Some(notes) = req.notes {
        booking.notes = Some(notes);
    }
    if let Some(plate) = req.license_plate {
        booking.vehicle_plate = Some(plate);
    } else if let Some(vid) = req.vehicle_id {
        if let Ok(Some(v)) = state_guard.db.get_vehicle(&vid.to_string()).await {
            booking.vehicle_plate = Some(v.plate);
        }
    }
    booking.updated_at = Utc::now();

    if let Err(e) = state_guard.db.save_booking(&booking).await {
        tracing::error!("Failed to update booking: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to update booking")));
    }

    AuditEntry::new(AuditEventType::BookingUpdated)
        .user(auth_user.user_id, "")
        .resource("booking", &booking.id.to_string())
        .log();

    (StatusCode::OK, Json(ApiResponse::success(booking)))
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

    if let Ok(Some(mut slot)) = state_guard.db.get_parking_slot(&booking.slot_id.to_string()).await {
        slot.status = SlotStatus::Available;
        let _ = state_guard.db.save_parking_slot(&slot).await;
    }

    AuditEntry::new(AuditEventType::BookingCancelled)
        .user(auth_user.user_id, "")
        .resource("booking", &booking.id.to_string())
        .log();
    metrics::record_booking_event("cancelled");

    // Send cancellation email
    if let Some(ref email_svc) = state_guard.email {
        if let Ok(Some(user)) = state_guard.db.get_user(&auth_user.user_id.to_string()).await {
            let _ = email_svc.send_auto_release_notification(
                &user.email, &user.name,
                booking.slot_number.as_deref().unwrap_or("?"),
                booking.lot_name.as_deref().unwrap_or("Lot"),
            ).await;
        }
    }

    // Notify waitlist
    let date = booking.start_time.format("%Y-%m-%d").to_string();
    if let Ok(waitlist) = state_guard.db.list_waitlist_by_lot(&booking.lot_id.to_string(), Some(&date)).await {
        if let Some(first) = waitlist.first() {
            if !first.notified {
                if let Some(ref email_svc) = state_guard.email {
                    if let Ok(Some(wu)) = state_guard.db.get_user(&first.user_id.to_string()).await {
                        let _ = email_svc.send_waitlist_notification(&wu.email, &wu.name, booking.lot_name.as_deref().unwrap_or("Lot"), &date).await;
                    }
                }
                let mut ue = first.clone();
                ue.notified = true;
                let _ = state_guard.db.save_waitlist_entry(&ue).await;
            }
        }
    }

    (StatusCode::OK, Json(ApiResponse::success(())))
}

// ═══════════════════════════════════════════════════════════════════════════════
// VEHICLES (with proper CreateVehicleRequest DTO)
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
    Json(req): Json<CreateVehicleRequest>,
) -> (StatusCode, Json<ApiResponse<Vehicle>>) {
    let vehicle = Vehicle {
        id: Uuid::new_v4(),
        user_id: auth_user.user_id,
        plate: req.license_plate,
        make: req.make,
        model: req.model,
        color: req.color,
        is_default: false,
        photo_url: None,
        created_at: Utc::now(),
    };

    let state_guard = state.read().await;
    if let Err(e) = state_guard.db.save_vehicle(&vehicle).await {
        tracing::error!("Failed to save vehicle: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to create vehicle")));
    }
    (StatusCode::CREATED, Json(ApiResponse::success(vehicle)))
}

async fn delete_vehicle(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let state_guard = state.read().await;
    // Verify ownership
    match state_guard.db.get_vehicle(&id).await {
        Ok(Some(v)) if v.user_id == auth_user.user_id => {}
        Ok(Some(_)) => return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Access denied"))),
        Ok(None) => return (StatusCode::NOT_FOUND, Json(ApiResponse::error("NOT_FOUND", "Vehicle not found"))),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Internal server error")));
        }
    }
    match state_guard.db.delete_vehicle(&id).await {
        Ok(true) => (StatusCode::OK, Json(ApiResponse::success(()))),
        Ok(false) => (StatusCode::NOT_FOUND, Json(ApiResponse::error("NOT_FOUND", "Vehicle not found"))),
        Err(e) => {
            tracing::error!("Failed to delete vehicle: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to delete vehicle")))
        }
    }
}

async fn upload_vehicle_photo(
    State(_state): State<SharedState>,
    Extension(_auth_user): Extension<AuthUser>,
    Path(_id): Path<String>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
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


/// Wrapper for PUT /api/v1/homeoffice that accepts {"pattern": {"weekdays": [...]}}
#[derive(Debug, Deserialize)]
pub struct UpdateHomeofficeRequest {
    pub pattern: Option<HomeofficePatternInput>,
    pub weekdays: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
pub struct HomeofficePatternInput {
    pub weekdays: Vec<serde_json::Value>,
}

fn parse_weekdays(vals: &[serde_json::Value]) -> Vec<u8> {
    vals.iter().filter_map(|v| {
        match v {
            serde_json::Value::Number(n) => n.as_u64().map(|n| n as u8),
            serde_json::Value::String(s) => match s.to_lowercase().as_str() {
                "monday" | "mon" => Some(0),
                "tuesday" | "tue" => Some(1),
                "wednesday" | "wed" => Some(2),
                "thursday" | "thu" => Some(3),
                "friday" | "fri" => Some(4),
                "saturday" | "sat" => Some(5),
                "sunday" | "sun" => Some(6),
                _ => s.parse::<u8>().ok(),
            },
            _ => None,
        }
    }).collect()
}

async fn update_homeoffice_settings(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<UpdateHomeofficeRequest>,
) -> (StatusCode, Json<ApiResponse<HomeofficeSettings>>) {
    let pattern = if let Some(p) = req.pattern {
        HomeofficePattern { weekdays: parse_weekdays(&p.weekdays) }
    } else if let Some(weekdays) = req.weekdays {
        HomeofficePattern { weekdays: parse_weekdays(&weekdays) }
    } else {
        HomeofficePattern { weekdays: vec![] }
    };
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


/// POST /api/v1/admin/users - Admin creates a new user
async fn admin_create_user(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<AdminCreateUserRequest>,
) -> (StatusCode, Json<ApiResponse<UserResponse>>) {
    let state_guard = state.read().await;
    let user = match state_guard.db.get_user(&auth_user.user_id.to_string()).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Access denied"))),
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Admin access required")));
    }
    if let Ok(Some(_)) = state_guard.db.get_user_by_username(&req.username).await {
        return (StatusCode::CONFLICT, Json(ApiResponse::error("USERNAME_EXISTS", "Username already taken")));
    }
    let password_hash = hash_password(&req.password);
    let now = Utc::now();
    let new_user = User {
        id: Uuid::new_v4(),
        username: req.username,
        email: req.email,
        password_hash,
        name: req.name,
        picture: None,
        phone: None,
        role: req.role,
        created_at: now,
        updated_at: now,
        last_login: None,
        preferences: parkhub_common::UserPreferences::default(),
        is_active: true,
        department: None,
    };
    if let Err(e) = state_guard.db.save_user(&new_user).await {
        tracing::error!("Failed to create user: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to create user")));
    }
    AuditEntry::new(AuditEventType::UserCreated)
        .user(auth_user.user_id, &user.username)
        .resource("user", &new_user.id.to_string())
        .log();
    (StatusCode::CREATED, Json(ApiResponse::success(UserResponse::from(new_user))))
}

async fn admin_list_users(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<ApiResponse<Vec<UserResponse>>>) {
    let state_guard = state.read().await;
    let user = match state_guard.db.get_user(&auth_user.user_id.to_string()).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Access denied"))),
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Admin access required")));
    }
    match state_guard.db.list_users().await {
        Ok(users) => {
            let safe_users: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();
            (StatusCode::OK, Json(ApiResponse::success(safe_users)))
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
        [(header::CONTENT_TYPE, "text/calendar; charset=utf-8")],
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
    updated_booking.checked_in_at = Some(Utc::now());

    if let Err(e) = state_guard.db.save_booking(&updated_booking).await {
        tracing::error!("Failed to update booking: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to check in")));
    }

    if let Ok(Some(mut slot)) = state_guard.db.get_parking_slot(&updated_booking.slot_id.to_string()).await {
        slot.status = SlotStatus::Occupied;
        let _ = state_guard.db.save_parking_slot(&slot).await;
    }

    AuditEntry::new(AuditEventType::CheckIn)
        .user(auth_user.user_id, "")
        .resource("booking", &updated_booking.id.to_string())
        .log();

    (StatusCode::OK, Json(ApiResponse::success(updated_booking)))
}

// ═══════════════════════════════════════════════════════════════════════════════
// ADMIN REPORTS
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
struct ReportParams {
    format: Option<String>,
}

async fn admin_reports(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(params): Query<ReportParams>,
) -> impl IntoResponse {
    let state_guard = state.read().await;
    let user = match state_guard.db.get_user(&auth_user.user_id.to_string()).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::FORBIDDEN, [(header::CONTENT_TYPE, "application/json")],
            r#"{"success":false,"error":{"code":"FORBIDDEN","message":"Access denied"}}"#.to_string()),
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (StatusCode::FORBIDDEN, [(header::CONTENT_TYPE, "application/json")],
            r#"{"success":false,"error":{"code":"FORBIDDEN","message":"Admin access required"}}"#.to_string());
    }

    let bookings = state_guard.db.list_bookings().await.unwrap_or_default();
    let lots = state_guard.db.list_parking_lots().await.unwrap_or_default();
    let db_stats = state_guard.db.stats().await.unwrap_or_default();

    let now = Utc::now();
    let today = now.date_naive();
    let active = bookings.iter().filter(|b| b.status == BookingStatus::Active || b.status == BookingStatus::Confirmed).count();
    let today_count = bookings.iter().filter(|b| b.start_time.date_naive() == today).count();
    let total_slots: i32 = lots.iter().map(|l| l.total_slots).sum();
    let occupancy_pct = if total_slots > 0 { active as f64 / total_slots as f64 * 100.0 } else { 0.0 };

    let format = params.format.as_deref().unwrap_or("json");
    if format == "csv" {
        let mut csv = String::from("metric,value\n");
        csv.push_str(&format!("total_users,{}\n", db_stats.users));
        csv.push_str(&format!("total_bookings,{}\n", db_stats.bookings));
        csv.push_str(&format!("active_bookings,{}\n", active));
        csv.push_str(&format!("bookings_today,{}\n", today_count));
        csv.push_str(&format!("total_lots,{}\n", db_stats.parking_lots));
        csv.push_str(&format!("total_slots,{}\n", total_slots));
        csv.push_str(&format!("occupancy_percent,{:.1}\n", occupancy_pct));
        return (StatusCode::OK, [(header::CONTENT_TYPE, "text/csv")], csv);
    }

    let report = serde_json::json!({
        "generated_at": now.to_rfc3339(),
        "total_users": db_stats.users,
        "total_bookings": db_stats.bookings,
        "active_bookings": active,
        "bookings_today": today_count,
        "total_lots": db_stats.parking_lots,
        "total_slots": total_slots,
        "occupancy_percent": format!("{:.1}", occupancy_pct),
        "lots": lots.iter().map(|l| serde_json::json!({
            "id": l.id,
            "name": l.name,
            "total_slots": l.total_slots,
            "available_slots": l.available_slots,
        })).collect::<Vec<_>>(),
    });

    (StatusCode::OK, [(header::CONTENT_TYPE, "application/json")],
        serde_json::json!({"success": true, "data": report}).to_string())
}

// ═══════════════════════════════════════════════════════════════════════════════
// QR CODE
// ═══════════════════════════════════════════════════════════════════════════════

async fn slot_qr_code(
    State(state): State<SharedState>,
    Extension(_auth_user): Extension<AuthUser>,
    Path((lot_id, slot_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let state_guard = state.read().await;

    match state_guard.db.get_parking_lot(&lot_id).await {
        Ok(Some(_)) => {}
        _ => return (StatusCode::NOT_FOUND, [(header::CONTENT_TYPE, "application/json")],
            r#"{"success":false,"error":{"code":"NOT_FOUND","message":"Lot not found"}}"#.to_string()),
    };

    match state_guard.db.get_parking_slot(&slot_id).await {
        Ok(Some(_)) => {}
        _ => return (StatusCode::NOT_FOUND, [(header::CONTENT_TYPE, "application/json")],
            r#"{"success":false,"error":{"code":"NOT_FOUND","message":"Slot not found"}}"#.to_string()),
    };

    let booking_url = format!("/book?lot={}&slot={}", lot_id, slot_id);
    let qr = match qrcode::QrCode::new(booking_url.as_bytes()) {
        Ok(q) => q,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, [(header::CONTENT_TYPE, "application/json")],
            r#"{"success":false,"error":{"code":"SERVER_ERROR","message":"Failed to generate QR code"}}"#.to_string()),
    };

    let svg = qr.render::<qrcode::render::svg::Color>().quiet_zone(true).min_dimensions(256, 256).build();
    (StatusCode::OK, [(header::CONTENT_TYPE, "image/svg+xml")], svg)
}

// ═══════════════════════════════════════════════════════════════════════════════
// WAITLIST
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
struct WaitlistParams {
    date: Option<String>,
}

async fn join_waitlist(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(lot_id): Path<String>,
    Query(params): Query<WaitlistParams>,
) -> (StatusCode, Json<ApiResponse<WaitlistEntry>>) {
    let date = params.date.unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string());
    let state_guard = state.read().await;
    match state_guard.db.get_parking_lot(&lot_id).await {
        Ok(Some(_)) => {}
        _ => return (StatusCode::NOT_FOUND, Json(ApiResponse::error("NOT_FOUND", "Lot not found"))),
    };
    let entry = WaitlistEntry {
        id: Uuid::new_v4(),
        lot_id: lot_id.parse().unwrap_or_default(),
        user_id: auth_user.user_id,
        date,
        created_at: Utc::now(),
        notified: false,
    };
    if let Err(e) = state_guard.db.save_waitlist_entry(&entry).await {
        tracing::error!("Failed to save waitlist entry: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to join waitlist")));
    }
    (StatusCode::CREATED, Json(ApiResponse::success(entry)))
}

async fn get_waitlist(
    State(state): State<SharedState>,
    Path(lot_id): Path<String>,
    Query(params): Query<WaitlistParams>,
) -> Json<ApiResponse<Vec<WaitlistEntry>>> {
    let state_guard = state.read().await;
    match state_guard.db.list_waitlist_by_lot(&lot_id, params.date.as_deref()).await {
        Ok(entries) => Json(ApiResponse::success(entries)),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            Json(ApiResponse::error("SERVER_ERROR", "Failed to get waitlist"))
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PUSH SUBSCRIBE (with proper DTO)
// ═══════════════════════════════════════════════════════════════════════════════

async fn push_subscribe(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreatePushSubscriptionRequest>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let sub = PushSubscription {
        user_id: auth_user.user_id,
        endpoint: req.endpoint,
        p256dh: req.p256dh,
        auth: req.auth,
        created_at: Utc::now(),
    };
    let state_guard = state.read().await;
    if let Err(e) = state_guard.db.save_push_subscription(&sub).await {
        tracing::error!("Failed to save push subscription: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to save subscription")));
    }
    (StatusCode::CREATED, Json(ApiResponse::success(())))
}

// ═══════════════════════════════════════════════════════════════════════════════
// ADMIN USER/SLOT UPDATE
// ═══════════════════════════════════════════════════════════════════════════════

async fn admin_update_user(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(user_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> (StatusCode, Json<ApiResponse<UserResponse>>) {
    let state_guard = state.read().await;
    let admin = match state_guard.db.get_user(&auth_user.user_id.to_string()).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Access denied"))),
    };
    if admin.role != UserRole::Admin && admin.role != UserRole::SuperAdmin {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Admin access required")));
    }

    let mut user = match state_guard.db.get_user(&user_id).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::NOT_FOUND, Json(ApiResponse::error("NOT_FOUND", "User not found"))),
    };

    if let Some(dept) = body.get("department") {
        user.department = dept.as_str().map(|s| s.to_string());
    }
    if let Some(role) = body.get("role").and_then(|r| r.as_str()) {
        match role {
            "user" => user.role = UserRole::User,
            "admin" => user.role = UserRole::Admin,
            "superadmin" => user.role = UserRole::SuperAdmin,
            _ => {}
        }
    }
    if let Some(active) = body.get("is_active").and_then(|a| a.as_bool()) {
        user.is_active = active;
    }

    user.updated_at = Utc::now();

    if let Err(e) = state_guard.db.save_user(&user).await {
        tracing::error!("Failed to update user: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to update user")));
    }

    AuditEntry::new(AuditEventType::UserUpdated)
        .user(auth_user.user_id, &admin.username)
        .resource("user", &user_id)
        .log();

    (StatusCode::OK, Json(ApiResponse::success(UserResponse::from(user))))
}

#[derive(Debug, Deserialize)]
struct AdminUpdateSlotRequest {
    reserved_for_department: Option<String>,
}

async fn admin_update_slot(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(slot_id): Path<String>,
    Json(req): Json<AdminUpdateSlotRequest>,
) -> (StatusCode, Json<ApiResponse<ParkingSlot>>) {
    let state_guard = state.read().await;
    let user = match state_guard.db.get_user(&auth_user.user_id.to_string()).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Access denied"))),
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Admin access required")));
    }
    let mut slot = match state_guard.db.get_parking_slot(&slot_id).await {
        Ok(Some(s)) => s,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(ApiResponse::error("NOT_FOUND", "Slot not found"))),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Internal server error")));
        }
    };
    slot.reserved_for_department = req.reserved_for_department;
    if let Err(e) = state_guard.db.save_parking_slot(&slot).await {
        tracing::error!("Failed to update slot: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to update slot")));
    }
    (StatusCode::OK, Json(ApiResponse::success(slot)))
}


// ═══════════════════════════════════════════════════════════════════════════════
// GDPR / DSGVO ENDPOINTS (Art. 15, 17)
// ═══════════════════════════════════════════════════════════════════════════════

/// GET /api/v1/users/me/export - Export all user data (DSGVO Art. 15)
async fn export_user_data(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let state = state.read().await;
    let uid = auth_user.user_id.to_string();

    let user = match state.db.get_user(&uid).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::NOT_FOUND, Json(ApiResponse::error("NOT_FOUND", "User not found"))),
    };

    let bookings = state.db.list_bookings_by_user(&uid).await.unwrap_or_default();
    let vehicles = state.db.list_vehicles_by_user(&uid).await.unwrap_or_default();
    let homeoffice = state.db.get_homeoffice_settings(&uid).await.ok().flatten();
    let push_subs = state.db.list_push_subscriptions_by_user(&uid).await.unwrap_or_default();

    let export = serde_json::json!({
        "exported_at": Utc::now().to_rfc3339(),
        "gdpr_article": "Art. 15 DSGVO - Right of access",
        "profile": {
            "id": user.id,
            "username": user.username,
            "email": user.email,
            "name": user.name,
            "phone": user.phone,
            "role": user.role,
            "department": user.department,
            "created_at": user.created_at,
            "updated_at": user.updated_at,
            "last_login": user.last_login,
            "is_active": user.is_active,
        },
        "preferences": user.preferences,
        "bookings": bookings,
        "vehicles": vehicles,
        "homeoffice_settings": homeoffice,
        "push_subscriptions": push_subs.iter().map(|s| serde_json::json!({
            "endpoint": s.endpoint,
            "created_at": s.created_at,
        })).collect::<Vec<_>>(),
    });

    AuditEntry::new(AuditEventType::UserUpdated)
        .user(auth_user.user_id, &user.username)
        .details(serde_json::json!({"action": "gdpr_data_export"}))
        .log();

    (StatusCode::OK, Json(ApiResponse::success(export)))
}

/// DELETE /api/v1/users/me/delete - Delete own account and all data (DSGVO Art. 17)
async fn delete_own_account(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let state = state.read().await;
    let uid = auth_user.user_id.to_string();

    let user = match state.db.get_user(&uid).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::NOT_FOUND, Json(ApiResponse::error("NOT_FOUND", "User not found"))),
    };

    // Prevent SuperAdmin self-deletion
    if user.role == UserRole::SuperAdmin {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "SuperAdmin accounts cannot be self-deleted. Contact another admin.")));
    }

    let mut deleted = serde_json::Map::new();

    // Delete bookings
    let bookings = state.db.list_bookings_by_user(&uid).await.unwrap_or_default();
    for b in &bookings {
        let _ = state.db.delete_booking(&b.id.to_string()).await;
    }
    deleted.insert("bookings".into(), serde_json::json!(bookings.len()));

    // Delete vehicles
    let vehicles = state.db.list_vehicles_by_user(&uid).await.unwrap_or_default();
    for v in &vehicles {
        let _ = state.db.delete_vehicle(&v.id.to_string()).await;
    }
    deleted.insert("vehicles".into(), serde_json::json!(vehicles.len()));

    // Delete homeoffice settings
    let _ = state.db.delete_homeoffice_settings(&uid).await;
    deleted.insert("homeoffice_settings".into(), serde_json::json!(true));

    // Delete push subscriptions
    let push_count = state.db.delete_push_subscriptions_by_user(&uid).await.unwrap_or(0);
    deleted.insert("push_subscriptions".into(), serde_json::json!(push_count));

    // Delete waitlist entries
    let waitlist_count = state.db.delete_waitlist_entries_by_user(&uid).await.unwrap_or(0);
    deleted.insert("waitlist_entries".into(), serde_json::json!(waitlist_count));

    // Finally delete the user
    let _ = state.db.delete_user(&uid).await;
    deleted.insert("user_account".into(), serde_json::json!(true));

    AuditEntry::new(AuditEventType::UserDeleted)
        .user(auth_user.user_id, &user.username)
        .details(serde_json::json!({"action": "gdpr_account_deletion", "deleted": deleted.clone()}))
        .log();

    (StatusCode::OK, Json(ApiResponse::success(serde_json::json!({
        "message": "Account and all associated data have been permanently deleted (DSGVO Art. 17)",
        "deleted": deleted,
    }))))
}

/// GET /api/v1/privacy - Return privacy policy info
async fn get_privacy_policy(
    State(state): State<SharedState>,
) -> Json<ApiResponse<serde_json::Value>> {
    let state = state.read().await;
    let policy_url = if state.config.privacy_policy_url.is_empty() {
        std::env::var("PARKHUB_PRIVACY_POLICY_URL").unwrap_or_default()
    } else {
        state.config.privacy_policy_url.clone()
    };

    Json(ApiResponse::success(serde_json::json!({
        "privacy_policy_url": if policy_url.is_empty() { None } else { Some(&policy_url) },
        "data_processing": {
            "purpose": "Parking space management for employees",
            "legal_basis": "Art. 6(1)(b) DSGVO - Contract fulfillment / Art. 6(1)(f) - Legitimate interest",
            "data_categories": [
                "Name, email, phone (profile)",
                "Vehicle license plates",
                "Booking history",
                "Home office schedule",
            ],
            "retention": "Data is stored until account deletion or as required by law",
            "your_rights": [
                "Art. 15 - Right of access (GET /api/v1/users/me/export)",
                "Art. 16 - Right to rectification (PATCH /api/v1/users/me)",
                "Art. 17 - Right to erasure (DELETE /api/v1/users/me)",
                "Art. 20 - Right to data portability (GET /api/v1/users/me/export)",
            ],
        },
        "self_hosted": true,
        "third_party_sharing": false,
        "encryption_at_rest": true,
    })))
}

/// GET /api/v1/about - Return app info
async fn get_about() -> Json<ApiResponse<serde_json::Value>> {
    Json(ApiResponse::success(serde_json::json!({
        "name": "ParkHub",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Open-source parking management for companies",
        "license": "MIT",
        "repository": "https://github.com/nash87/parkhub",
        "tech_stack": {
            "backend": "Rust (Axum)",
            "frontend": "React (TypeScript, Tailwind CSS)",
            "database": "redb (embedded, pure Rust)",
            "discovery": "mDNS/DNS-SD",
        },
        "data_storage": {
            "type": "Embedded database (redb)",
            "location": "Local filesystem",
            "encryption": "Optional AES-256-GCM at rest",
            "backup": "Automatic daily backups",
        },
        "features": [
            "Self-hosted / on-premise",
            "Single binary deployment",
            "GDPR/DSGVO compliant",
            "End-to-end encryption",
            "LAN autodiscovery (mDNS)",
            "PWA support",
        ],
    })))
}

// ═══════════════════════════════════════════════════════════════════════════════
// SETUP / ONBOARDING ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// GET /api/v1/setup/status - Check if initial setup is complete
async fn get_setup_status(
    State(state): State<SharedState>,
) -> Json<ApiResponse<serde_json::Value>> {
    let state = state.read().await;
    let is_fresh = state.db.is_fresh().await.unwrap_or(true);
    let has_lots = state.db.list_parking_lots().await.map(|l| !l.is_empty()).unwrap_or(false);
    let stats = state.db.stats().await.unwrap_or_default();

    // Check if admin still has default password
    let needs_password_change = if let Ok(Some(admin)) = state.db.get_user_by_username("admin").await {
        // Check if password is still "admin"
        crate::api::check_default_password(&admin.password_hash)
    } else {
        false
    };

    Json(ApiResponse::success(serde_json::json!({
        "setup_complete": !is_fresh,
        "needs_password_change": needs_password_change,
        "has_parking_lots": has_lots,
        "has_users": stats.users > 1,
        "total_users": stats.users,
        "total_lots": stats.parking_lots,
    })))
}

/// POST /api/v1/setup/complete - Mark setup as complete (admin only)
async fn complete_setup(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let state = state.read().await;
    let user = match state.db.get_user(&auth_user.user_id.to_string()).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Access denied"))),
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Admin access required")));
    }

    if let Err(e) = state.db.mark_setup_completed().await {
        tracing::error!("Failed to mark setup complete: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to complete setup")));
    }

    AuditEntry::new(AuditEventType::ConfigChanged)
        .user(auth_user.user_id, &user.username)
        .details(serde_json::json!({"action": "setup_completed"}))
        .log();

    (StatusCode::OK, Json(ApiResponse::success(serde_json::json!({
        "message": "Setup marked as complete",
    }))))
}

/// PATCH /api/v1/users/me/password - Change own password
#[derive(Debug, Deserialize)]
struct ChangePasswordRequest {
    current_password: String,
    new_password: String,
}

async fn change_password(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<ChangePasswordRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let state = state.read().await;
    let mut user = match state.db.get_user(&auth_user.user_id.to_string()).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::NOT_FOUND, Json(ApiResponse::error("NOT_FOUND", "User not found"))),
    };

    if !verify_password(&req.current_password, &user.password_hash) {
        AuditEntry::new(AuditEventType::PasswordChanged)
            .user(auth_user.user_id, &user.username)
            .error("Invalid current password")
            .log();
        return (StatusCode::UNAUTHORIZED, Json(ApiResponse::error("INVALID_PASSWORD", "Current password is incorrect")));
    }

    if req.new_password.len() < 8 {
        return (StatusCode::BAD_REQUEST, Json(ApiResponse::error("WEAK_PASSWORD", "Password must be at least 8 characters")));
    }

    user.password_hash = hash_password(&req.new_password);
    user.updated_at = Utc::now();

    if let Err(e) = state.db.save_user(&user).await {
        tracing::error!("Failed to update password: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to update password")));
    }

    AuditEntry::new(AuditEventType::PasswordChanged)
        .user(auth_user.user_id, &user.username)
        .log();

    (StatusCode::OK, Json(ApiResponse::success(serde_json::json!({
        "message": "Password changed successfully",
    }))))
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELP / FAQ ENDPOINT
// ═══════════════════════════════════════════════════════════════════════════════

/// GET /api/v1/help - Return help content based on Accept-Language
async fn get_help(headers: HeaderMap) -> Json<ApiResponse<serde_json::Value>> {
    let lang = headers
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("en");

    let is_german = lang.starts_with("de");

    if is_german {
        Json(ApiResponse::success(serde_json::json!({
            "language": "de",
            "title": "Hilfe & FAQ",
            "sections": [
                {
                    "title": "Erste Schritte",
                    "items": [
                        {"q": "Wie buche ich einen Parkplatz?", "a": "Öffnen Sie die App, wählen Sie ein Parkhaus und einen freien Platz. Wählen Sie Datum und Uhrzeit und bestätigen Sie die Buchung."},
                        {"q": "Wie registriere ich mein Fahrzeug?", "a": "Gehen Sie zu 'Meine Fahrzeuge' und tippen Sie auf '+'. Geben Sie das Kennzeichen und optional Marke/Modell ein."},
                        {"q": "Was ist der Home-Office-Modus?", "a": "Unter 'Home Office' können Sie Ihre regelmäßigen Home-Office-Tage einstellen. An diesen Tagen wird Ihr Parkplatz automatisch für Kollegen freigegeben."},
                    ]
                },
                {
                    "title": "Buchungen",
                    "items": [
                        {"q": "Kann ich eine Buchung stornieren?", "a": "Ja, öffnen Sie die Buchung und tippen Sie auf 'Stornieren'. Der Platz wird sofort freigegeben."},
                        {"q": "Was ist eine Dauerbuchung?", "a": "Eine Dauerbuchung reserviert einen Platz für wiederkehrende Tage (z.B. jeden Mo-Fr). Ideal für regelmäßige Bürotage."},
                        {"q": "Wie funktioniert die Warteliste?", "a": "Wenn alle Plätze belegt sind, können Sie sich auf die Warteliste setzen. Sie werden per E-Mail benachrichtigt, sobald ein Platz frei wird."},
                    ]
                },
                {
                    "title": "Datenschutz (DSGVO)",
                    "items": [
                        {"q": "Welche Daten werden gespeichert?", "a": "Name, E-Mail, Fahrzeugkennzeichen, Buchungshistorie und Home-Office-Einstellungen. Alle Daten werden lokal auf dem Firmenserver gespeichert."},
                        {"q": "Wie kann ich meine Daten exportieren?", "a": "Unter 'Profil' > 'Daten exportieren' können Sie alle Ihre Daten als JSON herunterladen (Art. 15 DSGVO)."},
                        {"q": "Wie lösche ich mein Konto?", "a": "Unter 'Profil' > 'Konto löschen' werden alle Ihre Daten unwiderruflich gelöscht (Art. 17 DSGVO)."},
                    ]
                },
            ]
        })))
    } else {
        Json(ApiResponse::success(serde_json::json!({
            "language": "en",
            "title": "Help & FAQ",
            "sections": [
                {
                    "title": "Getting Started",
                    "items": [
                        {"q": "How do I book a parking spot?", "a": "Open the app, select a parking lot and an available slot. Choose your date and time, then confirm the booking."},
                        {"q": "How do I register my vehicle?", "a": "Go to 'My Vehicles' and tap '+'. Enter your license plate and optionally the make/model."},
                        {"q": "What is Home Office mode?", "a": "Under 'Home Office', set your regular work-from-home days. On those days, your parking spot is automatically released for colleagues."},
                    ]
                },
                {
                    "title": "Bookings",
                    "items": [
                        {"q": "Can I cancel a booking?", "a": "Yes, open the booking and tap 'Cancel'. The spot will be released immediately."},
                        {"q": "What is a permanent booking?", "a": "A permanent booking reserves a spot for recurring days (e.g., Mon-Fri every week). Ideal for regular office days."},
                        {"q": "How does the waitlist work?", "a": "When all spots are taken, you can join the waitlist. You'll be notified by email when a spot becomes available."},
                    ]
                },
                {
                    "title": "Privacy (GDPR)",
                    "items": [
                        {"q": "What data is stored?", "a": "Name, email, vehicle plates, booking history, and home office settings. All data is stored locally on your company server."},
                        {"q": "How can I export my data?", "a": "Under 'Profile' > 'Export Data', you can download all your data as JSON (GDPR Art. 15)."},
                        {"q": "How do I delete my account?", "a": "Under 'Profile' > 'Delete Account', all your data will be permanently erased (GDPR Art. 17)."},
                    ]
                },
            ]
        })))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ADMIN DELETE LOT
// ═══════════════════════════════════════════════════════════════════════════════

async fn admin_delete_lot(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let state_guard = state.read().await;
    let user = match state_guard.db.get_user(&auth_user.user_id.to_string()).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Access denied"))),
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Admin access required")));
    }

    // Check lot exists
    let lot = match state_guard.db.get_parking_lot(&id).await {
        Ok(Some(l)) => l,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(ApiResponse::error("NOT_FOUND", "Parking lot not found"))),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Internal server error")));
        }
    };

    // Delete associated slots
    let slots = state_guard.db.list_slots_by_lot(&id).await.unwrap_or_default();
    let mut deleted_slots = 0u64;
    for slot in &slots {
        if let Ok(true) = state_guard.db.delete_parking_slot(&slot.id.to_string()).await {
            deleted_slots += 1;
        }
    }

    // Delete associated bookings for this lot
    let bookings = state_guard.db.list_bookings().await.unwrap_or_default();
    let mut deleted_bookings = 0u64;
    for booking in &bookings {
        if booking.lot_id.to_string() == id {
            if let Ok(true) = state_guard.db.delete_booking(&booking.id.to_string()).await {
                deleted_bookings += 1;
            }
        }
    }

    // Delete lot layout
    let _ = state_guard.db.delete_lot_layout(&id).await;

    // Delete the lot itself
    match state_guard.db.delete_parking_lot(&id).await {
        Ok(true) => {
            AuditEntry::new(AuditEventType::ConfigChanged)
                .user(auth_user.user_id, &user.username)
                .resource("lot", &id)
                .details(serde_json::json!({
                    "action": "lot_deleted",
                    "lot_name": lot.name,
                    "deleted_slots": deleted_slots,
                    "deleted_bookings": deleted_bookings,
                }))
                .log();

            (StatusCode::OK, Json(ApiResponse::success(serde_json::json!({
                "message": "Parking lot deleted successfully",
                "deleted_slots": deleted_slots,
                "deleted_bookings": deleted_bookings,
            }))))
        }
        Ok(false) => (StatusCode::NOT_FOUND, Json(ApiResponse::error("NOT_FOUND", "Parking lot not found"))),
        Err(e) => {
            tracing::error!("Failed to delete parking lot: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to delete parking lot")))
        }
    }
}

/// Check if a password hash matches the default "admin" password
pub fn check_default_password(hash: &str) -> bool {
    verify_password("admin", hash)
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

// ═══════════════════════════════════════════════════════════════════════════════
// BRANDING / WHITE-LABEL
// ═══════════════════════════════════════════════════════════════════════════════

/// Branding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandingConfig {
    pub company_name: String,
    pub primary_color: String,
    pub secondary_color: String,
    pub logo_url: Option<String>,
    pub favicon_url: Option<String>,
    pub login_background_color: String,
    pub custom_css: Option<String>,
}

impl Default for BrandingConfig {
    fn default() -> Self {
        Self {
            company_name: std::env::var("PARKHUB_COMPANY_NAME").unwrap_or_else(|_| "ParkHub".to_string()),
            primary_color: std::env::var("PARKHUB_PRIMARY_COLOR").unwrap_or_else(|_| "#3B82F6".to_string()),
            secondary_color: std::env::var("PARKHUB_SECONDARY_COLOR").unwrap_or_else(|_| "#1D4ED8".to_string()),
            logo_url: None,
            favicon_url: None,
            login_background_color: "#2563EB".to_string(),
            custom_css: None,
        }
    }
}

/// GET /api/v1/branding - Public branding config (no auth)
async fn get_branding_public(
    State(state): State<SharedState>,
) -> Json<ApiResponse<BrandingConfig>> {
    let state = state.read().await;
    let config = load_branding_config(&state.db).await;
    Json(ApiResponse::success(config))
}

/// GET /api/v1/admin/branding - Admin branding config (auth required)
async fn get_branding_admin(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<ApiResponse<BrandingConfig>>) {
    let state = state.read().await;
    let user = match state.db.get_user(&auth_user.user_id.to_string()).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Access denied"))),
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Admin access required")));
    }
    let config = load_branding_config(&state.db).await;
    (StatusCode::OK, Json(ApiResponse::success(config)))
}

/// PUT /api/v1/admin/branding - Update branding config

/// Partial branding update request - all fields optional
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateBrandingRequest {
    pub company_name: Option<String>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub logo_url: Option<Option<String>>,
    pub favicon_url: Option<Option<String>>,
    pub login_background_color: Option<String>,
    pub custom_css: Option<Option<String>>,
}

/// Create parking lot request
#[derive(Debug, Deserialize)]
pub struct CreateLotRequest {
    pub name: String,
    pub address: String,
    pub total_slots: i32,
    #[serde(default)]
    pub status: Option<LotStatus>,
}

/// Admin create user request
#[derive(Debug, Deserialize)]
pub struct AdminCreateUserRequest {
    pub username: String,
    pub password: String,
    pub name: String,
    pub email: String,
    #[serde(default = "default_user_role")]
    pub role: UserRole,
}

fn default_user_role() -> UserRole {
    UserRole::User
}


async fn update_branding(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<UpdateBrandingRequest>,
) -> (StatusCode, Json<ApiResponse<BrandingConfig>>) {
    let state = state.read().await;
    let user = match state.db.get_user(&auth_user.user_id.to_string()).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Access denied"))),
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Admin access required")));
    }

    // Merge with existing branding config
    let mut final_config = load_branding_config(&state.db).await;
    if let Some(v) = req.company_name { final_config.company_name = v; }
    if let Some(v) = req.primary_color { final_config.primary_color = v; }
    if let Some(v) = req.secondary_color { final_config.secondary_color = v; }
    if let Some(v) = req.logo_url { final_config.logo_url = v; }
    if let Some(v) = req.favicon_url { final_config.favicon_url = v; }
    if let Some(v) = req.login_background_color { final_config.login_background_color = v; }
    if let Some(v) = req.custom_css { final_config.custom_css = v; }

    let json_data = serde_json::to_vec(&final_config).unwrap();
    if let Err(e) = state.db.save_branding("config", &json_data).await {
        tracing::error!("Failed to save branding config: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to save branding")));
    }

    AuditEntry::new(AuditEventType::ConfigChanged)
        .user(auth_user.user_id, &user.username)
        .details(serde_json::json!({"action": "branding_updated"}))
        .log();

    (StatusCode::OK, Json(ApiResponse::success(final_config)))
}

/// POST /api/v1/admin/branding/logo - Upload branding logo
async fn upload_branding_logo(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    mut multipart: Multipart,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let state_guard = state.read().await;
    let user = match state_guard.db.get_user(&auth_user.user_id.to_string()).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Access denied"))),
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Admin access required")));
    }

    while let Ok(Some(field)) = multipart.next_field().await {
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();

        // Validate content type
        let ext = match content_type.as_str() {
            "image/png" => "png",
            "image/jpeg" | "image/jpg" => "jpg",
            "image/svg+xml" => "svg",
            _ => {
                return (StatusCode::BAD_REQUEST, Json(ApiResponse::error("INVALID_TYPE", "Only PNG, JPEG, and SVG files are accepted")));
            }
        };

        let data = match field.bytes().await {
            Ok(d) => d,
            Err(e) => {
                tracing::error!("Failed to read upload: {}", e);
                return (StatusCode::BAD_REQUEST, Json(ApiResponse::error("UPLOAD_ERROR", "Failed to read file")));
            }
        };

        // Check size (2MB max)
        if data.len() > 2 * 1024 * 1024 {
            return (StatusCode::BAD_REQUEST, Json(ApiResponse::error("FILE_TOO_LARGE", "Logo must be under 2MB")));
        }

        // Store logo data in branding table with content type
        let logo_entry = serde_json::json!({
            "content_type": content_type,
            "ext": ext,
            "data": base64::engine::general_purpose::STANDARD.encode(&data),
        });
        let logo_bytes = serde_json::to_vec(&logo_entry).unwrap();

        if let Err(e) = state_guard.db.save_branding("logo", &logo_bytes).await {
            tracing::error!("Failed to save logo: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to save logo")));
        }

        // Update branding config with logo_url
        let mut config = load_branding_config(&state_guard.db).await;
        config.logo_url = Some("/api/v1/branding/logo".to_string());
        let config_data = serde_json::to_vec(&config).unwrap();
        let _ = state_guard.db.save_branding("config", &config_data).await;

        AuditEntry::new(AuditEventType::ConfigChanged)
            .user(auth_user.user_id, &user.username)
            .details(serde_json::json!({"action": "logo_uploaded", "type": content_type, "size": data.len()}))
            .log();

        return (StatusCode::OK, Json(ApiResponse::success(serde_json::json!({
            "logo_url": "/api/v1/branding/logo",
            "size": data.len(),
            "content_type": content_type,
        }))));
    }

    (StatusCode::BAD_REQUEST, Json(ApiResponse::error("NO_FILE", "No file uploaded")))
}

/// GET /api/v1/branding/logo - Serve the uploaded logo (no auth)
async fn serve_branding_logo(
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let state = state.read().await;
    match state.db.get_branding("logo").await {
        Ok(Some(data)) => {
            let logo: serde_json::Value = match serde_json::from_slice(&data) {
                Ok(v) => v,
                Err(_) => return (StatusCode::NOT_FOUND, [(header::CONTENT_TYPE, "text/plain".to_string())], vec![]).into_response(),
            };
            let content_type = logo["content_type"].as_str().unwrap_or("image/png").to_string();
            let b64_data = logo["data"].as_str().unwrap_or("");
            match base64::engine::general_purpose::STANDARD.decode(b64_data) {
                Ok(bytes) => {
                    (
                        StatusCode::OK,
                        [
                            (header::CONTENT_TYPE, content_type),
                            (header::CACHE_CONTROL, "public, max-age=3600".to_string()),
                        ],
                        bytes,
                    ).into_response()
                }
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, [(header::CONTENT_TYPE, "text/plain".to_string())], vec![]).into_response(),
            }
        }
        _ => (StatusCode::NOT_FOUND, [(header::CONTENT_TYPE, "text/plain".to_string())], vec![]).into_response(),
    }
}

/// Load branding config from DB, falling back to defaults
async fn load_branding_config(db: &crate::db::Database) -> BrandingConfig {
    match db.get_branding("config").await {
        Ok(Some(data)) => {
            serde_json::from_slice(&data).unwrap_or_default()
        }
        _ => BrandingConfig::default(),
    }
}

/// POST /api/v1/setup/change-password - Change admin password during setup (no auth required)
/// Only works when setup is not complete
async fn setup_change_password(
    State(state): State<SharedState>,
    Json(req): Json<serde_json::Value>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let state = state.read().await;
    
    let current_password = req.get("current_password").and_then(|v| v.as_str()).unwrap_or("");
    let new_password = req.get("new_password").and_then(|v| v.as_str()).unwrap_or("");
    
    if new_password.len() < 8 {
        return (StatusCode::BAD_REQUEST, Json(ApiResponse::error("WEAK_PASSWORD", "Password must be at least 8 characters")));
    }
    
    // Find admin user
    let mut admin = match state.db.get_user_by_username("admin").await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::NOT_FOUND, Json(ApiResponse::error("NOT_FOUND", "Admin user not found"))),
    };
    
    if !verify_password(current_password, &admin.password_hash) {
        return (StatusCode::UNAUTHORIZED, Json(ApiResponse::error("INVALID_PASSWORD", "Current password is incorrect")));
    }
    
    admin.password_hash = hash_password(new_password);
    admin.updated_at = chrono::Utc::now();
    
    if let Err(e) = state.db.save_user(&admin).await {
        tracing::error!("Failed to update password: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to update password")));
    }
    
    (StatusCode::OK, Json(ApiResponse::success(serde_json::json!({ "message": "Password changed successfully" }))))
}


// ═══════════════════════════════════════════════════════════════════════════════
// ADMIN DATABASE RESET
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
struct AdminResetRequest {
    confirm: String,
}

async fn admin_reset_database(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<AdminResetRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    if req.confirm != "RESET" {
        return (StatusCode::BAD_REQUEST, Json(ApiResponse::error("INVALID_CONFIRMATION", "Confirmation must be exactly RESET")));
    }

    let state_guard = state.read().await;
    let user = match state_guard.db.get_user(&auth_user.user_id.to_string()).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Access denied"))),
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Admin access required")));
    }

    if let Err(e) = state_guard.db.reset_all_data().await {
        tracing::error!("Failed to reset database: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to reset database")));
    }

    let _ = state_guard.db.delete_all_users_except(&auth_user.user_id.to_string()).await;

    let mut admin = user;
    admin.password_hash = hash_password("admin");
    admin.updated_at = Utc::now();
    let _ = state_guard.db.save_user(&admin).await;

    tracing::info!("Database reset by admin: {}", admin.username);

    (StatusCode::OK, Json(ApiResponse::success(serde_json::json!({
        "success": true,
        "message": "Database reset complete"
    }))))
}

// ═══════════════════════════════════════════════════════════════════════════════
// SLOT UPDATE (name/status) - PUT /api/v1/lots/:lot_id/slots/:slot_id
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
struct UpdateSlotPropertiesRequest {
    name: Option<String>,
    status: Option<String>,
}

async fn admin_update_slot_properties(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Path((lot_id, slot_id)): Path<(String, String)>,
    Json(req): Json<UpdateSlotPropertiesRequest>,
) -> (StatusCode, Json<ApiResponse<ParkingSlot>>) {
    let state_guard = state.read().await;
    let user = match state_guard.db.get_user(&auth_user.user_id.to_string()).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Access denied"))),
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (StatusCode::FORBIDDEN, Json(ApiResponse::error("FORBIDDEN", "Admin access required")));
    }

    let mut slot = match state_guard.db.get_parking_slot(&slot_id).await {
        Ok(Some(s)) => s,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(ApiResponse::error("NOT_FOUND", "Slot not found"))),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Internal server error")));
        }
    };

    if slot.lot_id.to_string() != lot_id {
        return (StatusCode::BAD_REQUEST, Json(ApiResponse::error("LOT_MISMATCH", "Slot does not belong to this lot")));
    }

    if let Some(ref name) = req.name {
        slot.slot_number = name.clone();
    }
    if let Some(ref status) = req.status {
        slot.status = match status.as_str() {
            "available" => SlotStatus::Available,
            "occupied" => SlotStatus::Occupied,
            "reserved" => SlotStatus::Reserved,
            "disabled" => SlotStatus::Disabled,
            _ => slot.status,
        };
    }

    if let Err(e) = state_guard.db.save_parking_slot(&slot).await {
        tracing::error!("Failed to update slot: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::error("SERVER_ERROR", "Failed to update slot")));
    }

    // Update slot name in lot layout too
    if req.name.is_some() {
        if let Ok(Some(mut lot)) = state_guard.db.get_parking_lot(&lot_id).await {
            if let Some(ref mut layout) = lot.layout {
                for row in &mut layout.rows {
                    for s in &mut row.slots {
                        if s.id == slot_id {
                            s.number = slot.slot_number.clone();
                        }
                    }
                }
                let _ = state_guard.db.save_parking_lot(&lot).await;
            }
        }
    }

    (StatusCode::OK, Json(ApiResponse::success(slot)))
}
