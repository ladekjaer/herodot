use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// User model representing a user account in the system
/// This model is mapped to the database table `auth.users` and contains user credentials.
/// The database is unaware of password hashing, and as such knows only the field for hashed
/// passwords as `passwords`.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub(crate) struct User {
    id: Uuid,
    username: String,
    #[sqlx(rename = "password")]
    hashed_password: String,
}

impl User {
    pub(crate) fn id(&self) -> &Uuid {
        &self.id
    }

    pub(crate) fn new(username: String, password: String) -> Result<Self, UserError> {
        let hashed_password = hash_password(password)?;
        Ok(Self { id: Uuid::new_v4(), username, hashed_password })
    }

    pub(crate) fn credentials_is(&self, credentials: UserCredentials) -> Result<bool, UserError> {
        if self.username != credentials.username {
            return Ok(false);
        }
        match self.password_matches(&credentials.password) {
            Ok(does_match) => Ok(does_match),
            Err(error) => {Err(error.into())}
        }
    }

    fn password_matches(&self, password: &str) -> Result<bool, UserError> {
        let argon2 = Argon2::default();
        let parsed_password_hash = PasswordHash::new(&self.hashed_password)?;
        Ok(argon2.verify_password(password.as_bytes(), &parsed_password_hash).is_ok())
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn hashed_password(&self) -> &str {
        &self.hashed_password
    }
}

pub(crate) struct UserCredentials {
    username: String,
    password: String,
}

impl UserCredentials {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }
}

#[derive(Debug)]
pub enum UserError {
    HashingError(argon2::password_hash::Error),
}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserError::HashingError(error) => write!(f, "Error hashing password: {}", error),
        }
    }
}

impl std::error::Error for UserError {}

impl From<argon2::password_hash::Error> for UserError {
    fn from(value: argon2::password_hash::Error) -> Self {
        UserError::HashingError(value)
    }
}

fn hash_password(password: String) -> Result<String, UserError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(hash) => Ok(hash.to_string()),
        Err(error) => Err(error.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let password = "securepassword123";
        let hashed_password = hash_password(password.to_string()).unwrap();
        assert!(hashed_password.len() > 0);
    }

    #[test]
    fn test_password_matches() {
        let password = "securepassword123";
        let wrong_password = "not the password";
        let user = User::new("testuser".to_string(), password.to_string()).unwrap();
        assert!(user.password_matches(password).unwrap());
        assert!(!user.password_matches(wrong_password).unwrap());
    }

    #[test]
    fn test_credentials_is() {
        let user = User::new("testuser".to_string(), "securepassword123".to_string()).unwrap();
        let credentials = UserCredentials::new("testuser".to_string(), "wrongpassword".to_string());
        assert!(!user.credentials_is(credentials).unwrap());
    }
}