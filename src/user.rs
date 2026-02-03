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
    pub(crate) fn credentials_is(&self, credentials: UserCredentials) -> bool {
        self.username == credentials.username && self.password_matches(&credentials.password)
    }

    fn password_matches(&self, password: &str) -> bool {
        self.hashed_password == password
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
