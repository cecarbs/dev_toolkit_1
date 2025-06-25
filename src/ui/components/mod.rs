pub mod automation_form;
pub mod collections_tree;
pub mod delete_confirmation_dialog;
pub mod folder_dialog;
pub mod logging_panel;
pub mod rename_dialog;
pub mod status_line;
pub mod template_dialog;

// Re-export components for easier imports
pub use automation_form::{render_automation_form, render_login_popup};
pub use collections_tree::{get_tree_help_text, render_collections_tree};
pub use delete_confirmation_dialog::render_delete_confirmation_dialog;
pub use folder_dialog::render_folder_creation_dialog;
pub use logging_panel::{render_log_stats, render_log_summary, render_logging_panel};
pub use rename_dialog::render_rename_dialog;
pub use status_line::{get_mode_indicator, render_status_line};
pub use template_dialog::render_template_creation_dialog;
