pub mod auth_service;
pub mod http_collection_storage;
pub mod template_storage;

// Re-export for convenience
pub use auth_service::AuthService;
pub use http_collection_storage::{HttpCollectionStorage, PostmanCollection, StoredHttpRequest};
pub use template_storage::{StoredTemplate, TemplateStorage}; // NEW
