pub mod automation_form;
pub mod logging_panel;

// Re-export components for easier imports
pub use automation_form::{render_automation_form, render_login_popup};
pub use logging_panel::{render_log_stats, render_log_summary, render_logging_panel};
