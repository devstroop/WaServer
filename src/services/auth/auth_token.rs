//! Authentication Token Service
//!
//! Handles JWT token generation, validation, and user authentication.
//! Users are stored in SQLite database for persistence across restarts.

use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::models::auth::{Claims, LoginResponse, RefreshTokenResponse, ForgotPasswordResponse, ResetPasswordResponse};
use crate::models::user::User;
use crate::services::database::DatabaseService;

// Re-export AuthError from models
pub use crate::models::error::AuthError;

/// Password reset token expiry in minutes
const RESET_TOKEN_EXPIRY_MINUTES: i64 = 15;

/// Authentication Token Service
/// 
/// Manages user authentication with JWT tokens. Users are stored in SQLite
/// for persistence across server restarts.
pub struct AuthTokenService {
    jwt_secret: String,
    access_token_expiry_hours: i64,
    refresh_token_expiry_days: i64,
    /// Database service for persistent user storage
    db: Arc<DatabaseService>,
    /// One-time setup token for initial admin creation (in-memory, regenerated on restart)
    setup_token: RwLock<Option<String>>,
}

impl AuthTokenService {
    /// Create a new AuthTokenService with configuration and database
    pub fn new(
        jwt_secret: String,
        access_token_expiry_hours: i64,
        refresh_token_expiry_days: i64,
        db: Arc<DatabaseService>,
    ) -> Result<Self, AuthError> {
        // Check if we need initial setup (no users exist)
        let needs_setup = !db.has_users().map_err(|e| {
            AuthError::DatabaseError(format!("Failed to check users: {}", e))
        })?;
        
        // Generate a setup token only if needed
        let setup_token = if needs_setup {
            Some(Uuid::new_v4().to_string())
        } else {
            None
        };
        
        let service = Self {
            jwt_secret,
            access_token_expiry_hours,
            refresh_token_expiry_days,
            db,
            setup_token: RwLock::new(setup_token),
        };

        // Clean up expired tokens on startup
        let _ = service.db.cleanup_expired_tokens();
        let _ = service.db.cleanup_expired_reset_tokens();

        Ok(service)
    }
    
    /// Get the setup token (if setup is still needed)
    pub fn get_setup_token(&self) -> Option<String> {
        self.setup_token.read().unwrap().clone()
    }
    
    /// Check if initial setup is needed (no users exist)
    pub fn needs_setup(&self) -> bool {
        !self.db.has_users().unwrap_or(true)
    }
    
    /// Complete initial setup by creating the first admin user
    pub fn complete_setup(&self, setup_token: &str, username: &str, password: &str) -> Result<(), AuthError> {
        // Verify setup token
        {
            let token = self.setup_token.read().unwrap();
            match &*token {
                Some(t) if t == setup_token => {}
                _ => return Err(AuthError::InvalidToken),
            }
        }
        
        // Check if setup already completed
        if !self.needs_setup() {
            return Err(AuthError::SetupAlreadyComplete);
        }
        
        // Validate username and password
        if username.trim().is_empty() {
            return Err(AuthError::ValidationFailed("Username cannot be empty".to_string()));
        }
        if password.len() < 8 {
            return Err(AuthError::ValidationFailed("Password must be at least 8 characters".to_string()));
        }
        
        // Create the admin user
        self.create_admin_user(username, password)?;
        
        // Invalidate the setup token
        {
            let mut token = self.setup_token.write().unwrap();
            *token = None;
        }
        
        tracing::info!("Initial admin user '{}' created via setup", username);
        Ok(())
    }

    /// Create a new admin user with hashed password
    fn create_admin_user(&self, username: &str, password: &str) -> Result<(), AuthError> {
        let password_hash = hash(password, DEFAULT_COST)
            .map_err(|e| AuthError::HashingFailed(e.to_string()))?;

        let user = User::new_admin(username, &password_hash);
        
        self.db.create_user(&user).map_err(|e| {
            AuthError::DatabaseError(format!("Failed to create user: {}", e))
        })?;

        Ok(())
    }

    /// Create a new regular user with hashed password
    pub fn create_user(&self, username: &str, password: &str) -> Result<(), AuthError> {
        // Check if user already exists
        if self.user_exists(username) {
            tracing::debug!("User '{}' already exists, skipping creation", username);
            return Ok(());
        }

        let password_hash = hash(password, DEFAULT_COST)
            .map_err(|e| AuthError::HashingFailed(e.to_string()))?;

        let user = User::new(username, &password_hash);
        
        self.db.create_user(&user).map_err(|e| {
            AuthError::DatabaseError(format!("Failed to create user: {}", e))
        })?;

        Ok(())
    }

    /// Authenticate user and return tokens
    pub fn login(&self, username: &str, password: &str) -> Result<LoginResponse, AuthError> {
        // Find user
        let user = self.db.get_user_by_username(username)
            .map_err(|e| AuthError::DatabaseError(format!("Database error: {}", e)))?
            .ok_or(AuthError::InvalidCredentials)?;

        // Check if user is active
        if !user.is_active {
            return Err(AuthError::AccountDisabled);
        }

        // Verify password
        if !verify(password, &user.password_hash).unwrap_or(false) {
            return Err(AuthError::InvalidCredentials);
        }

        // Update last login timestamp
        let _ = self.db.update_user_last_login(user.id);

        // Generate tokens
        let access_token = self.generate_access_token(username, user.id)?;
        let refresh_token = self.generate_refresh_token(username, user.id)?;

        // Store refresh token for tracking
        let token_id = Self::hash_token(&refresh_token);
        let expires_at = Utc::now() + Duration::days(self.refresh_token_expiry_days);
        let _ = self.db.store_refresh_token(&token_id, user.id, expires_at, None, None);

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
        let token_id = Self::hash_token(refresh_token);
        if !self.db.is_refresh_token_valid(&token_id).unwrap_or(false) {
            return Err(AuthError::InvalidToken);
        }

        // Validate refresh token
        let claims = self.validate_token(refresh_token)?;

        // Ensure it's a refresh token
        if claims.token_type != "refresh" {
            return Err(AuthError::InvalidToken);
        }

        // Get user to get ID for new token
        let user = self.db.get_user_by_username(&claims.sub)
            .map_err(|e| AuthError::DatabaseError(format!("Database error: {}", e)))?
            .ok_or(AuthError::InvalidCredentials)?;

        // Generate new access token
        let access_token = self.generate_access_token(&claims.sub, user.id)?;
        let expires_in = self.access_token_expiry_hours * 3600;

        Ok(RefreshTokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in,
        })
    }

    /// Logout - revoke refresh token
    pub fn logout(&self, refresh_token: &str) {
        let token_id = Self::hash_token(refresh_token);
        let _ = self.db.revoke_refresh_token(&token_id);
    }

    /// Logout from all devices
    pub fn logout_all(&self, username: &str) -> Result<usize, AuthError> {
        let user = self.db.get_user_by_username(username)
            .map_err(|e| AuthError::DatabaseError(format!("Database error: {}", e)))?
            .ok_or(AuthError::InvalidCredentials)?;

        self.db.revoke_all_user_tokens(user.id)
            .map_err(|e| AuthError::DatabaseError(format!("Failed to revoke tokens: {}", e)))
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
    fn generate_access_token(&self, username: &str, user_id: Uuid) -> Result<String, AuthError> {
        let now = Utc::now();
        let expiry = now + Duration::hours(self.access_token_expiry_hours);

        let claims = Claims {
            sub: username.to_string(),
            exp: expiry.timestamp(),
            iat: now.timestamp(),
            token_type: "access".to_string(),
        };

        // Log user_id for audit purposes
        tracing::debug!("Generated access token for user {} ({})", username, user_id);

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| AuthError::TokenGenerationFailed(e.to_string()))
    }

    /// Generate refresh token
    fn generate_refresh_token(&self, username: &str, user_id: Uuid) -> Result<String, AuthError> {
        let now = Utc::now();
        let expiry = now + Duration::days(self.refresh_token_expiry_days);

        let claims = Claims {
            sub: username.to_string(),
            exp: expiry.timestamp(),
            iat: now.timestamp(),
            token_type: "refresh".to_string(),
        };

        tracing::debug!("Generated refresh token for user {} ({})", username, user_id);

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| AuthError::TokenGenerationFailed(e.to_string()))
    }

    /// Check if a user exists
    pub fn user_exists(&self, username: &str) -> bool {
        self.db.get_user_by_username(username)
            .map(|u| u.is_some())
            .unwrap_or(false)
    }

    /// Get user by username
    pub fn get_user(&self, username: &str) -> Result<Option<User>, AuthError> {
        self.db.get_user_by_username(username)
            .map_err(|e| AuthError::DatabaseError(format!("Database error: {}", e)))
    }

    /// Hash a token for storage (we don't store raw tokens)
    fn hash_token(token: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        token.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    // =========================================================================
    // Password Reset Methods
    // =========================================================================

    /// Initiate password reset - generate a reset token
    pub fn forgot_password(&self, username: &str) -> Result<ForgotPasswordResponse, AuthError> {
        // Verify user exists
        let user = match self.db.get_user_by_username(username) {
            Ok(Some(u)) => u,
            _ => {
                // Don't reveal whether user exists for security
                return Ok(ForgotPasswordResponse {
                    message: "If the username exists, a reset token has been generated.".to_string(),
                    reset_token: None,
                    expires_in: RESET_TOKEN_EXPIRY_MINUTES * 60,
                });
            }
        };

        // Clean up expired tokens first
        let _ = self.db.cleanup_expired_reset_tokens();

        // Generate reset token
        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + Duration::minutes(RESET_TOKEN_EXPIRY_MINUTES);

        // Store the reset token
        self.db.store_password_reset_token(&token, user.id, expires_at)
            .map_err(|e| AuthError::DatabaseError(format!("Failed to store reset token: {}", e)))?;

        tracing::info!("Password reset token generated for user '{}'", username);

        Ok(ForgotPasswordResponse {
            message: "Password reset token generated. Use it to reset your password.".to_string(),
            reset_token: Some(token),
            expires_in: RESET_TOKEN_EXPIRY_MINUTES * 60,
        })
    }

    /// Reset password using a reset token
    pub fn reset_password(&self, reset_token: &str, new_password: &str) -> Result<ResetPasswordResponse, AuthError> {
        // Validate new password
        if new_password.len() < 8 {
            return Err(AuthError::ValidationFailed("Password must be at least 8 characters".to_string()));
        }

        // Find and validate the reset token
        let user_id = self.db.validate_password_reset_token(reset_token)
            .map_err(|e| AuthError::DatabaseError(format!("Database error: {}", e)))?
            .ok_or(AuthError::InvalidToken)?;

        // Hash the new password
        let password_hash = hash(new_password, DEFAULT_COST)
            .map_err(|e| AuthError::HashingFailed(e.to_string()))?;

        // Update the user's password
        self.db.update_user_password(user_id, &password_hash)
            .map_err(|e| AuthError::DatabaseError(format!("Failed to update password: {}", e)))?;

        // Mark token as used
        let _ = self.db.use_password_reset_token(reset_token);

        // Revoke all refresh tokens for this user (force re-login)
        let _ = self.db.revoke_all_user_tokens(user_id);

        tracing::info!("Password reset successfully for user ID: {}", user_id);

        Ok(ResetPasswordResponse {
            message: "Password has been reset successfully. Please login with your new password.".to_string(),
        })
    }

    /// Change password when already authenticated
    pub fn change_password(&self, username: &str, current_password: &str, new_password: &str) -> Result<ResetPasswordResponse, AuthError> {
        // Validate new password
        if new_password.len() < 8 {
            return Err(AuthError::ValidationFailed("New password must be at least 8 characters".to_string()));
        }

        // Get user and verify current password
        let user = self.db.get_user_by_username(username)
            .map_err(|e| AuthError::DatabaseError(format!("Database error: {}", e)))?
            .ok_or(AuthError::InvalidCredentials)?;

        if !verify(current_password, &user.password_hash).unwrap_or(false) {
            return Err(AuthError::InvalidCredentials);
        }

        // Hash the new password
        let password_hash = hash(new_password, DEFAULT_COST)
            .map_err(|e| AuthError::HashingFailed(e.to_string()))?;

        // Update the user's password
        self.db.update_user_password(user.id, &password_hash)
            .map_err(|e| AuthError::DatabaseError(format!("Failed to update password: {}", e)))?;

        tracing::info!("Password changed successfully for user '{}'", username);

        Ok(ResetPasswordResponse {
            message: "Password changed successfully.".to_string(),
        })
    }

    /// Get database reference (for handlers that need direct access)
    pub fn database(&self) -> &Arc<DatabaseService> {
        &self.db
    }
}
