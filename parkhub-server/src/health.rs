//! Health Check Types (used by OpenAPI schema)

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Health check response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub version: String,
    pub uptime_seconds: u64,
    pub checks: Vec<ComponentHealth>,
}

/// Health status
#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Individual component health
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_time_ms: Option<u64>,
}

/// Readiness response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ReadyResponse {
    pub ready: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
