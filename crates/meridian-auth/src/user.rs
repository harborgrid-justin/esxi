//! User management and authentication

use crate::error::{AuthError, AuthResult};
use crate::password::{PasswordHasher, PasswordPolicy, validate_password};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User account status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    /// Account is active
    Active,
    /// Account is disabled
    Disabled,
    /// Account is locked (e.g., too many failed login attempts)
    Locked,
    /// Account is pending verification
    PendingVerification,
}

/// User entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user ID
    pub id: String,
    /// User email (unique)
    pub email: String,
    /// Password hash (Argon2id)
    #[serde(skip_serializing)]
    pub password_hash: String,
    /// Display name
    pub display_name: Option<String>,
    /// User roles
    pub roles: Vec<String>,
    /// Account status
    pub status: UserStatus,
    /// Email verification status
    pub email_verified: bool,
    /// Email verification token
    #[serde(skip_serializing)]
    pub email_verification_token: Option<String>,
    /// Email verification token expiry
    pub email_verification_expires: Option<DateTime<Utc>>,
    /// Password reset token
    #[serde(skip_serializing)]
    pub password_reset_token: Option<String>,
    /// Password reset token expiry
    pub password_reset_expires: Option<DateTime<Utc>>,
    /// Failed login attempts
    pub failed_login_attempts: u32,
    /// Account locked until
    pub locked_until: Option<DateTime<Utc>>,
    /// Last login timestamp
    pub last_login: Option<DateTime<Utc>>,
    /// Account creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Additional metadata
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl User {
    /// Create a new user
    pub fn new(email: String, password: String, roles: Vec<String>) -> AuthResult<Self> {
        // Validate email
        if !is_valid_email(&email) {
            return Err(AuthError::InvalidEmail(email));
        }

        // Validate and hash password
        let policy = PasswordPolicy::default();
        validate_password(&password, &policy)?;

        let hasher = PasswordHasher::new();
        let password_hash = hasher.hash_password(&password)?;

        Ok(Self {
            id: Uuid::new_v4().to_string(),
            email,
            password_hash,
            display_name: None,
            roles,
            status: UserStatus::PendingVerification,
            email_verified: false,
            email_verification_token: Some(generate_token()),
            email_verification_expires: Some(Utc::now() + Duration::days(1)),
            password_reset_token: None,
            password_reset_expires: None,
            failed_login_attempts: 0,
            locked_until: None,
            last_login: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: serde_json::Value::Null,
        })
    }

    /// Verify password
    pub fn verify_password(&self, password: &str) -> AuthResult<bool> {
        // Check if account is locked
        if let Some(locked_until) = self.locked_until {
            if Utc::now() < locked_until {
                return Err(AuthError::UserLocked);
            }
        }

        // Check if account is disabled
        if self.status == UserStatus::Disabled {
            return Err(AuthError::UserDisabled);
        }

        let hasher = PasswordHasher::new();
        hasher.verify_password(password, &self.password_hash)
    }

    /// Authenticate user with password
    pub fn authenticate(&mut self, password: &str) -> AuthResult<()> {
        match self.verify_password(password)? {
            true => {
                // Reset failed attempts on successful login
                self.failed_login_attempts = 0;
                self.locked_until = None;
                self.last_login = Some(Utc::now());
                self.updated_at = Utc::now();
                Ok(())
            }
            false => {
                // Increment failed attempts
                self.failed_login_attempts += 1;
                self.updated_at = Utc::now();

                // Lock account after 5 failed attempts
                if self.failed_login_attempts >= 5 {
                    self.locked_until = Some(Utc::now() + Duration::minutes(30));
                    self.status = UserStatus::Locked;
                    return Err(AuthError::UserLocked);
                }

                Err(AuthError::InvalidPassword("Incorrect password".to_string()))
            }
        }
    }

    /// Update password
    pub fn update_password(&mut self, old_password: &str, new_password: &str) -> AuthResult<()> {
        // Verify old password
        if !self.verify_password(old_password)? {
            return Err(AuthError::InvalidPassword("Current password is incorrect".to_string()));
        }

        // Validate new password
        let policy = PasswordPolicy::default();
        validate_password(new_password, &policy)?;

        // Hash new password
        let hasher = PasswordHasher::new();
        self.password_hash = hasher.hash_password(new_password)?;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Generate email verification token
    pub fn generate_email_verification_token(&mut self) -> String {
        let token = generate_token();
        self.email_verification_token = Some(token.clone());
        self.email_verification_expires = Some(Utc::now() + Duration::days(1));
        self.updated_at = Utc::now();
        token
    }

    /// Verify email with token
    pub fn verify_email(&mut self, token: &str) -> AuthResult<()> {
        // Check if token matches
        if self.email_verification_token.as_deref() != Some(token) {
            return Err(AuthError::InvalidToken("Invalid verification token".to_string()));
        }

        // Check if token is expired
        if let Some(expires) = self.email_verification_expires {
            if Utc::now() > expires {
                return Err(AuthError::TokenExpired);
            }
        }

        // Mark email as verified
        self.email_verified = true;
        self.email_verification_token = None;
        self.email_verification_expires = None;
        self.status = UserStatus::Active;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Generate password reset token
    pub fn generate_password_reset_token(&mut self) -> String {
        let token = generate_token();
        self.password_reset_token = Some(token.clone());
        self.password_reset_expires = Some(Utc::now() + Duration::hours(1));
        self.updated_at = Utc::now();
        token
    }

    /// Reset password with token
    pub fn reset_password(&mut self, token: &str, new_password: &str) -> AuthResult<()> {
        // Check if token matches
        if self.password_reset_token.as_deref() != Some(token) {
            return Err(AuthError::InvalidToken("Invalid reset token".to_string()));
        }

        // Check if token is expired
        if let Some(expires) = self.password_reset_expires {
            if Utc::now() > expires {
                return Err(AuthError::TokenExpired);
            }
        }

        // Validate new password
        let policy = PasswordPolicy::default();
        validate_password(new_password, &policy)?;

        // Hash new password
        let hasher = PasswordHasher::new();
        self.password_hash = hasher.hash_password(new_password)?;

        // Clear reset token
        self.password_reset_token = None;
        self.password_reset_expires = None;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Update user profile
    pub fn update_profile(&mut self, display_name: Option<String>) -> AuthResult<()> {
        self.display_name = display_name;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Add role to user
    pub fn add_role(&mut self, role: String) {
        if !self.roles.contains(&role) {
            self.roles.push(role);
            self.updated_at = Utc::now();
        }
    }

    /// Remove role from user
    pub fn remove_role(&mut self, role: &str) {
        self.roles.retain(|r| r != role);
        self.updated_at = Utc::now();
    }

    /// Check if user has role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Lock user account
    pub fn lock_account(&mut self, duration: Duration) {
        self.status = UserStatus::Locked;
        self.locked_until = Some(Utc::now() + duration);
        self.updated_at = Utc::now();
    }

    /// Unlock user account
    pub fn unlock_account(&mut self) {
        self.status = UserStatus::Active;
        self.locked_until = None;
        self.failed_login_attempts = 0;
        self.updated_at = Utc::now();
    }

    /// Disable user account
    pub fn disable_account(&mut self) {
        self.status = UserStatus::Disabled;
        self.updated_at = Utc::now();
    }

    /// Enable user account
    pub fn enable_account(&mut self) {
        self.status = UserStatus::Active;
        self.updated_at = Utc::now();
    }

    /// Check if account is active and ready
    pub fn is_active(&self) -> bool {
        self.status == UserStatus::Active && self.email_verified
    }
}

/// User creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
    pub roles: Vec<String>,
}

/// User update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub roles: Option<Vec<String>>,
}

/// Password change request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

/// Password reset request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
}

/// Password reset confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetConfirmation {
    pub token: String,
    pub new_password: String,
}

/// Email verification request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailVerificationRequest {
    pub token: String,
}

/// Validate email format
fn is_valid_email(email: &str) -> bool {
    // Basic email validation
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }

    let local = parts[0];
    let domain = parts[1];

    // Check local part
    if local.is_empty() || local.len() > 64 {
        return false;
    }

    // Check domain part
    if domain.is_empty() || domain.len() > 255 {
        return false;
    }

    // Domain must have at least one dot
    if !domain.contains('.') {
        return false;
    }

    // Basic character validation
    email.chars().all(|c| c.is_alphanumeric() || "@.-_+".contains(c))
}

/// Generate a secure random token
fn generate_token() -> String {
    use base64::Engine;
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    base64::engine::general_purpose::STANDARD.encode(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new(
            "user@example.com".to_string(),
            "SecurePassword123!".to_string(),
            vec!["viewer".to_string()],
        )
        .unwrap();

        assert_eq!(user.email, "user@example.com");
        assert_eq!(user.status, UserStatus::PendingVerification);
        assert!(!user.email_verified);
        assert!(user.email_verification_token.is_some());
    }

    #[test]
    fn test_password_verification() {
        let user = User::new(
            "user@example.com".to_string(),
            "SecurePassword123!".to_string(),
            vec!["viewer".to_string()],
        )
        .unwrap();

        // Correct password - but account is pending verification
        // We need to verify email first or change status
        let mut active_user = user.clone();
        active_user.status = UserStatus::Active;

        assert!(active_user.verify_password("SecurePassword123!").unwrap());
        assert!(!active_user.verify_password("WrongPassword").unwrap());
    }

    #[test]
    fn test_authentication() {
        let mut user = User::new(
            "user@example.com".to_string(),
            "SecurePassword123!".to_string(),
            vec!["viewer".to_string()],
        )
        .unwrap();

        // Activate user
        user.status = UserStatus::Active;

        // Successful authentication
        assert!(user.authenticate("SecurePassword123!").is_ok());
        assert_eq!(user.failed_login_attempts, 0);

        // Failed authentication
        assert!(user.authenticate("WrongPassword").is_err());
        assert_eq!(user.failed_login_attempts, 1);
    }

    #[test]
    fn test_account_locking() {
        let mut user = User::new(
            "user@example.com".to_string(),
            "SecurePassword123!".to_string(),
            vec!["viewer".to_string()],
        )
        .unwrap();

        user.status = UserStatus::Active;

        // Attempt wrong password 5 times
        for _ in 0..5 {
            let _ = user.authenticate("WrongPassword");
        }

        assert_eq!(user.status, UserStatus::Locked);
        assert!(user.locked_until.is_some());
    }

    #[test]
    fn test_email_verification() {
        let mut user = User::new(
            "user@example.com".to_string(),
            "SecurePassword123!".to_string(),
            vec!["viewer".to_string()],
        )
        .unwrap();

        let token = user.email_verification_token.clone().unwrap();

        // Verify email
        assert!(user.verify_email(&token).is_ok());
        assert!(user.email_verified);
        assert_eq!(user.status, UserStatus::Active);
        assert!(user.email_verification_token.is_none());
    }

    #[test]
    fn test_password_reset() {
        let mut user = User::new(
            "user@example.com".to_string(),
            "SecurePassword123!".to_string(),
            vec!["viewer".to_string()],
        )
        .unwrap();

        user.status = UserStatus::Active;

        // Generate reset token
        let token = user.generate_password_reset_token();

        // Reset password
        assert!(user.reset_password(&token, "NewSecurePassword456!").is_ok());

        // Old password should not work
        assert!(!user.verify_password("SecurePassword123!").unwrap());

        // New password should work
        assert!(user.verify_password("NewSecurePassword456!").unwrap());
    }

    #[test]
    fn test_role_management() {
        let mut user = User::new(
            "user@example.com".to_string(),
            "SecurePassword123!".to_string(),
            vec!["viewer".to_string()],
        )
        .unwrap();

        assert!(user.has_role("viewer"));
        assert!(!user.has_role("admin"));

        user.add_role("admin".to_string());
        assert!(user.has_role("admin"));

        user.remove_role("viewer");
        assert!(!user.has_role("viewer"));
    }

    #[test]
    fn test_email_validation() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("user.name@example.co.uk"));
        assert!(!is_valid_email("invalid"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("user@"));
        assert!(!is_valid_email("user@domain"));
    }
}
