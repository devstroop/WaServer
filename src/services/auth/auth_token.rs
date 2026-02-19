//! Authentication Token Service
//!
//! Handles JWT token generation, validation, and user authentication.

use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::sync::RwLock;

use crate::models::auth::{Claims, LoginResponse, RefreshTokenResponse};

// Re-export AuthError from models
pub use crate::models::error::AuthError;

/// Local user storage (in-memory for simplicity)
#[derive(Debug, Clone)]
pub struct LocalUser {
    pub username: String,
    pub password_hash: String,
}

/// Authentication Token Service
pub struct AuthTokenService {
    jwt_secret: String,
    access_token_expiry_hours: i64,
    refresh_token_expiry_days: i64,
    users: RwLock<Vec<LocalUser>>,
    /// Store invalidated refresh tokens (for logout)
    revoked_tokens: RwLock<Vec<String>>,
}

impl AuthTokenService {
    /// Create a new AuthTokenService with configuration
    pub fn new(
        jwt_secret: String,
        access_token_expiry_hours: i64,
        refresh_token_expiry_days: i64,
        default_username: Option<String>,
        default_password: Option<String>,
    ) -> Result<Self, AuthError> {
        let service = Self {
            jwt_secret,
            access_token_expiry_hours,
            refresh_token_expiry_days,
            users: RwLock::new(Vec::new()),
            revoked_tokens: RwLock::new(Vec::new()),
        };

        // Create default user if credentials provided
        if let (Some(username), Some(password)) = (default_username, default_password) {
            service.create_user(&username, &password)?;
            tracing::info!("Default user '{}' created", username);
        }

        Ok(service)
    }

    /// Create a new user with hashed password
    pub fn create_user(&self, username: &str, password: &str) -> Result<(), AuthError> {
        let password_hash = hash(password, DEFAULT_COST)
            .map_err(|e| AuthError::HashingFailed(e.to_string()))?;

        let user = LocalUser {
            username: username.to_string(),
            password_hash,
        };

        let mut users = self.users.write().unwrap();
        
        // Check if user already exists
        if users.iter().any(|u| u.username == username) {
            tracing::debug!("User '{}' already exists, skipping creation", username);
            return Ok(());
        }

        users.push(user);
        Ok(())
    }

    /// Authenticate user and return tokens
    pub fn login(&self, username: &str, password: &str) -> Result<LoginResponse, AuthError> {
        // Find user
        let users = self.users.read().unwrap();
        let user = users
            .iter()
            .find(|u| u.username == username)
            .ok_or(AuthError::InvalidCredentials)?;

        // Verify password
        if !verify(password, &user.password_hash).unwrap_or(false) {
            return Err(AuthError::InvalidCredentials);
        }

        // Generate tokens
        let access_token = self.generate_access_token(username)?;
        let refresh_token = self.generate_refresh_token(username)?;

        let expires_in = self.access_token_expiry_hours * 3600;

        Ok(LoginResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
            username: username.to_string(),
        })
    }

    /// Refresh access token using refresh token
    pub fn refresh_token(&self, refresh_token: &str) -> Result<RefreshTokenResponse, AuthError> {
        // Check if token is revoked
        {
            let revoked = self.revoked_tokens.read().unwrap();
            if revoked.contains(&refresh_token.to_string()) {
                return Err(AuthError::InvalidToken);
            }
        }

        // Validate refresh token
        let claims = self.validate_token(refresh_token)?;

        // Ensure it's a refresh token
        if claims.token_type != "refresh" {
            return Err(AuthError::InvalidToken);
        }

        // Generate new access token
        let access_token = self.generate_access_token(&claims.sub)?;
        let expires_in = self.access_token_expiry_hours * 3600;

        Ok(RefreshTokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in,
        })
    }

    /// Logout - revoke refresh token
    pub fn logout(&self, refresh_token: &str) {
        let mut revoked = self.revoked_tokens.write().unwrap();
        if !revoked.contains(&refresh_token.to_string()) {
            revoked.push(refresh_token.to_string());
        }
    }

    /// Validate a token and return claims
    pub fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        let validation = Validation::default();
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &validation,
        )
        .map_err(|e| {
            tracing::debug!("Token validation failed: {:?}", e);
            match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                _ => AuthError::InvalidToken,
            }
        })?;

        Ok(token_data.claims)
    }

    /// Validate access token specifically
    pub fn validate_access_token(&self, token: &str) -> Result<String, AuthError> {
        let claims = self.validate_token(token)?;

        // Ensure it's an access token
        if claims.token_type != "access" {
            return Err(AuthError::InvalidToken);
        }

        Ok(claims.sub)
    }

    /// Generate access token
    fn generate_access_token(&self, username: &str) -> Result<String, AuthError> {
        let now = Utc::now();
        let expiry = now + Duration::hours(self.access_token_expiry_hours);

        let claims = Claims {
            sub: username.to_string(),
            exp: expiry.timestamp(),
            iat: now.timestamp(),
            token_type: "access".to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| AuthError::TokenGenerationFailed(e.to_string()))
    }

    /// Generate refresh token
    fn generate_refresh_token(&self, username: &str) -> Result<String, AuthError> {
        let now = Utc::now();
        let expiry = now + Duration::days(self.refresh_token_expiry_days);

        let claims = Claims {
            sub: username.to_string(),
            exp: expiry.timestamp(),
            iat: now.timestamp(),
            token_type: "refresh".to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| AuthError::TokenGenerationFailed(e.to_string()))
    }

    /// Check if a user exists
    pub fn user_exists(&self, username: &str) -> bool {
        let users = self.users.read().unwrap();
        users.iter().any(|u| u.username == username)
    }
}
