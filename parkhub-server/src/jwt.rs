//! JWT Authentication Module
//!
//! Available but currently unused - the server uses UUID session tokens.
//! Kept for future migration to stateless JWT auth.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::error::AppError;

/// JWT configuration
#[derive(Clone)]
#[allow(dead_code)]
pub struct JwtConfig {
    pub secret: String,
    pub access_token_expiry_hours: i64,
    pub refresh_token_expiry_days: i64,
    pub issuer: String,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: Uuid::new_v4().to_string(),
            access_token_expiry_hours: 24,
            refresh_token_expiry_days: 30,
            issuer: "parkhub".to_string(),
        }
    }
}

/// JWT Claims
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub role: String,
    pub iat: i64,
    pub exp: i64,
    pub iss: String,
    pub token_type: TokenType,
}

/// Token type
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[allow(dead_code)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

/// Token pair (access + refresh) - used in OpenAPI schema
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// JWT Manager for creating and validating tokens
#[derive(Clone)]
#[allow(dead_code)]
pub struct JwtManager {
    config: JwtConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtManager {
    #[allow(dead_code)]
    pub fn new(config: JwtConfig) -> Self {
        let encoding_key = EncodingKey::from_secret(config.secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.secret.as_bytes());
        Self { config, encoding_key, decoding_key }
    }

    #[allow(dead_code)]
    pub fn generate_tokens(&self, user_id: &Uuid, username: &str, role: &str) -> Result<TokenPair, AppError> {
        let now = Utc::now();
        let access_exp = now + Duration::hours(self.config.access_token_expiry_hours);
        let access_claims = Claims {
            sub: user_id.to_string(), username: username.to_string(), role: role.to_string(),
            iat: now.timestamp(), exp: access_exp.timestamp(),
            iss: self.config.issuer.clone(), token_type: TokenType::Access,
        };
        let access_token = encode(&Header::default(), &access_claims, &self.encoding_key)
            .map_err(|e| AppError::InvalidInput(format!("Failed to create token: {}", e)))?;

        let refresh_exp = now + Duration::days(self.config.refresh_token_expiry_days);
        let refresh_claims = Claims {
            sub: user_id.to_string(), username: username.to_string(), role: role.to_string(),
            iat: now.timestamp(), exp: refresh_exp.timestamp(),
            iss: self.config.issuer.clone(), token_type: TokenType::Refresh,
        };
        let refresh_token = encode(&Header::default(), &refresh_claims, &self.encoding_key)
            .map_err(|e| AppError::InvalidInput(format!("Failed to create token: {}", e)))?;

        Ok(TokenPair {
            access_token, refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.access_token_expiry_hours * 3600,
        })
    }

    #[allow(dead_code)]
    pub fn validate_token(&self, token: &str) -> Result<Claims, AppError> {
        let mut validation = Validation::default();
        validation.set_issuer(&[&self.config.issuer]);
        let token_data: TokenData<Claims> = decode(token, &self.decoding_key, &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::TokenExpired,
                _ => AppError::InvalidToken,
            })?;
        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_validate_tokens() {
        let jwt = JwtManager::new(JwtConfig::default());
        let user_id = Uuid::new_v4();
        let tokens = jwt.generate_tokens(&user_id, "testuser", "user").unwrap();
        assert!(!tokens.access_token.is_empty());
        let claims = jwt.validate_token(&tokens.access_token).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
    }

    #[test]
    fn test_invalid_token() {
        let jwt = JwtManager::new(JwtConfig::default());
        assert!(jwt.validate_token("invalid.token.here").is_err());
    }
}
