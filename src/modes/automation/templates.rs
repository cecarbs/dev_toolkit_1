use crate::models::AutomationTemplate;
use anyhow::Result;

/// Template manager for handling template operations
pub struct TemplateManager {
    templates: Vec<AutomationTemplate>,
}

impl TemplateManager {
    pub fn new() -> Self {
        Self {
            templates: Vec::new(),
        }
    }

    /// Load templates from hardcoded data (for now)
    /// In the future, this could load from JSON files
    pub fn load_templates(&mut self) -> Result<()> {
        self.templates = vec![
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
                .with_field("Contact Email", "user@company.com"),
        ];

        Ok(())
    }

    /// Get all available templates
    pub fn get_templates(&self) -> &[AutomationTemplate] {
        &self.templates
    }

    /// Get a specific template by index
    pub fn get_template(&self, index: usize) -> Option<&AutomationTemplate> {
        self.templates.get(index)
    }

    /// Add a new template (for future save functionality)
    pub fn add_template(&mut self, template: AutomationTemplate) {
        self.templates.push(template);
    }

    /// Remove a template by index
    pub fn remove_template(&mut self, index: usize) -> Option<AutomationTemplate> {
        if index < self.templates.len() {
            Some(self.templates.remove(index))
        } else {
            None
        }
    }

    /// Future: Save templates to JSON file
    #[allow(dead_code)]
    pub fn save_templates(&self) -> Result<()> {
        // TODO: Implement JSON file saving
        // let json = serde_json::to_string_pretty(&self.templates)?;
        // std::fs::write("templates.json", json)?;
        Ok(())
    }

    /// Future: Load templates from JSON file
    #[allow(dead_code)]
    pub fn load_from_file(&mut self, _path: &str) -> Result<()> {
        // TODO: Implement JSON file loading
        // let content = std::fs::read_to_string(path)?;
        // self.templates = serde_json::from_str(&content)?;
        Ok(())
    }
}
