pub mod browser;
pub mod state;
pub mod templates;

// Re-export for convenience
pub use browser::BrowserEngine;
pub use state::{AutomationState, Credentials};
pub use templates::TemplateManager;
