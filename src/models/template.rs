use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a form field with its selector and current value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormField {
    pub name: String,
    pub selector: String, // CSS selector for the field
    pub value: String,
    pub field_type: FieldType,
}

/// Different types of form fields we can handle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    Text,
    Email,
    Number,
    Select,
    Textarea,
}

impl FormField {
    pub fn new(
        name: impl Into<String>,
        selector: impl Into<String>,
        field_type: FieldType,
    ) -> Self {
        Self {
            name: name.into(),
            selector: selector.into(),
            value: String::new(),
            field_type,
        }
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }
}

/// A template contains predefined values for the form fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationTemplate {
    pub name: String,
    pub description: String,
    pub field_values: HashMap<String, String>, // field_name -> value
}

impl AutomationTemplate {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            field_values: HashMap::new(),
        }
    }

    pub fn with_field(mut self, field_name: impl Into<String>, value: impl Into<String>) -> Self {
        self.field_values.insert(field_name.into(), value.into());
        self
    }

    /// Apply this template's values to a collection of form fields
    pub fn apply_to_fields(&self, fields: &mut [FormField]) {
        for field in fields.iter_mut() {
            if let Some(template_value) = self.field_values.get(&field.name) {
                field.value = template_value.clone();
            }
        }
    }
}

/// Configuration for the target website and form
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebsiteConfig {
    pub name: String,
    pub url: String,
    pub login_url: String,
    pub form_url: String,
    pub username_selector: String,
    pub password_selector: String,
    pub submit_selector: String,
}

impl WebsiteConfig {
    /// Hardcoded config for now - you can replace with your actual website details
    pub fn default() -> Self {
        Self {
            name: "Company Portal".to_string(),
            url: "https://yourcompany.com".to_string(),
            login_url: "https://yourcompany.com/login".to_string(),
            form_url: "https://yourcompany.com/form".to_string(),
            username_selector: "#username".to_string(),
            password_selector: "#password".to_string(),
            submit_selector: "#submit".to_string(),
        }
    }
}
