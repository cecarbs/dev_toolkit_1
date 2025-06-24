use crate::models::{AutomationTemplate, FieldType, FormField, WebsiteConfig};
use std::collections::HashMap;

/// Current state of the automation mode
#[derive(Debug, Clone)]
pub struct AutomationState {
    /// The form fields for the website
    pub fields: Vec<FormField>,

    /// Available templates
    pub templates: Vec<AutomationTemplate>,

    /// Currently selected template index (None if no template selected)
    pub selected_template: Option<usize>,

    /// Currently focused field index for UI navigation
    pub focused_field: usize,

    /// Whether we're currently running automation
    pub is_running: bool,

    /// Website configuration
    pub website_config: WebsiteConfig,
}

#[derive(Debug, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl AutomationState {
    pub fn new() -> Self {
        Self {
            fields: Self::create_default_fields(),
            templates: Self::create_default_templates(),
            selected_template: None,
            focused_field: 0,
            is_running: false,
            website_config: WebsiteConfig::default(),
        }
    }

    /// Create form fields with examples of text inputs and dropdowns
    fn create_default_fields() -> Vec<FormField> {
        vec![
            // Text input example
            FormField::new("Project Name", "#project_name", FieldType::Text).with_required(true),
            // Dropdown example
            FormField::new("Department", "#department", FieldType::Select)
                .with_required(true)
                .with_dropdown_options(vec![
                    "Engineering".to_string(),
                    "Marketing".to_string(),
                    "Sales".to_string(),
                    "Support".to_string(),
                    "HR".to_string(),
                ]),
            // Another dropdown example
            FormField::new("Priority Level", "#priority", FieldType::Select)
                .with_required(true)
                .with_dropdown_options(vec![
                    "High".to_string(),
                    "Medium".to_string(),
                    "Low".to_string(),
                ]),
            // Textarea example
            FormField::new("Description", "#description", FieldType::Textarea).with_required(true),
            // Optional text input example
            FormField::new("Contact Email", "#contact_email", FieldType::Email)
                .with_required(false), // This will show as "(optional)"
        ]
    }

    /// Create some default templates with predefined values
    fn create_default_templates() -> Vec<AutomationTemplate> {
        vec![
            AutomationTemplate::new("Quick Task", "Standard task template")
                .with_field("Project Name", "Daily Task")
                .with_field("Department", "Engineering")
                .with_field("Priority Level", "Medium")
                .with_field("Description", "Standard daily task submission")
                .with_field("Contact Email", "user@company.com"),
            AutomationTemplate::new("Urgent Request", "High priority request template")
                .with_field("Project Name", "Urgent Fix")
                .with_field("Department", "Engineering")
                .with_field("Priority Level", "High")
                .with_field("Description", "Urgent issue that needs immediate attention")
                .with_field("Contact Email", "user@company.com"),
            AutomationTemplate::new("Weekly Report", "Weekly status report template")
                .with_field("Project Name", "Weekly Status")
                .with_field("Department", "Engineering")
                .with_field("Priority Level", "Low")
                .with_field("Description", "Weekly progress report and status update")
                .with_field("Contact Email", ""),
        ]
    }

    /// Apply the currently selected template to the form fields
    pub fn apply_selected_template(&mut self) {
        if let Some(template_index) = self.selected_template {
            if let Some(template) = self.templates.get(template_index) {
                template.apply_to_fields(&mut self.fields);
            }
        }
    }

    /// Get the currently selected template
    pub fn get_selected_template(&self) -> Option<&AutomationTemplate> {
        self.selected_template.and_then(|i| self.templates.get(i))
    }

    /// Move focus to the next field (for keyboard navigation)
    pub fn focus_next_field(&mut self) {
        if !self.fields.is_empty() {
            self.focused_field = (self.focused_field + 1) % self.fields.len();
        }
    }

    /// Move focus to the previous field (for keyboard navigation)
    pub fn focus_prev_field(&mut self) {
        if !self.fields.is_empty() {
            if self.focused_field == 0 {
                self.focused_field = self.fields.len() - 1;
            } else {
                self.focused_field -= 1;
            }
        }
    }

    /// Get the currently focused field
    pub fn get_focused_field(&self) -> Option<&FormField> {
        self.fields.get(self.focused_field)
    }

    /// Get the currently focused field mutably
    pub fn get_focused_field_mut(&mut self) -> Option<&mut FormField> {
        self.fields.get_mut(self.focused_field)
    }

    /// Update the value of the currently focused field
    pub fn update_focused_field_value(&mut self, value: String) {
        if let Some(field) = self.get_focused_field_mut() {
            field.value = value;
        }
    }

    /// Check if all required fields have values
    pub fn is_valid(&self) -> bool {
        self.fields.iter().all(|field| field.is_valid())
    }

    /// Set the running state
    pub fn set_running(&mut self, running: bool) {
        self.is_running = running;
    }

    /// Get validation errors for display
    pub fn get_validation_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();

        for field in &self.fields {
            if !field.is_valid() {
                errors.push(format!("'{}' is required", field.name));
            }
        }

        errors
    }
}
