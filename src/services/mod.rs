pub mod auth_service;
pub mod template_storage;

// Re-export for convenience
pub use auth_service::AuthService;
pub use template_storage::{StoredTemplate, TemplateStorage};
