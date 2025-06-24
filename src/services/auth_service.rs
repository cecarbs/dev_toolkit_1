use crate::modes::automation::Credentials;
use std::sync::{Arc, Mutex};

/// Authentication service for managing user credentials throughout the application
#[derive(Debug, Clone)]
pub struct AuthService {
    credentials: Arc<Mutex<Option<Credentials>>>,
}

impl AuthService {
    /// Create a new auth service instance
    pub fn new() -> Self {
        Self {
            credentials: Arc::new(Mutex::new(None)),
        }
    }

    /// Store credentials in memory for the session
    pub fn store_credentials(&self, username: String, password: String) -> Result<(), String> {
        match self.credentials.lock() {
            Ok(mut creds) => {
                *creds = Some(Credentials { username, password });
                Ok(())
            }
            Err(_) => Err("Failed to acquire credentials lock".to_string()),
        }
    }

    /// Retrieve stored credentials
    pub fn get_credentials(&self) -> Option<Credentials> {
        match self.credentials.lock() {
            Ok(creds) => creds.clone(),
            Err(_) => None,
        }
    }

    /// Check if credentials are currently stored
    pub fn has_credentials(&self) -> bool {
        match self.credentials.lock() {
            Ok(creds) => creds.is_some(),
            Err(_) => false,
        }
    }

    /// Clear stored credentials
    pub fn clear_credentials(&self) -> Result<(), String> {
        match self.credentials.lock() {
            Ok(mut creds) => {
                *creds = None;
                Ok(())
            }
            Err(_) => Err("Failed to acquire credentials lock".to_string()),
        }
    }

    /// Get username if credentials exist
    pub fn get_username(&self) -> Option<String> {
        self.get_credentials().map(|creds| creds.username)
    }

    /// Validate credentials format (basic validation)
    pub fn validate_credentials(username: &str, password: &str) -> Result<(), String> {
        if username.trim().is_empty() {
            return Err("Username cannot be empty".to_string());
        }

        if password.is_empty() {
            return Err("Password cannot be empty".to_string());
        }

        if username.len() < 3 {
            return Err("Username must be at least 3 characters".to_string());
        }

        if password.len() < 3 {
            return Err("Password must be at least 3 characters".to_string());
        }

        Ok(())
    }
}
