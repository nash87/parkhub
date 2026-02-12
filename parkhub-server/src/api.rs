//! HTTP API Routes
//!
//! RESTful API for the parking system.

use axum::{
    body::Body,
    extract::Multipart,
    extract::{Path, Query, State},
    http::{header, Request, StatusCode},
    middleware::{self, Next},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    routing::{delete, get, patch, post, put},
    Extension, Json, Router,
};
use base64::Engine;
use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
#[cfg(feature = "swagger-ui")]
use utoipa::OpenApi;
#[cfg(feature = "swagger-ui")]
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

use crate::audit::{AuditEntry, AuditEventType};
use crate::metrics;
#[cfg(feature = "swagger-ui")]
use crate::openapi::ApiDoc;
use crate::rate_limit::EndpointRateLimiters;
use crate::static_files;
use axum::http::HeaderMap;

use parkhub_common::{
    AbsenceEntry, AbsencePattern, AbsenceSource, AbsenceType, AdminStats, ApiResponse, AuthTokens,
    Booking, BookingStatus, CreateBookingRequest, HandshakeRequest, HandshakeResponse,
    HomeofficeDay, HomeofficePattern, HomeofficeSettings, LoginRequest, LotStatus, ParkingLot,
    ParkingSlot, PushSubscription, RecurrenceRule, RefreshTokenRequest, RegisterRequest,
    ServerStatus, SlotStatus, TeamAbsenceEntry, TeamVacationEntry, User, UserRole, VacationEntry,
    VacationSource, Vehicle, WaitlistEntry, PROTOCOL_VERSION,
};

use crate::db::Session;
use crate::AppState;

type SharedState = Arc<RwLock<AppState>>;

/// Current version from Cargo.toml
pub const VERSION: &str = match option_env!("PARKHUB_VERSION") {
    Some(v) => v,
    None => env!("CARGO_PKG_VERSION"),
};

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
    pub photo: Option<String>,
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
        .route("/api/v1/auth/forgot-password", post(forgot_password))
        .route("/api/v1/privacy", get(get_privacy_policy))
        .route("/api/v1/about", get(get_about))
        .route("/api/v1/help", get(get_help))
        .route("/api/v1/setup/change-password", post(setup_change_password))
        .route("/api/v1/setup/status", get(get_setup_status))
        .route("/api/v1/branding", get(get_branding_public))
        .route("/api/v1/branding/logo", get(serve_branding_logo))
        .route("/api/v1/vehicles/:id/photo", get(get_vehicle_photo))
        .route("/api/v1/system/maintenance", get(get_maintenance_status))
        .route("/api/v1/admin/updates/stream", get(admin_update_stream))
        .route("/api/v1/settings/privacy", get(get_public_privacy))
        .route("/api/v1/system/version", get(get_system_version));

    // Protected routes (auth required)
    let protected_routes = Router::new()
        .route("/api/v1/users/me", get(get_current_user))
        .route("/api/v1/users/:id", get(get_user))
        .route("/api/v1/lots", get(list_lots).post(create_lot))
        .route("/api/v1/lots/:id", get(get_lot))
        .route("/api/v1/lots/:id/slots", get(get_lot_slots))
        .route(
            "/api/v1/lots/:id/layout",
            get(get_lot_layout).put(update_lot_layout),
        )
        .route("/api/v1/bookings", get(list_bookings).post(create_booking))
        .route(
            "/api/v1/bookings/:id",
            get(get_booking)
                .delete(cancel_booking)
                .patch(update_booking),
        )
        .route("/api/v1/bookings/ical", get(export_ical))
        .route("/api/v1/bookings/:id/checkin", post(checkin_booking))
        .route("/api/v1/vehicles", get(list_vehicles).post(create_vehicle))
        .route("/api/v1/vehicles/:id", delete(delete_vehicle))
        .route("/api/v1/vehicles/:id/photo", post(upload_vehicle_photo))
        .route(
            "/api/v1/homeoffice",
            get(get_homeoffice_settings).put(update_homeoffice_settings),
        )
        .route("/api/v1/homeoffice/pattern", put(update_homeoffice_pattern))
        .route("/api/v1/homeoffice/days", post(add_homeoffice_day))
        .route("/api/v1/homeoffice/days/:id", delete(remove_homeoffice_day))
        .route("/api/v1/vacation", get(list_vacation).post(create_vacation))
        .route("/api/v1/vacation/:id", delete(delete_vacation))
        .route("/api/v1/vacation/import", post(import_vacation_ical))
        .route("/api/v1/vacation/team", get(team_vacation))
        // ── Unified Absence endpoints ──
        .route("/api/v1/absences", get(list_absences).post(create_absence))
        .route("/api/v1/absences/:id", delete(delete_absence))
        .route("/api/v1/absences/import", post(import_absence_ical))
        .route("/api/v1/absences/team", get(team_absences))
        .route(
            "/api/v1/absences/pattern",
            get(get_absence_pattern).post(set_absence_pattern),
        )
        .route(
            "/api/v1/admin/users",
            get(admin_list_users).post(admin_create_user),
        )
        .route("/api/v1/admin/bookings", get(admin_list_bookings))
        .route("/api/v1/admin/stats", get(admin_stats))
        .route("/api/v1/admin/reports", get(admin_reports))
        .route("/api/v1/lots/:lot_id/slots/:slot_id/qr", get(slot_qr_code))
        .route(
            "/api/v1/lots/:id/waitlist",
            get(get_waitlist).post(join_waitlist),
        )
        .route("/api/v1/push/subscribe", post(push_subscribe))
        .route("/api/v1/admin/users/:id", patch(admin_update_user))
        .route("/api/v1/admin/slots/:id", patch(admin_update_slot))
        .route("/api/v1/admin/lots/:id", delete(admin_delete_lot))
        .route("/api/v1/users/me/export", get(export_user_data))
        .route("/api/v1/users/me/delete", delete(delete_own_account))
        .route("/api/v1/users/me/password", patch(change_password))
        .route("/api/v1/setup/complete", post(complete_setup))
        .route(
            "/api/v1/admin/branding",
            get(get_branding_admin).put(update_branding),
        )
        .route(
            "/api/v1/admin/privacy",
            get(get_admin_privacy).put(update_admin_privacy),
        )
        .route("/api/v1/admin/branding/logo", post(upload_branding_logo))
        .route("/api/v1/admin/reset", post(admin_reset_database))
        .route("/api/v1/admin/updates/check", get(admin_check_updates))
        .route("/api/v1/admin/updates/apply", post(admin_apply_update))
        .route(
            "/api/v1/lots/:lot_id/slots/:slot_id",
            put(admin_update_slot_properties),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let metrics_handle_clone = metrics_handle.clone();

    // API catch-all: return 404 JSON for unknown /api/* routes
    let api_fallback = Router::new().route(
        "/api/*rest",
        get(api_not_found)
            .post(api_not_found)
            .put(api_not_found)
            .delete(api_not_found)
            .patch(api_not_found),
    );

    let router = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .route(
            "/metrics",
            get(move || async move {
                (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
                    metrics_handle_clone.render(),
                )
            }),
        )
        .merge(api_fallback)
        .fallback(static_files::static_handler)
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    #[cfg(feature = "swagger-ui")]
    let router =
        router.merge(SwaggerUi::new("/swagger-ui").url("/api/openapi.json", ApiDoc::openapi()));

    router
}

// ═══════════════════════════════════════════════════════════════════════════════
// API CATCH-ALL (returns JSON 404 for /api/* routes)
// ═══════════════════════════════════════════════════════════════════════════════

async fn api_not_found() -> (StatusCode, Json<ApiResponse<()>>) {
    (
        StatusCode::NOT_FOUND,
        Json(ApiResponse::error("NOT_FOUND", "API endpoint not found")),
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
                Json(ApiResponse::error(
                    "UNAUTHORIZED",
                    "Missing or invalid authorization header",
                )),
            ));
        }
    };

    let state_guard = state.read().await;
    let session = match state_guard.db.get_session(token).await {
        Ok(Some(s)) if !s.is_expired() => s,
        _ => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse::error(
                    "UNAUTHORIZED",
                    "Invalid or expired token",
                )),
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

async fn forgot_password(Json(body): Json<serde_json::Value>) -> Json<ApiResponse<()>> {
    // Stub: SMTP not configured, always return success to not leak user existence
    tracing::info!("Forgot password request for: {:?}", body.get("email"));
    Json(ApiResponse::success(()))
}

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
            Json(
                serde_json::json!({"ready": false, "reason": format!("Database unavailable: {}", e)}),
            ),
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
            format!(
                "Protocol version mismatch: server={}, client={}",
                PROTOCOL_VERSION, request.protocol_version
            ),
        ));
    }
    Json(ApiResponse::success(HandshakeResponse {
        server_name: state.config.server_name.clone(),
        server_version: VERSION.to_string(),
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
        Ok(None) => match state_guard.db.get_user_by_email(&request.username).await {
            Ok(Some(u)) => u,
            _ => {
                AuditEntry::builder(AuditEventType::LoginFailed)
                    .details(serde_json::json!({"username": &request.username}))
                    .error("Invalid credentials")
                    .log();
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(ApiResponse::error(
                        "INVALID_CREDENTIALS",
                        "Invalid username or password",
                    )),
                );
            }
        },
        Err(e) => {
            tracing::error!("Database error during login: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Internal server error")),
            );
        }
    };

    if !verify_password(&request.password, &user.password_hash) {
        AuditEntry::builder(AuditEventType::LoginFailed)
            .user(user.id, &user.username)
            .error("Invalid password")
            .log();
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::error(
                "INVALID_CREDENTIALS",
                "Invalid username or password",
            )),
        );
    }

    if !user.is_active {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error(
                "ACCOUNT_DISABLED",
                "This account has been disabled",
            )),
        );
    }

    let session = Session::new(user.id, 24);
    let access_token = Uuid::new_v4().to_string();

    if let Err(e) = state_guard.db.save_session(&access_token, &session).await {
        tracing::error!("Failed to save session: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(
                "SERVER_ERROR",
                "Failed to create session",
            )),
        );
    }

    AuditEntry::builder(AuditEventType::LoginSuccess)
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
        return (
            StatusCode::CONFLICT,
            Json(ApiResponse::error(
                "EMAIL_EXISTS",
                "An account with this email already exists",
            )),
        );
    }

    // Validate username format
    let username_trimmed = request.username.trim().to_string();
    if username_trimmed.len() < 3 || username_trimmed.len() > 30 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(
                "INVALID_USERNAME",
                "Username must be 3-30 characters",
            )),
        );
    }

    if let Ok(Some(_)) = state_guard.db.get_user_by_username(&request.username).await {
        return (
            StatusCode::CONFLICT,
            Json(ApiResponse::error(
                "USERNAME_EXISTS",
                "This username is already taken",
            )),
        );
    }

    // Validate password strength
    if crate::validation::validate_password_strength(&request.password).is_err() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(
                "WEAK_PASSWORD",
                "Password must be at least 8 characters with uppercase, lowercase, and digit",
            )),
        );
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
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(
                "SERVER_ERROR",
                "Failed to create account",
            )),
        );
    }

    let session = Session::new(user.id, 24);
    let access_token = Uuid::new_v4().to_string();

    if let Err(e) = state_guard.db.save_session(&access_token, &session).await {
        tracing::error!("Failed to save session: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(
                "SERVER_ERROR",
                "Failed to create session",
            )),
        );
    }

    AuditEntry::builder(AuditEventType::UserCreated)
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
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ApiResponse::error(
            "NOT_IMPLEMENTED",
            "Token refresh not yet fully implemented",
        )),
    )
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
        Ok(Some(user)) => (
            StatusCode::OK,
            Json(ApiResponse::success(UserResponse::from(user))),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("NOT_FOUND", "User not found")),
        ),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Internal server error")),
            )
        }
    }
}

async fn get_user(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<ApiResponse<UserResponse>>) {
    let state = state.read().await;
    match state.db.get_user(&id).await {
        Ok(Some(user)) => (
            StatusCode::OK,
            Json(ApiResponse::success(UserResponse::from(user))),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("NOT_FOUND", "User not found")),
        ),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Internal server error")),
            )
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
            Json(ApiResponse::error(
                "SERVER_ERROR",
                "Failed to list parking lots",
            ))
        }
    }
}

/// Sync ParkingSlot records from a LotLayout
async fn sync_slots_from_layout(
    db: &crate::db::Database,
    lot_id: &uuid::Uuid,
    layout: &parkhub_common::LotLayout,
) {
    for row in &layout.rows {
        for slot_config in &row.slots {
            let slot_id =
                uuid::Uuid::parse_str(&slot_config.id).unwrap_or_else(|_| uuid::Uuid::new_v4());
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
    let user = match state_guard
        .db
        .get_user(&auth_user.user_id.to_string())
        .await
    {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("FORBIDDEN", "Access denied")),
            )
        }
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("FORBIDDEN", "Admin access required")),
        );
    }
    // Sanitize inputs (strip HTML tags) and validate length
    let name = req.name.replace("<", "&lt;").replace(">", "&gt;");
    let address = req.address.replace("<", "&lt;").replace(">", "&gt;");
    if name.len() > 200 || address.len() > 500 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(
                "INVALID_INPUT",
                "Name or address too long",
            )),
        );
    }

    // Validate total_slots
    if req.total_slots < 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(
                "INVALID_INPUT",
                "total_slots must be non-negative",
            )),
        );
    }
    if req.total_slots > 10000 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(
                "INVALID_INPUT",
                "total_slots exceeds maximum (10000)",
            )),
        );
    }

    let now = Utc::now();
    let lot = ParkingLot {
        id: Uuid::new_v4(),
        name,
        address,
        total_slots: req.total_slots,
        available_slots: req.total_slots,
        layout: None,
        status: req.status.unwrap_or(LotStatus::Open),
        created_at: now,
        updated_at: now,
    };
    if let Err(e) = state_guard.db.save_parking_lot(&lot).await {
        tracing::error!("Failed to save parking lot: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(
                "SERVER_ERROR",
                "Failed to create parking lot",
            )),
        );
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
    AuditEntry::builder(AuditEventType::LotCreated)
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
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("NOT_FOUND", "Parking lot not found")),
        ),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Internal server error")),
            )
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
) -> (
    StatusCode,
    Json<ApiResponse<Option<parkhub_common::LotLayout>>>,
) {
    let state = state.read().await;
    match state.db.get_parking_lot(&id).await {
        Ok(Some(lot)) => {
            if lot.layout.is_some() {
                return (StatusCode::OK, Json(ApiResponse::success(lot.layout)));
            }
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("NOT_FOUND", "Parking lot not found")),
            )
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Internal server error")),
            );
        }
    }
    match state.db.get_lot_layout(&id).await {
        Ok(layout) => (StatusCode::OK, Json(ApiResponse::success(layout))),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Internal server error")),
            )
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
    let user = match state_guard
        .db
        .get_user(&auth_user.user_id.to_string())
        .await
    {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("FORBIDDEN", "Access denied")),
            )
        }
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("FORBIDDEN", "Admin access required")),
        );
    }
    match state_guard.db.get_parking_lot(&id).await {
        Ok(Some(mut lot)) => {
            lot.layout = Some(layout.clone());
            lot.updated_at = Utc::now();
            if let Err(e) = state_guard.db.save_parking_lot(&lot).await {
                tracing::error!("Failed to save parking lot: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(
                        "SERVER_ERROR",
                        "Failed to update layout",
                    )),
                );
            }
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("NOT_FOUND", "Parking lot not found")),
            )
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Internal server error")),
            );
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
    match state
        .db
        .list_bookings_by_user(&auth_user.user_id.to_string())
        .await
    {
        Ok(bookings) => Json(ApiResponse::success(bookings)),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            Json(ApiResponse::error(
                "SERVER_ERROR",
                "Failed to list bookings",
            ))
        }
    }
}

async fn create_booking(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateBookingRequest>,
) -> (StatusCode, Json<ApiResponse<Booking>>) {
    let state_guard = state.read().await;

    let slot = match state_guard
        .db
        .get_parking_slot(&req.slot_id.to_string())
        .await
    {
        Ok(Some(s)) => s,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("NOT_FOUND", "Slot not found")),
            )
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Internal server error")),
            );
        }
    };

    if slot.status != SlotStatus::Available {
        return (
            StatusCode::CONFLICT,
            Json(ApiResponse::error(
                "SLOT_UNAVAILABLE",
                "This slot is not available",
            )),
        );
    }

    if let Some(ref dept) = slot.reserved_for_department {
        let user = match state_guard
            .db
            .get_user(&auth_user.user_id.to_string())
            .await
        {
            Ok(Some(u)) => u,
            _ => {
                return (
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::error("FORBIDDEN", "Access denied")),
                )
            }
        };
        let user_dept = user.department.as_deref().unwrap_or("");
        if user_dept != dept && user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error(
                    "DEPARTMENT_RESTRICTED",
                    format!("This slot is reserved for department: {}", dept),
                )),
            );
        }
    }

    let lot_name = match state_guard
        .db
        .get_parking_lot(&req.lot_id.to_string())
        .await
    {
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

    // Validate booking times
    if end_time <= req.start_time {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(
                "INVALID_TIME",
                "End time must be after start time",
            )),
        );
    }
    let now_check = Utc::now();
    if req.start_time < now_check - chrono::Duration::hours(24) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(
                "PAST_BOOKING",
                "Cannot create bookings in the past",
            )),
        );
    }

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
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(
                "SERVER_ERROR",
                "Failed to create booking",
            )),
        );
    }

    let mut updated_slot = slot;
    updated_slot.status = SlotStatus::Reserved;
    let _ = state_guard.db.save_parking_slot(&updated_slot).await;

    // Audit log
    AuditEntry::builder(AuditEventType::BookingCreated)
        .user(auth_user.user_id, "")
        .resource("booking", &booking.id.to_string())
        .log();
    metrics::record_booking_event("created");

    // Email confirmation
    if let Some(ref email_svc) = state_guard.email {
        if let Ok(Some(user)) = state_guard
            .db
            .get_user(&auth_user.user_id.to_string())
            .await
        {
            let _ = email_svc
                .send_booking_confirmation(
                    &user.email,
                    &user.name,
                    booking.slot_number.as_deref().unwrap_or("?"),
                    booking.lot_name.as_deref().unwrap_or("Lot"),
                    &booking.start_time.to_rfc3339(),
                    &booking.end_time.to_rfc3339(),
                )
                .await;
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
                    id: Uuid::new_v4(),
                    user_id: auth_user.user_id,
                    lot_id: booking.lot_id,
                    slot_id: booking.slot_id,
                    booking_type: booking.booking_type.clone(),
                    dauer_interval: booking.dauer_interval.clone(),
                    lot_name: booking.lot_name.clone(),
                    slot_number: booking.slot_number.clone(),
                    vehicle_plate: booking.vehicle_plate.clone(),
                    start_time: cs,
                    end_time: ce,
                    status: BookingStatus::Confirmed,
                    created_at: now,
                    updated_at: now,
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
                return (
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::error("FORBIDDEN", "Access denied")),
                );
            }
            (StatusCode::OK, Json(ApiResponse::success(booking)))
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("NOT_FOUND", "Booking not found")),
        ),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Internal server error")),
            )
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
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("NOT_FOUND", "Booking not found")),
            )
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Internal server error")),
            );
        }
    };

    if booking.user_id != auth_user.user_id {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("FORBIDDEN", "Access denied")),
        );
    }

    if booking.status == BookingStatus::Cancelled || booking.status == BookingStatus::Completed {
        return (
            StatusCode::CONFLICT,
            Json(ApiResponse::error(
                "BOOKING_NOT_MODIFIABLE",
                "Cannot modify a cancelled or completed booking",
            )),
        );
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
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(
                "SERVER_ERROR",
                "Failed to update booking",
            )),
        );
    }

    AuditEntry::builder(AuditEventType::BookingUpdated)
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
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("NOT_FOUND", "Booking not found")),
            )
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Internal server error")),
            );
        }
    };

    if booking.user_id != auth_user.user_id {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("FORBIDDEN", "Access denied")),
        );
    }

    let mut updated_booking = booking.clone();
    updated_booking.status = BookingStatus::Cancelled;
    updated_booking.updated_at = Utc::now();

    if let Err(e) = state_guard.db.save_booking(&updated_booking).await {
        tracing::error!("Failed to update booking: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(
                "SERVER_ERROR",
                "Failed to cancel booking",
            )),
        );
    }

    if let Ok(Some(mut slot)) = state_guard
        .db
        .get_parking_slot(&booking.slot_id.to_string())
        .await
    {
        slot.status = SlotStatus::Available;
        let _ = state_guard.db.save_parking_slot(&slot).await;
    }

    AuditEntry::builder(AuditEventType::BookingCancelled)
        .user(auth_user.user_id, "")
        .resource("booking", &booking.id.to_string())
        .log();
    metrics::record_booking_event("cancelled");

    // Send cancellation email
    if let Some(ref email_svc) = state_guard.email {
        if let Ok(Some(user)) = state_guard
            .db
            .get_user(&auth_user.user_id.to_string())
            .await
        {
            let _ = email_svc
                .send_auto_release_notification(
                    &user.email,
                    &user.name,
                    booking.slot_number.as_deref().unwrap_or("?"),
                    booking.lot_name.as_deref().unwrap_or("Lot"),
                )
                .await;
        }
    }

    // Notify waitlist
    let date = booking.start_time.format("%Y-%m-%d").to_string();
    if let Ok(waitlist) = state_guard
        .db
        .list_waitlist_by_lot(&booking.lot_id.to_string(), Some(&date))
        .await
    {
        if let Some(first) = waitlist.first() {
            if !first.notified {
                if let Some(ref email_svc) = state_guard.email {
                    if let Ok(Some(wu)) = state_guard.db.get_user(&first.user_id.to_string()).await
                    {
                        let _ = email_svc
                            .send_waitlist_notification(
                                &wu.email,
                                &wu.name,
                                booking.lot_name.as_deref().unwrap_or("Lot"),
                                &date,
                            )
                            .await;
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
    match state
        .db
        .list_vehicles_by_user(&auth_user.user_id.to_string())
        .await
    {
        Ok(vehicles) => Json(ApiResponse::success(vehicles)),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            Json(ApiResponse::error(
                "SERVER_ERROR",
                "Failed to list vehicles",
            ))
        }
    }
}

async fn create_vehicle(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateVehicleRequest>,
) -> (StatusCode, Json<ApiResponse<Vehicle>>) {
    let vehicle_id = Uuid::new_v4();
    let mut photo_url = None;

    // Handle base64 photo if provided
    if let Some(ref photo_b64) = req.photo {
        let state_guard = state.read().await;
        match process_and_save_photo(&state_guard.data_dir, &vehicle_id.to_string(), photo_b64) {
            Ok(_) => {
                photo_url = Some(format!("/api/v1/vehicles/{}/photo", vehicle_id));
            }
            Err(e) => {
                tracing::warn!("Failed to process vehicle photo: {}", e);
            }
        }
    }

    let vehicle = Vehicle {
        id: vehicle_id,
        user_id: auth_user.user_id,
        plate: req.license_plate,
        make: req.make,
        model: req.model,
        color: req.color,
        is_default: false,
        photo_url,
        created_at: Utc::now(),
    };

    let state_guard = state.read().await;
    if let Err(e) = state_guard.db.save_vehicle(&vehicle).await {
        tracing::error!("Failed to save vehicle: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(
                "SERVER_ERROR",
                "Failed to create vehicle",
            )),
        );
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
        Ok(Some(_)) => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("FORBIDDEN", "Access denied")),
            )
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("NOT_FOUND", "Vehicle not found")),
            )
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Internal server error")),
            );
        }
    }
    match state_guard.db.delete_vehicle(&id).await {
        Ok(true) => (StatusCode::OK, Json(ApiResponse::success(()))),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("NOT_FOUND", "Vehicle not found")),
        ),
        Err(e) => {
            tracing::error!("Failed to delete vehicle: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(
                    "SERVER_ERROR",
                    "Failed to delete vehicle",
                )),
            )
        }
    }
}

fn process_and_save_photo(
    data_dir: &std::path::Path,
    vehicle_id: &str,
    photo_b64: &str,
) -> anyhow::Result<()> {
    use image::GenericImageView;
    // Strip data URL prefix if present
    let b64_data = if let Some(idx) = photo_b64.find(",") {
        &photo_b64[idx + 1..]
    } else {
        photo_b64
    };
    let bytes = base64::engine::general_purpose::STANDARD.decode(b64_data)?;
    let img = image::load_from_memory(&bytes)?;
    let (w, h) = img.dimensions();
    let img = if w > 800 || h > 800 {
        let ratio = 800.0 / w.max(h) as f64;
        let nw = (w as f64 * ratio) as u32;
        let nh = (h as f64 * ratio) as u32;
        img.resize(nw, nh, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };
    let vehicles_dir = data_dir.join("vehicles");
    std::fs::create_dir_all(&vehicles_dir)?;
    let path = vehicles_dir.join(format!("{}.jpg", vehicle_id));
    let mut output = std::io::BufWriter::new(std::fs::File::create(&path)?);
    img.write_to(&mut output, image::ImageFormat::Jpeg)?;
    Ok(())
}

async fn upload_vehicle_photo(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<String>,
    mut multipart: Multipart,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let state_guard = state.read().await;
    // Verify ownership
    match state_guard.db.get_vehicle(&id).await {
        Ok(Some(v)) if v.user_id == auth_user.user_id => {}
        Ok(Some(_)) => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("FORBIDDEN", "Access denied")),
            )
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("NOT_FOUND", "Vehicle not found")),
            )
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Database error")),
            )
        }
    }

    if let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("photo") {
            if let Ok(data) = field.bytes().await {
                let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
                match process_and_save_photo(&state_guard.data_dir, &id, &b64) {
                    Ok(_) => {
                        let photo_url = format!("/api/v1/vehicles/{}/photo", id);
                        // Update vehicle in DB
                        if let Ok(Some(mut vehicle)) = state_guard.db.get_vehicle(&id).await {
                            vehicle.photo_url = Some(photo_url.clone());
                            let _ = state_guard.db.save_vehicle(&vehicle).await;
                        }
                        return (
                            StatusCode::OK,
                            Json(ApiResponse::success(
                                serde_json::json!({"photo_url": photo_url}),
                            )),
                        );
                    }
                    Err(e) => {
                        tracing::error!("Failed to process photo: {}", e);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ApiResponse::error(
                                "SERVER_ERROR",
                                "Failed to process photo",
                            )),
                        );
                    }
                }
            }
        }
    }
    (
        StatusCode::BAD_REQUEST,
        Json(ApiResponse::error("BAD_REQUEST", "No photo field found")),
    )
}

async fn get_vehicle_photo(State(state): State<SharedState>, Path(id): Path<String>) -> Response {
    let state_guard = state.read().await;
    let path = state_guard
        .data_dir
        .join("vehicles")
        .join(format!("{}.jpg", id));
    if path.exists() {
        match tokio::fs::read(&path).await {
            Ok(bytes) => Response::builder()
                .status(200)
                .header(header::CONTENT_TYPE, "image/jpeg")
                .header(header::CACHE_CONTROL, "public, max-age=3600")
                .body(Body::from(bytes))
                .unwrap(),
            Err(_) => Response::builder().status(500).body(Body::empty()).unwrap(),
        }
    } else {
        Response::builder().status(404).body(Body::empty()).unwrap()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HOMEOFFICE
// ═══════════════════════════════════════════════════════════════════════════════

async fn get_homeoffice_settings(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<ApiResponse<HomeofficeSettings>>) {
    let state = state.read().await;
    match state
        .db
        .get_homeoffice_settings(&auth_user.user_id.to_string())
        .await
    {
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
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Internal server error")),
            )
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
    vals.iter()
        .filter_map(|v| match v {
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
        })
        .collect()
}

async fn update_homeoffice_settings(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<UpdateHomeofficeRequest>,
) -> (StatusCode, Json<ApiResponse<HomeofficeSettings>>) {
    let pattern = if let Some(p) = req.pattern {
        HomeofficePattern {
            weekdays: parse_weekdays(&p.weekdays),
        }
    } else if let Some(weekdays) = req.weekdays {
        HomeofficePattern {
            weekdays: parse_weekdays(&weekdays),
        }
    } else {
        HomeofficePattern { weekdays: vec![] }
    };
    let state_guard = state.read().await;
    let mut settings = match state_guard
        .db
        .get_homeoffice_settings(&auth_user.user_id.to_string())
        .await
    {
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
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(
                "SERVER_ERROR",
                "Failed to save settings",
            )),
        );
    }
    (StatusCode::OK, Json(ApiResponse::success(settings)))
}

async fn update_homeoffice_pattern(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(pattern): Json<HomeofficePattern>,
) -> (StatusCode, Json<ApiResponse<HomeofficeSettings>>) {
    let state_guard = state.read().await;
    let mut settings = match state_guard
        .db
        .get_homeoffice_settings(&auth_user.user_id.to_string())
        .await
    {
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
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(
                "SERVER_ERROR",
                "Failed to save settings",
            )),
        );
    }
    (StatusCode::OK, Json(ApiResponse::success(settings)))
}

async fn add_homeoffice_day(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(day): Json<HomeofficeDay>,
) -> (StatusCode, Json<ApiResponse<HomeofficeSettings>>) {
    let state_guard = state.read().await;
    let mut settings = match state_guard
        .db
        .get_homeoffice_settings(&auth_user.user_id.to_string())
        .await
    {
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
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(
                "SERVER_ERROR",
                "Failed to save settings",
            )),
        );
    }
    (StatusCode::CREATED, Json(ApiResponse::success(settings)))
}

async fn remove_homeoffice_day(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(day_id): Path<String>,
) -> (StatusCode, Json<ApiResponse<HomeofficeSettings>>) {
    let state_guard = state.read().await;
    let mut settings = match state_guard
        .db
        .get_homeoffice_settings(&auth_user.user_id.to_string())
        .await
    {
        Ok(Some(s)) => s,
        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error(
                    "NOT_FOUND",
                    "No homeoffice settings found",
                )),
            )
        }
    };
    settings.single_days.retain(|d| d.id != day_id);
    if let Err(e) = state_guard.db.save_homeoffice_settings(&settings).await {
        tracing::error!("Failed to save homeoffice settings: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(
                "SERVER_ERROR",
                "Failed to save settings",
            )),
        );
    }
    (StatusCode::OK, Json(ApiResponse::success(settings)))
}

// ═══════════════════════════════════════════════════════════════════════════════
// VACATION
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct CreateVacationRequest {
    pub start_date: String,
    pub end_date: String,
    pub note: Option<String>,
}

async fn list_vacation(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<ApiResponse<Vec<VacationEntry>>>) {
    let state = state.read().await;
    match state
        .db
        .list_vacation_entries_by_user(&auth_user.user_id.to_string())
        .await
    {
        Ok(entries) => (StatusCode::OK, Json(ApiResponse::success(entries))),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Internal server error")),
            )
        }
    }
}

async fn create_vacation(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateVacationRequest>,
) -> (StatusCode, Json<ApiResponse<VacationEntry>>) {
    if req.start_date.is_empty() || req.end_date.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(
                "VALIDATION",
                "start_date and end_date are required",
            )),
        );
    }
    if req.end_date < req.start_date {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(
                "VALIDATION",
                "end_date must be >= start_date",
            )),
        );
    }
    let entry = VacationEntry {
        id: Uuid::new_v4(),
        user_id: auth_user.user_id,
        start_date: req.start_date,
        end_date: req.end_date,
        note: req.note,
        source: VacationSource::Manual,
    };
    let state = state.read().await;
    if let Err(e) = state.db.save_vacation_entry(&entry).await {
        tracing::error!("Failed to save vacation entry: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(
                "SERVER_ERROR",
                "Failed to save vacation",
            )),
        );
    }
    (StatusCode::CREATED, Json(ApiResponse::success(entry)))
}

async fn delete_vacation(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let state = state.read().await;
    // Verify ownership
    match state.db.get_vacation_entry(&id).await {
        Ok(Some(entry)) if entry.user_id == auth_user.user_id => {}
        Ok(Some(_)) => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("FORBIDDEN", "Not your vacation entry")),
            )
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("NOT_FOUND", "Vacation entry not found")),
            )
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Internal server error")),
            );
        }
    }
    match state.db.delete_vacation_entry(&id).await {
        Ok(true) => (StatusCode::OK, Json(ApiResponse::success(()))),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("NOT_FOUND", "Vacation entry not found")),
        ),
        Err(e) => {
            tracing::error!("Failed to delete vacation: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(
                    "SERVER_ERROR",
                    "Failed to delete vacation",
                )),
            )
        }
    }
}

/// Parse iCal data and extract vacation events
fn parse_ical_vacation_events(ical_data: &str) -> Vec<(String, String, Option<String>)> {
    let mut events = Vec::new();
    let keywords = [
        "urlaub",
        "vacation",
        "holiday",
        "ooo",
        "out of office",
        "abwesend",
    ];

    let mut i = 0;
    let lines: Vec<&str> = ical_data.lines().collect();
    while i < lines.len() {
        if lines[i].trim() == "BEGIN:VEVENT" {
            let mut summary = String::new();
            let mut dtstart = String::new();
            let mut dtend = String::new();
            let mut j = i + 1;
            while j < lines.len() && lines[j].trim() != "END:VEVENT" {
                let line = lines[j].trim();
                if let Some(stripped) = line.strip_prefix("SUMMARY:") {
                    summary = stripped.to_string();
                } else if line.starts_with("DTSTART") {
                    // Handle DTSTART;VALUE=DATE:20260101 or DTSTART:20260101T...
                    if let Some(val) = line.split(':').next_back() {
                        let val = val.trim();
                        if val.len() >= 8 {
                            dtstart = format!("{}-{}-{}", &val[0..4], &val[4..6], &val[6..8]);
                        }
                    }
                } else if line.starts_with("DTEND") {
                    if let Some(val) = line.split(':').next_back() {
                        let val = val.trim();
                        if val.len() >= 8 {
                            dtend = format!("{}-{}-{}", &val[0..4], &val[4..6], &val[6..8]);
                        }
                    }
                }
                j += 1;
            }
            let summary_lower = summary.to_lowercase();
            if keywords.iter().any(|kw| summary_lower.contains(kw)) && !dtstart.is_empty() {
                if dtend.is_empty() {
                    dtend = dtstart.clone();
                }
                // For all-day events, DTEND is exclusive, so subtract one day
                // But we keep it simple: if dtend > dtstart, use dtend - 1 day
                // Actually, let's keep end_date as-is for simplicity (inclusive interpretation)
                events.push((
                    dtstart,
                    dtend,
                    if summary.is_empty() {
                        None
                    } else {
                        Some(summary)
                    },
                ));
            }
            i = j;
        }
        i += 1;
    }
    events
}

async fn import_vacation_ical(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    mut multipart: Multipart,
) -> (StatusCode, Json<ApiResponse<Vec<VacationEntry>>>) {
    let mut ical_data = String::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        if let Ok(text) = field.text().await {
            ical_data = text;
            break;
        }
    }
    if ical_data.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("VALIDATION", "No iCal data provided")),
        );
    }

    let events = parse_ical_vacation_events(&ical_data);
    let mut entries = Vec::new();
    let state = state.read().await;

    for (start, end, note) in events {
        let entry = VacationEntry {
            id: Uuid::new_v4(),
            user_id: auth_user.user_id,
            start_date: start,
            end_date: end,
            note,
            source: VacationSource::Import,
        };
        if let Err(e) = state.db.save_vacation_entry(&entry).await {
            tracing::error!("Failed to save imported vacation: {}", e);
            continue;
        }
        entries.push(entry);
    }

    (StatusCode::CREATED, Json(ApiResponse::success(entries)))
}

async fn team_vacation(
    State(state): State<SharedState>,
    Extension(_auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<ApiResponse<Vec<TeamVacationEntry>>>) {
    let state = state.read().await;
    let all_entries = match state.db.list_all_vacation_entries().await {
        Ok(e) => e,
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Internal server error")),
            );
        }
    };
    let users = match state.db.list_users().await {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Internal server error")),
            );
        }
    };
    let user_map: std::collections::HashMap<String, String> = users
        .into_iter()
        .map(|u| (u.id.to_string(), u.name))
        .collect();

    let team: Vec<TeamVacationEntry> = all_entries
        .into_iter()
        .filter_map(|e| {
            let name = user_map.get(&e.user_id.to_string())?.clone();
            Some(TeamVacationEntry {
                user_name: name,
                start_date: e.start_date,
                end_date: e.end_date,
            })
        })
        .collect();

    (StatusCode::OK, Json(ApiResponse::success(team)))
}

// ═══════════════════════════════════════════════════════════════════════════════
// ABSENCES (unified)
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct AbsenceQueryParams {
    #[serde(rename = "type")]
    pub absence_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAbsenceRequest {
    pub absence_type: String,
    pub start_date: String,
    pub end_date: String,
    pub note: Option<String>,
}

fn parse_absence_type(s: &str) -> Option<AbsenceType> {
    match s.to_lowercase().as_str() {
        "homeoffice" | "home_office" => Some(AbsenceType::Homeoffice),
        "vacation" | "urlaub" => Some(AbsenceType::Vacation),
        "sick" | "krank" => Some(AbsenceType::Sick),
        "business_trip" | "dienstreise" => Some(AbsenceType::BusinessTrip),
        "other" | "sonstiges" => Some(AbsenceType::Other),
        _ => None,
    }
}

async fn list_absences(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(params): Query<AbsenceQueryParams>,
) -> (StatusCode, Json<ApiResponse<Vec<AbsenceEntry>>>) {
    let state = state.read().await;
    match state
        .db
        .list_absence_entries_by_user(&auth_user.user_id.to_string())
        .await
    {
        Ok(entries) => {
            let filtered = if let Some(ref t) = params.absence_type {
                if let Some(at) = parse_absence_type(t) {
                    entries
                        .into_iter()
                        .filter(|e| e.absence_type == at)
                        .collect()
                } else {
                    entries
                }
            } else {
                entries
            };
            (StatusCode::OK, Json(ApiResponse::success(filtered)))
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Internal server error")),
            )
        }
    }
}

async fn create_absence(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateAbsenceRequest>,
) -> (StatusCode, Json<ApiResponse<AbsenceEntry>>) {
    let absence_type =
        match parse_absence_type(&req.absence_type) {
            Some(t) => t,
            None => return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error(
                    "VALIDATION",
                    "Invalid absence_type. Use: homeoffice, vacation, sick, business_trip, other",
                )),
            ),
        };
    if req.start_date.is_empty() || req.end_date.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(
                "VALIDATION",
                "start_date and end_date are required",
            )),
        );
    }
    if req.end_date < req.start_date {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(
                "VALIDATION",
                "end_date must be >= start_date",
            )),
        );
    }
    let entry = AbsenceEntry {
        id: Uuid::new_v4(),
        user_id: auth_user.user_id,
        absence_type,
        start_date: req.start_date,
        end_date: req.end_date,
        note: req.note,
        source: AbsenceSource::Manual,
        created_at: Utc::now(),
    };
    let state = state.read().await;
    if let Err(e) = state.db.save_absence_entry(&entry).await {
        tracing::error!("Failed to save absence entry: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error("SERVER_ERROR", "Failed to save absence")),
        );
    }
    (StatusCode::CREATED, Json(ApiResponse::success(entry)))
}

async fn delete_absence(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    let state = state.read().await;
    match state.db.get_absence_entry(&id).await {
        Ok(Some(entry)) if entry.user_id == auth_user.user_id => {}
        Ok(Some(_)) => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("FORBIDDEN", "Not your absence entry")),
            )
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("NOT_FOUND", "Absence entry not found")),
            )
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Internal server error")),
            );
        }
    }
    match state.db.delete_absence_entry(&id).await {
        Ok(true) => (StatusCode::OK, Json(ApiResponse::success(()))),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("NOT_FOUND", "Absence entry not found")),
        ),
        Err(e) => {
            tracing::error!("Failed to delete absence: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(
                    "SERVER_ERROR",
                    "Failed to delete absence",
                )),
            )
        }
    }
}

/// Parse iCal data and extract absence events with type detection
fn parse_ical_absence_events(
    ical_data: &str,
) -> Vec<(AbsenceType, String, String, Option<String>)> {
    let mut events = Vec::new();

    let vacation_kw = [
        "urlaub",
        "vacation",
        "holiday",
        "ooo",
        "out of office",
        "abwesend",
        "frei",
        "off",
    ];
    let sick_kw = ["krank", "sick", "illness", "arzt", "doctor"];
    let trip_kw = [
        "dienstreise",
        "business trip",
        "travel",
        "reise",
        "konferenz",
        "conference",
        "messe",
    ];
    let ho_kw = [
        "homeoffice",
        "home office",
        "remote",
        "wfh",
        "work from home",
    ];

    let lines: Vec<&str> = ical_data.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim() == "BEGIN:VEVENT" {
            let mut summary = String::new();
            let mut dtstart = String::new();
            let mut dtend = String::new();
            let mut j = i + 1;
            while j < lines.len() && lines[j].trim() != "END:VEVENT" {
                let line = lines[j].trim();
                if let Some(stripped) = line.strip_prefix("SUMMARY:") {
                    summary = stripped.to_string();
                } else if line.starts_with("DTSTART") {
                    if let Some(val) = line.split(':').next_back() {
                        let val = val.trim();
                        if val.len() >= 8 {
                            dtstart = format!("{}-{}-{}", &val[0..4], &val[4..6], &val[6..8]);
                        }
                    }
                } else if line.starts_with("DTEND") {
                    if let Some(val) = line.split(':').next_back() {
                        let val = val.trim();
                        if val.len() >= 8 {
                            dtend = format!("{}-{}-{}", &val[0..4], &val[4..6], &val[6..8]);
                        }
                    }
                }
                j += 1;
            }

            if !dtstart.is_empty() {
                if dtend.is_empty() {
                    dtend = dtstart.clone();
                }
                let summary_lower = summary.to_lowercase();

                let absence_type = if ho_kw.iter().any(|kw| summary_lower.contains(kw)) {
                    Some(AbsenceType::Homeoffice)
                } else if sick_kw.iter().any(|kw| summary_lower.contains(kw)) {
                    Some(AbsenceType::Sick)
                } else if trip_kw.iter().any(|kw| summary_lower.contains(kw)) {
                    Some(AbsenceType::BusinessTrip)
                } else if vacation_kw.iter().any(|kw| summary_lower.contains(kw)) {
                    Some(AbsenceType::Vacation)
                } else {
                    None
                };

                if let Some(at) = absence_type {
                    events.push((
                        at,
                        dtstart,
                        dtend,
                        if summary.is_empty() {
                            None
                        } else {
                            Some(summary)
                        },
                    ));
                }
            }
            i = j;
        }
        i += 1;
    }
    events
}

async fn import_absence_ical(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    mut multipart: Multipart,
) -> (StatusCode, Json<ApiResponse<Vec<AbsenceEntry>>>) {
    let mut ical_data = String::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        if let Ok(text) = field.text().await {
            ical_data = text;
            break;
        }
    }
    if ical_data.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error("VALIDATION", "No iCal data provided")),
        );
    }

    let events = parse_ical_absence_events(&ical_data);
    let mut entries = Vec::new();
    let state = state.read().await;

    for (absence_type, start, end, note) in events {
        let entry = AbsenceEntry {
            id: Uuid::new_v4(),
            user_id: auth_user.user_id,
            absence_type,
            start_date: start,
            end_date: end,
            note,
            source: AbsenceSource::Import,
            created_at: Utc::now(),
        };
        if let Err(e) = state.db.save_absence_entry(&entry).await {
            tracing::error!("Failed to save imported absence: {}", e);
            continue;
        }
        entries.push(entry);
    }

    (StatusCode::CREATED, Json(ApiResponse::success(entries)))
}

async fn team_absences(
    State(state): State<SharedState>,
    Extension(_auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<ApiResponse<Vec<TeamAbsenceEntry>>>) {
    let state = state.read().await;
    let all_entries = match state.db.list_all_absence_entries().await {
        Ok(e) => e,
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Internal server error")),
            );
        }
    };
    let users = match state.db.list_users().await {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Internal server error")),
            );
        }
    };
    let user_map: std::collections::HashMap<String, String> = users
        .into_iter()
        .map(|u| (u.id.to_string(), u.name))
        .collect();

    let team: Vec<TeamAbsenceEntry> = all_entries
        .into_iter()
        .filter_map(|e| {
            let name = user_map.get(&e.user_id.to_string())?.clone();
            Some(TeamAbsenceEntry {
                user_name: name,
                absence_type: e.absence_type,
                start_date: e.start_date,
                end_date: e.end_date,
            })
        })
        .collect();

    (StatusCode::OK, Json(ApiResponse::success(team)))
}

#[derive(Debug, Deserialize)]
pub struct SetAbsencePatternRequest {
    pub absence_type: Option<String>,
    pub weekdays: Vec<u8>,
}

async fn get_absence_pattern(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<ApiResponse<Vec<AbsencePattern>>>) {
    let state = state.read().await;
    match state
        .db
        .get_absence_patterns_by_user(&auth_user.user_id.to_string())
        .await
    {
        Ok(patterns) => (StatusCode::OK, Json(ApiResponse::success(patterns))),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Internal server error")),
            )
        }
    }
}

async fn set_absence_pattern(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<SetAbsencePatternRequest>,
) -> (StatusCode, Json<ApiResponse<AbsencePattern>>) {
    let absence_type =
        parse_absence_type(&req.absence_type.unwrap_or_else(|| "homeoffice".to_string()))
            .unwrap_or(AbsenceType::Homeoffice);
    let weekdays: Vec<u8> = req.weekdays.into_iter().filter(|&d| d <= 4).collect();
    let pattern = AbsencePattern {
        user_id: auth_user.user_id,
        absence_type,
        weekdays,
    };
    let state = state.read().await;
    if let Err(e) = state.db.save_absence_pattern(&pattern).await {
        tracing::error!("Failed to save absence pattern: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error("SERVER_ERROR", "Failed to save pattern")),
        );
    }
    (StatusCode::OK, Json(ApiResponse::success(pattern)))
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
    let user = match state_guard
        .db
        .get_user(&auth_user.user_id.to_string())
        .await
    {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("FORBIDDEN", "Access denied")),
            )
        }
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("FORBIDDEN", "Admin access required")),
        );
    }
    if let Ok(Some(_)) = state_guard.db.get_user_by_username(&req.username).await {
        return (
            StatusCode::CONFLICT,
            Json(ApiResponse::error(
                "USERNAME_EXISTS",
                "Username already taken",
            )),
        );
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
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error("SERVER_ERROR", "Failed to create user")),
        );
    }
    AuditEntry::builder(AuditEventType::UserCreated)
        .user(auth_user.user_id, &user.username)
        .resource("user", &new_user.id.to_string())
        .log();
    (
        StatusCode::CREATED,
        Json(ApiResponse::success(UserResponse::from(new_user))),
    )
}

async fn admin_list_users(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<ApiResponse<Vec<UserResponse>>>) {
    let state_guard = state.read().await;
    let user = match state_guard
        .db
        .get_user(&auth_user.user_id.to_string())
        .await
    {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("FORBIDDEN", "Access denied")),
            )
        }
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("FORBIDDEN", "Admin access required")),
        );
    }
    match state_guard.db.list_users().await {
        Ok(users) => {
            let safe_users: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();
            (StatusCode::OK, Json(ApiResponse::success(safe_users)))
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Failed to list users")),
            )
        }
    }
}

async fn admin_list_bookings(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<ApiResponse<Vec<Booking>>>) {
    let state_guard = state.read().await;
    let user = match state_guard
        .db
        .get_user(&auth_user.user_id.to_string())
        .await
    {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("FORBIDDEN", "Access denied")),
            )
        }
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("FORBIDDEN", "Admin access required")),
        );
    }
    match state_guard.db.list_bookings().await {
        Ok(bookings) => (StatusCode::OK, Json(ApiResponse::success(bookings))),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(
                    "SERVER_ERROR",
                    "Failed to list bookings",
                )),
            )
        }
    }
}

async fn admin_stats(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<ApiResponse<AdminStats>>) {
    let state_guard = state.read().await;
    let user = match state_guard
        .db
        .get_user(&auth_user.user_id.to_string())
        .await
    {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("FORBIDDEN", "Access denied")),
            )
        }
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("FORBIDDEN", "Admin access required")),
        );
    }
    let db_stats = state_guard.db.stats().await.unwrap_or_default();
    let bookings = state_guard.db.list_bookings().await.unwrap_or_default();
    let now = Utc::now();
    let active_bookings = bookings
        .iter()
        .filter(|b| b.status == BookingStatus::Active || b.status == BookingStatus::Confirmed)
        .count();
    let today = now.date_naive();
    let bookings_today = bookings
        .iter()
        .filter(|b| b.created_at.date_naive() == today)
        .count();

    (
        StatusCode::OK,
        Json(ApiResponse::success(AdminStats {
            total_users: db_stats.users as i32,
            total_bookings: db_stats.bookings as i32,
            total_lots: db_stats.parking_lots as i32,
            active_bookings: active_bookings as i32,
            bookings_today: bookings_today as i32,
        })),
    )
}

// ═══════════════════════════════════════════════════════════════════════════════
// ICAL EXPORT
// ═══════════════════════════════════════════════════════════════════════════════

async fn export_ical(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
) -> impl IntoResponse {
    let state = state.read().await;
    let bookings = match state
        .db
        .list_bookings_by_user(&auth_user.user_id.to_string())
        .await
    {
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
        let summary = format!(
            "Parking: {}",
            booking.slot_number.as_deref().unwrap_or("Unknown")
        );
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
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("NOT_FOUND", "Booking not found")),
            )
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Internal server error")),
            );
        }
    };

    if booking.user_id != auth_user.user_id {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("FORBIDDEN", "Access denied")),
        );
    }

    if booking.status != BookingStatus::Confirmed {
        return (
            StatusCode::CONFLICT,
            Json(ApiResponse::error(
                "INVALID_STATUS",
                "Booking is not in confirmed status",
            )),
        );
    }

    let mut updated_booking = booking;
    updated_booking.status = BookingStatus::Active;
    updated_booking.updated_at = Utc::now();
    updated_booking.checked_in_at = Some(Utc::now());

    if let Err(e) = state_guard.db.save_booking(&updated_booking).await {
        tracing::error!("Failed to update booking: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error("SERVER_ERROR", "Failed to check in")),
        );
    }

    if let Ok(Some(mut slot)) = state_guard
        .db
        .get_parking_slot(&updated_booking.slot_id.to_string())
        .await
    {
        slot.status = SlotStatus::Occupied;
        let _ = state_guard.db.save_parking_slot(&slot).await;
    }

    AuditEntry::builder(AuditEventType::CheckIn)
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
    let user = match state_guard
        .db
        .get_user(&auth_user.user_id.to_string())
        .await
    {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::FORBIDDEN,
                [(header::CONTENT_TYPE, "application/json")],
                r#"{"success":false,"error":{"code":"FORBIDDEN","message":"Access denied"}}"#
                    .to_string(),
            )
        }
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (
            StatusCode::FORBIDDEN,
            [(header::CONTENT_TYPE, "application/json")],
            r#"{"success":false,"error":{"code":"FORBIDDEN","message":"Admin access required"}}"#
                .to_string(),
        );
    }

    let bookings = state_guard.db.list_bookings().await.unwrap_or_default();
    let lots = state_guard.db.list_parking_lots().await.unwrap_or_default();
    let db_stats = state_guard.db.stats().await.unwrap_or_default();

    let now = Utc::now();
    let today = now.date_naive();
    let active = bookings
        .iter()
        .filter(|b| b.status == BookingStatus::Active || b.status == BookingStatus::Confirmed)
        .count();
    let today_count = bookings
        .iter()
        .filter(|b| b.start_time.date_naive() == today)
        .count();
    let total_slots: i32 = lots.iter().map(|l| l.total_slots).sum();
    let occupancy_pct = if total_slots > 0 {
        active as f64 / total_slots as f64 * 100.0
    } else {
        0.0
    };

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

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        serde_json::json!({"success": true, "data": report}).to_string(),
    )
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
        _ => {
            return (
                StatusCode::NOT_FOUND,
                [(header::CONTENT_TYPE, "application/json")],
                r#"{"success":false,"error":{"code":"NOT_FOUND","message":"Lot not found"}}"#
                    .to_string(),
            )
        }
    };

    match state_guard.db.get_parking_slot(&slot_id).await {
        Ok(Some(_)) => {}
        _ => {
            return (
                StatusCode::NOT_FOUND,
                [(header::CONTENT_TYPE, "application/json")],
                r#"{"success":false,"error":{"code":"NOT_FOUND","message":"Slot not found"}}"#
                    .to_string(),
            )
        }
    };

    let booking_url = format!("/book?lot={}&slot={}", lot_id, slot_id);
    let qr = match qrcode::QrCode::new(booking_url.as_bytes()) {
        Ok(q) => q,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, [(header::CONTENT_TYPE, "application/json")],
            r#"{"success":false,"error":{"code":"SERVER_ERROR","message":"Failed to generate QR code"}}"#.to_string()),
    };

    let svg = qr
        .render::<qrcode::render::svg::Color>()
        .quiet_zone(true)
        .min_dimensions(256, 256)
        .build();
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/svg+xml")],
        svg,
    )
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
    let date = params
        .date
        .unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string());
    let state_guard = state.read().await;
    match state_guard.db.get_parking_lot(&lot_id).await {
        Ok(Some(_)) => {}
        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("NOT_FOUND", "Lot not found")),
            )
        }
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
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(
                "SERVER_ERROR",
                "Failed to join waitlist",
            )),
        );
    }
    (StatusCode::CREATED, Json(ApiResponse::success(entry)))
}

async fn get_waitlist(
    State(state): State<SharedState>,
    Path(lot_id): Path<String>,
    Query(params): Query<WaitlistParams>,
) -> Json<ApiResponse<Vec<WaitlistEntry>>> {
    let state_guard = state.read().await;
    match state_guard
        .db
        .list_waitlist_by_lot(&lot_id, params.date.as_deref())
        .await
    {
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
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(
                "SERVER_ERROR",
                "Failed to save subscription",
            )),
        );
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
    let admin = match state_guard
        .db
        .get_user(&auth_user.user_id.to_string())
        .await
    {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("FORBIDDEN", "Access denied")),
            )
        }
    };
    if admin.role != UserRole::Admin && admin.role != UserRole::SuperAdmin {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("FORBIDDEN", "Admin access required")),
        );
    }

    let mut user = match state_guard.db.get_user(&user_id).await {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("NOT_FOUND", "User not found")),
            )
        }
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
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error("SERVER_ERROR", "Failed to update user")),
        );
    }

    AuditEntry::builder(AuditEventType::UserUpdated)
        .user(auth_user.user_id, &admin.username)
        .resource("user", &user_id)
        .log();

    (
        StatusCode::OK,
        Json(ApiResponse::success(UserResponse::from(user))),
    )
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
    let user = match state_guard
        .db
        .get_user(&auth_user.user_id.to_string())
        .await
    {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("FORBIDDEN", "Access denied")),
            )
        }
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("FORBIDDEN", "Admin access required")),
        );
    }
    let mut slot = match state_guard.db.get_parking_slot(&slot_id).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("NOT_FOUND", "Slot not found")),
            )
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Internal server error")),
            );
        }
    };
    slot.reserved_for_department = req.reserved_for_department;
    if let Err(e) = state_guard.db.save_parking_slot(&slot).await {
        tracing::error!("Failed to update slot: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error("SERVER_ERROR", "Failed to update slot")),
        );
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
        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("NOT_FOUND", "User not found")),
            )
        }
    };

    let bookings = state
        .db
        .list_bookings_by_user(&uid)
        .await
        .unwrap_or_default();
    let vehicles = state
        .db
        .list_vehicles_by_user(&uid)
        .await
        .unwrap_or_default();
    let homeoffice = state.db.get_homeoffice_settings(&uid).await.ok().flatten();
    let vacation: Vec<VacationEntry> = state
        .db
        .list_vacation_entries_by_user(&uid)
        .await
        .unwrap_or_default();
    let absences: Vec<AbsenceEntry> = state
        .db
        .list_absence_entries_by_user(&uid)
        .await
        .unwrap_or_default();
    let absence_patterns = state
        .db
        .get_absence_patterns_by_user(&uid)
        .await
        .unwrap_or_default();
    let push_subs = state
        .db
        .list_push_subscriptions_by_user(&uid)
        .await
        .unwrap_or_default();

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
        "vacation_entries": vacation,
        "absence_entries": absences,
        "absence_patterns": absence_patterns,
        "push_subscriptions": push_subs.iter().map(|s| serde_json::json!({
            "endpoint": s.endpoint,
            "created_at": s.created_at,
        })).collect::<Vec<_>>(),
    });

    AuditEntry::builder(AuditEventType::UserUpdated)
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
        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("NOT_FOUND", "User not found")),
            )
        }
    };

    // Prevent SuperAdmin self-deletion
    if user.role == UserRole::SuperAdmin {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error(
                "FORBIDDEN",
                "SuperAdmin accounts cannot be self-deleted. Contact another admin.",
            )),
        );
    }

    let mut deleted = serde_json::Map::new();

    // Delete bookings
    let bookings = state
        .db
        .list_bookings_by_user(&uid)
        .await
        .unwrap_or_default();
    for b in &bookings {
        let _ = state.db.delete_booking(&b.id.to_string()).await;
    }
    deleted.insert("bookings".into(), serde_json::json!(bookings.len()));

    // Delete vehicles
    let vehicles = state
        .db
        .list_vehicles_by_user(&uid)
        .await
        .unwrap_or_default();
    for v in &vehicles {
        let _ = state.db.delete_vehicle(&v.id.to_string()).await;
    }
    deleted.insert("vehicles".into(), serde_json::json!(vehicles.len()));

    // Delete homeoffice settings
    let _ = state.db.delete_homeoffice_settings(&uid).await;
    // Delete vacation entries
    let _ = state.db.delete_vacation_entries_by_user(&uid).await;
    // Delete absence entries and patterns
    let _ = state.db.delete_absence_entries_by_user(&uid).await;
    let _ = state.db.delete_absence_patterns_by_user(&uid).await;
    deleted.insert("absence_entries".into(), serde_json::json!(true));
    deleted.insert("vacation_entries".into(), serde_json::json!(true));
    deleted.insert("homeoffice_settings".into(), serde_json::json!(true));

    // Delete push subscriptions
    let push_count = state
        .db
        .delete_push_subscriptions_by_user(&uid)
        .await
        .unwrap_or(0);
    deleted.insert("push_subscriptions".into(), serde_json::json!(push_count));

    // Delete waitlist entries
    let waitlist_count = state
        .db
        .delete_waitlist_entries_by_user(&uid)
        .await
        .unwrap_or(0);
    deleted.insert("waitlist_entries".into(), serde_json::json!(waitlist_count));

    // Finally delete the user
    let _ = state.db.delete_user(&uid).await;
    deleted.insert("user_account".into(), serde_json::json!(true));

    AuditEntry::builder(AuditEventType::UserDeleted)
        .user(auth_user.user_id, &user.username)
        .details(serde_json::json!({"action": "gdpr_account_deletion", "deleted": deleted.clone()}))
        .log();

    (
        StatusCode::OK,
        Json(ApiResponse::success(serde_json::json!({
            "message": "Account and all associated data have been permanently deleted (DSGVO Art. 17)",
            "deleted": deleted,
        }))),
    )
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
        "version": VERSION,
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
    let has_lots = state
        .db
        .list_parking_lots()
        .await
        .map(|l| !l.is_empty())
        .unwrap_or(false);
    let stats = state.db.stats().await.unwrap_or_default();

    // Check if admin still has default password
    let needs_password_change =
        if let Ok(Some(admin)) = state.db.get_user_by_username("admin").await {
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
        _ => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("FORBIDDEN", "Access denied")),
            )
        }
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("FORBIDDEN", "Admin access required")),
        );
    }

    if let Err(e) = state.db.mark_setup_completed().await {
        tracing::error!("Failed to mark setup complete: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(
                "SERVER_ERROR",
                "Failed to complete setup",
            )),
        );
    }

    AuditEntry::builder(AuditEventType::ConfigChanged)
        .user(auth_user.user_id, &user.username)
        .details(serde_json::json!({"action": "setup_completed"}))
        .log();

    (
        StatusCode::OK,
        Json(ApiResponse::success(serde_json::json!({
            "message": "Setup marked as complete",
        }))),
    )
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
        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("NOT_FOUND", "User not found")),
            )
        }
    };

    if !verify_password(&req.current_password, &user.password_hash) {
        AuditEntry::builder(AuditEventType::PasswordChanged)
            .user(auth_user.user_id, &user.username)
            .error("Invalid current password")
            .log();
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::error(
                "INVALID_PASSWORD",
                "Current password is incorrect",
            )),
        );
    }

    if req.new_password.len() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(
                "WEAK_PASSWORD",
                "Password must be at least 8 characters",
            )),
        );
    }

    user.password_hash = hash_password(&req.new_password);
    user.updated_at = Utc::now();

    if let Err(e) = state.db.save_user(&user).await {
        tracing::error!("Failed to update password: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(
                "SERVER_ERROR",
                "Failed to update password",
            )),
        );
    }

    AuditEntry::builder(AuditEventType::PasswordChanged)
        .user(auth_user.user_id, &user.username)
        .log();

    (
        StatusCode::OK,
        Json(ApiResponse::success(serde_json::json!({
            "message": "Password changed successfully",
        }))),
    )
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
    let user = match state_guard
        .db
        .get_user(&auth_user.user_id.to_string())
        .await
    {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("FORBIDDEN", "Access denied")),
            )
        }
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("FORBIDDEN", "Admin access required")),
        );
    }

    // Check lot exists
    let lot = match state_guard.db.get_parking_lot(&id).await {
        Ok(Some(l)) => l,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("NOT_FOUND", "Parking lot not found")),
            )
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Internal server error")),
            );
        }
    };

    // Delete associated slots
    let slots = state_guard
        .db
        .list_slots_by_lot(&id)
        .await
        .unwrap_or_default();
    let mut deleted_slots = 0u64;
    for slot in &slots {
        if let Ok(true) = state_guard
            .db
            .delete_parking_slot(&slot.id.to_string())
            .await
        {
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
            AuditEntry::builder(AuditEventType::ConfigChanged)
                .user(auth_user.user_id, &user.username)
                .resource("lot", &id)
                .details(serde_json::json!({
                    "action": "lot_deleted",
                    "lot_name": lot.name,
                    "deleted_slots": deleted_slots,
                    "deleted_bookings": deleted_bookings,
                }))
                .log();

            (
                StatusCode::OK,
                Json(ApiResponse::success(serde_json::json!({
                    "message": "Parking lot deleted successfully",
                    "deleted_slots": deleted_slots,
                    "deleted_bookings": deleted_bookings,
                }))),
            )
        }
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("NOT_FOUND", "Parking lot not found")),
        ),
        Err(e) => {
            tracing::error!("Failed to delete parking lot: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(
                    "SERVER_ERROR",
                    "Failed to delete parking lot",
                )),
            )
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
            company_name: std::env::var("PARKHUB_COMPANY_NAME")
                .unwrap_or_else(|_| "ParkHub".to_string()),
            primary_color: std::env::var("PARKHUB_PRIMARY_COLOR")
                .unwrap_or_else(|_| "#3B82F6".to_string()),
            secondary_color: std::env::var("PARKHUB_SECONDARY_COLOR")
                .unwrap_or_else(|_| "#1D4ED8".to_string()),
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
        _ => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("FORBIDDEN", "Access denied")),
            )
        }
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("FORBIDDEN", "Admin access required")),
        );
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
        _ => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("FORBIDDEN", "Access denied")),
            )
        }
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("FORBIDDEN", "Admin access required")),
        );
    }

    // Merge with existing branding config
    let mut final_config = load_branding_config(&state.db).await;
    if let Some(v) = req.company_name {
        final_config.company_name = v;
    }
    if let Some(v) = req.primary_color {
        final_config.primary_color = v;
    }
    if let Some(v) = req.secondary_color {
        final_config.secondary_color = v;
    }
    if let Some(v) = req.logo_url {
        final_config.logo_url = v;
    }
    if let Some(v) = req.favicon_url {
        final_config.favicon_url = v;
    }
    if let Some(v) = req.login_background_color {
        final_config.login_background_color = v;
    }
    if let Some(v) = req.custom_css {
        final_config.custom_css = v;
    }

    let json_data = serde_json::to_vec(&final_config).unwrap();
    if let Err(e) = state.db.save_branding("config", &json_data).await {
        tracing::error!("Failed to save branding config: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(
                "SERVER_ERROR",
                "Failed to save branding",
            )),
        );
    }

    AuditEntry::builder(AuditEventType::ConfigChanged)
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
    let user = match state_guard
        .db
        .get_user(&auth_user.user_id.to_string())
        .await
    {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("FORBIDDEN", "Access denied")),
            )
        }
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("FORBIDDEN", "Admin access required")),
        );
    }

    if let Ok(Some(field)) = multipart.next_field().await {
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        // Validate content type
        let ext = match content_type.as_str() {
            "image/png" => "png",
            "image/jpeg" | "image/jpg" => "jpg",
            "image/svg+xml" => "svg",
            _ => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::error(
                        "INVALID_TYPE",
                        "Only PNG, JPEG, and SVG files are accepted",
                    )),
                );
            }
        };

        let data = match field.bytes().await {
            Ok(d) => d,
            Err(e) => {
                tracing::error!("Failed to read upload: {}", e);
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::error("UPLOAD_ERROR", "Failed to read file")),
                );
            }
        };

        // Check size (2MB max)
        if data.len() > 2 * 1024 * 1024 {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error(
                    "FILE_TOO_LARGE",
                    "Logo must be under 2MB",
                )),
            );
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
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Failed to save logo")),
            );
        }

        // Update branding config with logo_url
        let mut config = load_branding_config(&state_guard.db).await;
        config.logo_url = Some("/api/v1/branding/logo".to_string());
        let config_data = serde_json::to_vec(&config).unwrap();
        let _ = state_guard.db.save_branding("config", &config_data).await;

        AuditEntry::builder(AuditEventType::ConfigChanged)
            .user(auth_user.user_id, &user.username)
            .details(serde_json::json!({"action": "logo_uploaded", "type": content_type, "size": data.len()}))
            .log();

        return (
            StatusCode::OK,
            Json(ApiResponse::success(serde_json::json!({
                "logo_url": "/api/v1/branding/logo",
                "size": data.len(),
                "content_type": content_type,
            }))),
        );
    }

    (
        StatusCode::BAD_REQUEST,
        Json(ApiResponse::error("NO_FILE", "No file uploaded")),
    )
}

/// GET /api/v1/branding/logo - Serve the uploaded logo (no auth)
async fn serve_branding_logo(State(state): State<SharedState>) -> impl IntoResponse {
    let state = state.read().await;
    match state.db.get_branding("logo").await {
        Ok(Some(data)) => {
            let logo: serde_json::Value = match serde_json::from_slice(&data) {
                Ok(v) => v,
                Err(_) => {
                    return (
                        StatusCode::NOT_FOUND,
                        [(header::CONTENT_TYPE, "text/plain".to_string())],
                        vec![],
                    )
                        .into_response()
                }
            };
            let content_type = logo["content_type"]
                .as_str()
                .unwrap_or("image/png")
                .to_string();
            let b64_data = logo["data"].as_str().unwrap_or("");
            match base64::engine::general_purpose::STANDARD.decode(b64_data) {
                Ok(bytes) => (
                    StatusCode::OK,
                    [
                        (header::CONTENT_TYPE, content_type),
                        (header::CACHE_CONTROL, "public, max-age=3600".to_string()),
                    ],
                    bytes,
                )
                    .into_response(),
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    [(header::CONTENT_TYPE, "text/plain".to_string())],
                    vec![],
                )
                    .into_response(),
            }
        }
        _ => (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain".to_string())],
            vec![],
        )
            .into_response(),
    }
}

/// Load branding config from DB, falling back to defaults
async fn load_branding_config(db: &crate::db::Database) -> BrandingConfig {
    match db.get_branding("config").await {
        Ok(Some(data)) => serde_json::from_slice(&data).unwrap_or_default(),
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

    let current_password = req
        .get("current_password")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let new_password = req
        .get("new_password")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if new_password.len() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(
                "WEAK_PASSWORD",
                "Password must be at least 8 characters",
            )),
        );
    }

    // Find admin user
    let mut admin = match state.db.get_user_by_username("admin").await {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("NOT_FOUND", "Admin user not found")),
            )
        }
    };

    if !verify_password(current_password, &admin.password_hash) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::error(
                "INVALID_PASSWORD",
                "Current password is incorrect",
            )),
        );
    }

    admin.password_hash = hash_password(new_password);
    admin.updated_at = chrono::Utc::now();

    if let Err(e) = state.db.save_user(&admin).await {
        tracing::error!("Failed to update password: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(
                "SERVER_ERROR",
                "Failed to update password",
            )),
        );
    }

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            serde_json::json!({ "message": "Password changed successfully" }),
        )),
    )
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
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(
                "INVALID_CONFIRMATION",
                "Confirmation must be exactly RESET",
            )),
        );
    }

    let state_guard = state.read().await;
    let user = match state_guard
        .db
        .get_user(&auth_user.user_id.to_string())
        .await
    {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("FORBIDDEN", "Access denied")),
            )
        }
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("FORBIDDEN", "Admin access required")),
        );
    }

    if let Err(e) = state_guard.db.reset_all_data().await {
        tracing::error!("Failed to reset database: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(
                "SERVER_ERROR",
                "Failed to reset database",
            )),
        );
    }

    let _ = state_guard
        .db
        .delete_all_users_except(&auth_user.user_id.to_string())
        .await;

    let mut admin = user;
    admin.password_hash = hash_password("admin");
    admin.updated_at = Utc::now();
    let _ = state_guard.db.save_user(&admin).await;

    tracing::info!("Database reset by admin: {}", admin.username);

    (
        StatusCode::OK,
        Json(ApiResponse::success(serde_json::json!({
            "success": true,
            "message": "Database reset complete"
        }))),
    )
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
    let user = match state_guard
        .db
        .get_user(&auth_user.user_id.to_string())
        .await
    {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("FORBIDDEN", "Access denied")),
            )
        }
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("FORBIDDEN", "Admin access required")),
        );
    }

    let mut slot = match state_guard.db.get_parking_slot(&slot_id).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("NOT_FOUND", "Slot not found")),
            )
        }
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("SERVER_ERROR", "Internal server error")),
            );
        }
    };

    if slot.lot_id.to_string() != lot_id {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(
                "LOT_MISMATCH",
                "Slot does not belong to this lot",
            )),
        );
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
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error("SERVER_ERROR", "Failed to update slot")),
        );
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

// ═══════════════════════════════════════════════════════════════════════════════
// SYSTEM VERSION & UPDATE CHECK
// ═══════════════════════════════════════════════════════════════════════════════

/// Get current system version
async fn get_system_version() -> impl IntoResponse {
    Json(serde_json::json!({
        "name": "ParkHub Server",
        "version": VERSION,
        "build_date": env!("BUILD_DATE"),
        "repo_url": "https://github.com/nash87/parkhub",
        "releases_url": "https://github.com/nash87/parkhub/releases",
        "changelog_url": "https://github.com/nash87/parkhub/blob/main/CHANGELOG.md"
    }))
}

/// Check for updates (admin only)
async fn admin_check_updates(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<serde_json::Value>) {
    let state_guard = state.read().await;
    let user = match state_guard
        .db
        .get_user(&auth_user.user_id.to_string())
        .await
    {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": "Access denied"})),
            )
        }
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        );
    }
    drop(state_guard);

    let repo_url = "https://github.com/nash87/parkhub".to_string();
    let api_url = "https://api.github.com/repos/nash87/parkhub/releases/latest";

    let client = match reqwest::Client::builder()
        .user_agent("ParkHub-Server")
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(_) => {
            return (
                StatusCode::OK,
                Json(serde_json::json!({
                    "current": VERSION,
                    "latest": serde_json::Value::Null,
                    "update_available": false,
                    "repo_url": repo_url,
                    "error": "Failed to create HTTP client",
                })),
            )
        }
    };

    match client.get(api_url).send().await {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(body) = resp.json::<serde_json::Value>().await {
                let tag = body
                    .get("tag_name")
                    .and_then(|v: &serde_json::Value| v.as_str())
                    .unwrap_or("")
                    .trim_start_matches('v')
                    .to_string();
                // Normalize versions: strip leading zeros for comparison (2026.02.09 == 2026.2.9)
                let normalize_ver = |v: &str| -> String {
                    v.split(".")
                        .map(|p| p.trim_start_matches("0").to_string())
                        .map(|p| if p.is_empty() { "0".to_string() } else { p })
                        .collect::<Vec<_>>()
                        .join(".")
                };
                let update_available =
                    !tag.is_empty() && normalize_ver(&tag) != normalize_ver(VERSION);
                let release_notes = body
                    .get("body")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let published_at = body
                    .get("published_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let release_url = body
                    .get("html_url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "current": VERSION,
                        "latest": tag,
                        "update_available": update_available,
                        "repo_url": repo_url,
                        "release_notes": release_notes,
                        "published_at": published_at,
                        "release_url": release_url,
                    })),
                )
            } else {
                (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "current": VERSION,
                        "latest": serde_json::Value::Null,
                        "update_available": false,
                        "repo_url": repo_url,
                        "error": "Failed to parse response",
                    })),
                )
            }
        }
        Ok(resp) => {
            let status = resp.status().to_string();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "current": VERSION,
                    "latest": serde_json::Value::Null,
                    "update_available": false,
                    "repo_url": repo_url,
                    "error": format!("GitHub API returned {}", status),
                })),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "current": VERSION,
                "latest": serde_json::Value::Null,
                "update_available": false,
                "repo_url": repo_url,
                "error": format!("Network error: {}", e),
            })),
        ),
    }
}

/// GET /api/v1/system/maintenance - Check maintenance mode
async fn get_maintenance_status(State(state): State<SharedState>) -> impl IntoResponse {
    let state_guard = state.read().await;
    let is_maintenance = state_guard
        .maintenance
        .load(std::sync::atomic::Ordering::Relaxed);
    Json(serde_json::json!({
        "maintenance": is_maintenance,
        "message": if is_maintenance { "Update in progress..." } else { "" },
    }))
}

/// POST /api/v1/admin/updates/apply - Apply update (admin only)
async fn admin_apply_update(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<serde_json::Value>) {
    let state_guard = state.read().await;
    let user = match state_guard
        .db
        .get_user(&auth_user.user_id.to_string())
        .await
    {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": "Access denied"})),
            )
        }
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        );
    }

    // Set maintenance mode
    state_guard
        .maintenance
        .store(true, std::sync::atomic::Ordering::Relaxed);
    drop(state_guard);

    // Determine current platform binary name
    let binary_name = if cfg!(target_os = "linux") {
        if cfg!(target_arch = "aarch64") {
            "parkhub-linux-arm64"
        } else {
            "parkhub-linux-amd64"
        }
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "parkhub-macos-arm64"
        } else {
            "parkhub-macos-amd64"
        }
    } else if cfg!(target_os = "windows") {
        "parkhub-windows-amd64.exe"
    } else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Unsupported platform"})),
        );
    };

    // Fetch latest release info
    let client = match reqwest::Client::builder()
        .user_agent("ParkHub-Server")
        .timeout(std::time::Duration::from_secs(30))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            let sg = state.read().await;
            sg.maintenance
                .store(false, std::sync::atomic::Ordering::Relaxed);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("HTTP client error: {}", e)})),
            );
        }
    };

    let api_url = "https://api.github.com/repos/nash87/parkhub/releases/latest";
    let release = match client.get(api_url).send().await {
        Ok(resp) if resp.status().is_success() => match resp.json::<serde_json::Value>().await {
            Ok(body) => body,
            Err(e) => {
                let sg = state.read().await;
                sg.maintenance
                    .store(false, std::sync::atomic::Ordering::Relaxed);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": format!("Parse error: {}", e)})),
                );
            }
        },
        _ => {
            let sg = state.read().await;
            sg.maintenance
                .store(false, std::sync::atomic::Ordering::Relaxed);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch release info"})),
            );
        }
    };

    // Find the download URL for our binary
    let download_url = release
        .get("assets")
        .and_then(|a| a.as_array())
        .and_then(|assets| {
            assets.iter().find(|a| {
                a.get("name")
                    .and_then(|n| n.as_str())
                    .map(|n| n == binary_name)
                    .unwrap_or(false)
            })
        })
        .and_then(|a| a.get("browser_download_url"))
        .and_then(|u| u.as_str());

    let download_url = match download_url {
        Some(url) => url.to_string(),
        None => {
            let sg = state.read().await;
            sg.maintenance
                .store(false, std::sync::atomic::Ordering::Relaxed);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("No binary found for platform: {}", binary_name)
                })),
            );
        }
    };

    // Download the binary
    let binary_data = match client.get(&download_url).send().await {
        Ok(resp) if resp.status().is_success() => match resp.bytes().await {
            Ok(bytes) => bytes,
            Err(e) => {
                let sg = state.read().await;
                sg.maintenance
                    .store(false, std::sync::atomic::Ordering::Relaxed);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": format!("Download error: {}", e)})),
                );
            }
        },
        _ => {
            let sg = state.read().await;
            sg.maintenance
                .store(false, std::sync::atomic::Ordering::Relaxed);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to download binary"})),
            );
        }
    };

    // Replace current binary
    let current_exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            let sg = state.read().await;
            sg.maintenance
                .store(false, std::sync::atomic::Ordering::Relaxed);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Cannot find current exe: {}", e)})),
            );
        }
    };

    let backup_path = current_exe.with_extension("bak");
    if let Err(e) = std::fs::rename(&current_exe, &backup_path) {
        let sg = state.read().await;
        sg.maintenance
            .store(false, std::sync::atomic::Ordering::Relaxed);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Backup failed: {}", e)})),
        );
    }

    if let Err(e) = std::fs::write(&current_exe, &binary_data) {
        // Try to restore backup
        let _ = std::fs::rename(&backup_path, &current_exe);
        let sg = state.read().await;
        sg.maintenance
            .store(false, std::sync::atomic::Ordering::Relaxed);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Write failed: {}", e)})),
        );
    }

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&current_exe, std::fs::Permissions::from_mode(0o755));
    }

    let tag = release
        .get("tag_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    // Schedule graceful restart
    tokio::spawn(async {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        std::process::exit(0);
    });

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": format!("Updated to {}. Server will restart shortly.", tag),
            "version": tag,
        })),
    )
}

/// GET /api/v1/admin/updates/stream?token=<jwt> - SSE stream for update progress
async fn admin_update_stream(
    State(state): State<SharedState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<
    Sse<impl futures_core::Stream<Item = Result<Event, std::convert::Infallible>>>,
    (StatusCode, Json<serde_json::Value>),
> {
    // Auth via query param (EventSource can't set headers)
    let token = params.get("token").cloned().unwrap_or_default();
    let state_guard = state.read().await;
    let session = match state_guard.db.get_session(&token).await {
        Ok(Some(s)) if !s.is_expired() => s,
        _ => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Unauthorized"})),
            ))
        }
    };
    let user = match state_guard.db.get_user(&session.user_id.to_string()).await {
        Ok(Some(u)) if u.role == UserRole::Admin || u.role == UserRole::SuperAdmin => u,
        _ => {
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": "Admin required"})),
            ))
        }
    };
    drop(state_guard);
    let _ = user;

    let state_clone = state.clone();
    let stream = async_stream::stream! {
        // Helper macro for sending SSE events
        macro_rules! send_progress {
            ($step:expr, $progress:expr, $msg:expr) => {
                let data = serde_json::json!({
                    "step": $step,
                    "progress": $progress,
                    "message": $msg,
                });
                yield Ok(Event::default().data(data.to_string()));
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            };
        }

        send_progress!("checking", 5, "Checking for updates...");

        // Check if running on NixOS
        let is_nixos = std::path::Path::new("/etc/NIXOS").exists();

        let client = match reqwest::Client::builder()
            .user_agent("ParkHub-Server")
            .timeout(std::time::Duration::from_secs(30))
            .build() {
            Ok(c) => c,
            Err(e) => {
                send_progress!("error", 0, format!("HTTP client error: {}", e));
                return;
            }
        };

        send_progress!("checking", 10, "Fetching latest release info...");

        let api_url = "https://api.github.com/repos/nash87/parkhub/releases/latest";
        let release = match client.get(api_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<serde_json::Value>().await {
                    Ok(body) => body,
                    Err(e) => {
                        send_progress!("error", 0, format!("Parse error: {}", e));
                        return;
                    }
                }
            }
            Ok(resp) => {
                send_progress!("error", 0, format!("GitHub API error: {}", resp.status()));
                return;
            }
            Err(e) => {
                send_progress!("error", 0, format!("Network error: {}", e));
                return;
            }
        };

        let tag = release.get("tag_name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let tag_clean = tag.trim_start_matches('v');
        if tag_clean == VERSION {
            send_progress!("complete", 100, "Already up to date!");
            return;
        }

        send_progress!("checking", 15, format!("Update available: v{}", tag_clean));

        if is_nixos {
            send_progress!("error", 0, "Updates are managed externally on this system.");
            return;
        }

        // Default for demo/container installs: request host-side update via shared data dir (podman pull + systemctl restart).
        // Enable legacy in-place binary self-update only when explicitly allowed.
        let allow_binary_update = std::env::var("PARKHUB_SELF_UPDATE_BINARY").ok().as_deref() == Some("1");
        if !allow_binary_update {
            send_progress!("requesting", 20, "Requesting update...");

            let req_path = state_clone.read().await.data_dir.join("update-request.json");
            let req = serde_json::json!({
                "target_tag": "latest",
                "requested_at": chrono::Utc::now().to_rfc3339(),
                "from_version": VERSION,
                "to_version": tag_clean,
            });
            if let Err(e) = std::fs::write(&req_path, req.to_string()) {
                send_progress!("error", 0, format!("Failed to write update request: {}", e));
                return;
            }

            send_progress!("restarting", 80, "Update requested. Waiting for server to restart...");
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            send_progress!("complete", 100, "Update triggered. Please refresh in a few seconds.");
            return;
        }

        // Non-NixOS: download binary
        let binary_name = if cfg!(target_os = "linux") {
            if cfg!(target_arch = "aarch64") { "parkhub-linux-arm64" } else { "parkhub-linux-amd64" }
        } else if cfg!(target_os = "macos") {
            if cfg!(target_arch = "aarch64") { "parkhub-macos-arm64" } else { "parkhub-macos-amd64" }
        } else if cfg!(target_os = "windows") {
            "parkhub-windows-amd64.exe"
        } else {
            send_progress!("error", 0, "Unsupported platform");
            return;
        };

        let download_url = release.get("assets")
            .and_then(|a| a.as_array())
            .and_then(|assets| {
                assets.iter().find(|a| {
                    a.get("name").and_then(|n| n.as_str()).map(|n| n == binary_name).unwrap_or(false)
                })
            })
            .and_then(|a| a.get("browser_download_url"))
            .and_then(|u| u.as_str());

        let download_url = match download_url {
            Some(url) => url.to_string(),
            None => {
                send_progress!("error", 0, format!("No binary found for: {}", binary_name));
                return;
            }
        };

        send_progress!("downloading", 25, format!("Downloading v{}...", tag_clean));

        let binary_data = match client.get(&download_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.bytes().await {
                    Ok(bytes) => {
                        let size_mb = bytes.len() as f64 / 1_048_576.0;
                        send_progress!("downloading", 60, format!("Downloaded {:.1} MB", size_mb));
                        bytes
                    }
                    Err(e) => {
                        send_progress!("error", 0, format!("Download error: {}", e));
                        return;
                    }
                }
            }
            _ => {
                send_progress!("error", 0, "Failed to download binary");
                return;
            }
        };

        send_progress!("installing", 70, "Testing downloaded binary...");

        // Write to temp location and test
        let temp_path = std::env::temp_dir().join("parkhub-server-new");
        if let Err(e) = std::fs::write(&temp_path, &binary_data) {
            send_progress!("error", 0, format!("Write error: {}", e));
            return;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&temp_path, std::fs::Permissions::from_mode(0o755));
        }

        // Test execution
        let test_result = tokio::process::Command::new(&temp_path)
            .arg("--version")
            .output()
            .await;

        match test_result {
            Ok(output) if output.status.success() => {
                send_progress!("installing", 80, "Binary verified, installing...");
            }
            _ => {
                let _ = std::fs::remove_file(&temp_path);
                send_progress!("error", 0, "Downloaded binary can't run on this system. Please rebuild from source: cd /home/florian/parkhub && git pull && cargo build --release -p parkhub-server --no-default-features --features headless");
                return;
            }
        }

        // Replace current binary
        let current_exe = match std::env::current_exe() {
            Ok(p) => p,
            Err(e) => {
                let _ = std::fs::remove_file(&temp_path);
                send_progress!("error", 0, format!("Cannot find current exe: {}", e));
                return;
            }
        };

        let backup_path = current_exe.with_extension("bak");
        if let Err(e) = std::fs::rename(&current_exe, &backup_path) {
            let _ = std::fs::remove_file(&temp_path);
            send_progress!("error", 0, format!("Backup failed: {}", e));
            return;
        }

        if let Err(e) = std::fs::rename(&temp_path, &current_exe) {
            let _ = std::fs::rename(&backup_path, &current_exe);
            send_progress!("error", 0, format!("Install failed: {}", e));
            return;
        }

        send_progress!("restarting", 95, "Restarting server...");

        let sg = state_clone.read().await;
        sg.maintenance.store(true, std::sync::atomic::Ordering::Relaxed);
        drop(sg);

        tokio::spawn(async {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            // Spawn new server process before exiting
            let exe = std::env::current_exe().unwrap_or_default();
            let _ = std::process::Command::new(&exe)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            std::process::exit(0);
        });

        send_progress!("complete", 100, format!("Updated to v{}! Server restarting...", tag_clean));
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

// ═══════════════════════════════════════════════════════════════════════════════
// PRIVACY SETTINGS
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    pub store_ip_addresses: bool,
    pub booking_visibility: u8,
    pub show_plates_to_users: bool,
    pub data_retention_days: u32,
    pub audit_retention_days: u32,
    pub show_booker_name: bool,
    pub license_plate_display: u8,
    #[serde(default)]
    pub license_plate_entry_mode: u8, // 0=optional, 1=required, 2=disabled
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            store_ip_addresses: false,
            booking_visibility: 1,
            show_plates_to_users: false,
            data_retention_days: 0,
            audit_retention_days: 0,
            show_booker_name: true,
            license_plate_display: 0,
            license_plate_entry_mode: 0,
        }
    }
}

/// Load privacy config from DB, falling back to server config
async fn load_privacy_config(state: &AppState) -> PrivacyConfig {
    match state.db.get_branding("privacy_config").await {
        Ok(Some(data)) => serde_json::from_slice(&data).unwrap_or(PrivacyConfig {
            store_ip_addresses: state.config.store_ip_addresses,
            booking_visibility: state.config.booking_visibility,
            show_plates_to_users: state.config.show_plates_to_users,
            data_retention_days: state.config.data_retention_days,
            audit_retention_days: state.config.audit_retention_days,
            show_booker_name: state.config.show_booker_name,
            license_plate_display: state.config.license_plate_display,
            license_plate_entry_mode: 0,
        }),
        _ => PrivacyConfig {
            store_ip_addresses: state.config.store_ip_addresses,
            booking_visibility: state.config.booking_visibility,
            show_plates_to_users: state.config.show_plates_to_users,
            data_retention_days: state.config.data_retention_days,
            audit_retention_days: state.config.audit_retention_days,
            show_booker_name: state.config.show_booker_name,
            license_plate_display: state.config.license_plate_display,
            license_plate_entry_mode: 0,
        },
    }
}

/// GET /api/v1/admin/privacy - Get privacy settings (admin only)
async fn get_admin_privacy(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<ApiResponse<PrivacyConfig>>) {
    let state = state.read().await;
    let user = match state.db.get_user(&auth_user.user_id.to_string()).await {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("FORBIDDEN", "Access denied")),
            )
        }
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("FORBIDDEN", "Admin access required")),
        );
    }
    let config = load_privacy_config(&state).await;
    (StatusCode::OK, Json(ApiResponse::success(config)))
}

/// PUT /api/v1/admin/privacy - Update privacy settings (admin only)
async fn update_admin_privacy(
    State(state): State<SharedState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<PrivacyConfig>,
) -> (StatusCode, Json<ApiResponse<PrivacyConfig>>) {
    let state = state.read().await;
    let user = match state.db.get_user(&auth_user.user_id.to_string()).await {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("FORBIDDEN", "Access denied")),
            )
        }
    };
    if user.role != UserRole::Admin && user.role != UserRole::SuperAdmin {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("FORBIDDEN", "Admin access required")),
        );
    }

    let json_data = serde_json::to_vec(&req).unwrap();
    if let Err(e) = state.db.save_branding("privacy_config", &json_data).await {
        tracing::error!("Failed to save privacy config: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(
                "SERVER_ERROR",
                "Failed to save privacy settings",
            )),
        );
    }

    AuditEntry::builder(AuditEventType::ConfigChanged)
        .user(auth_user.user_id, &user.username)
        .details(serde_json::json!({"action": "privacy_settings_updated"}))
        .log();

    (StatusCode::OK, Json(ApiResponse::success(req)))
}

/// GET /api/v1/settings/privacy - Public privacy settings (what users can see)
#[derive(Debug, Serialize)]
pub struct PublicPrivacySettings {
    pub booking_visibility: u8,
    pub show_booker_name: bool,
    pub show_plates_to_users: bool,
    pub license_plate_display: u8,
    pub license_plate_entry_mode: u8,
}

async fn get_public_privacy(
    State(state): State<SharedState>,
) -> Json<ApiResponse<PublicPrivacySettings>> {
    let state = state.read().await;
    let config = load_privacy_config(&state).await;
    Json(ApiResponse::success(PublicPrivacySettings {
        booking_visibility: config.booking_visibility,
        show_booker_name: config.show_booker_name,
        show_plates_to_users: config.show_plates_to_users,
        license_plate_display: config.license_plate_display,
        license_plate_entry_mode: config.license_plate_entry_mode,
    }))
}
