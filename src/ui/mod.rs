pub mod components;
pub mod layout;

// Re-export main functions for convenience
pub use layout::{calculate_layout, get_theme_colors, render_app};
