use crate::app::AppMessage;
use crate::models::{FormField, LogLevel, WebsiteConfig};
use crate::modes::automation::Credentials;
use anyhow::{Context, Result};
use headless_chrome::{Browser, LaunchOptions};
use std::time::Duration;
use tokio::sync::mpsc;

/// Browser automation engine that runs in the background
pub struct BrowserEngine {
    message_sender: mpsc::UnboundedSender<AppMessage>,
}

impl BrowserEngine {
    pub fn new(message_sender: mpsc::UnboundedSender<AppMessage>) -> Self {
        Self { message_sender }
    }

    /// Run the browser automation in a background task
    pub async fn run_automation(
        &self,
        fields: Vec<FormField>,
        credentials: Credentials,
        website_config: WebsiteConfig,
    ) -> Result<()> {
        // Send initial progress update
        self.log_progress("Starting browser automation...").await;

        // Switch to real browser automation to test UI responsiveness
        // Comment out this line and uncomment the simple test if browser fails
        self.run_google_navigation_example().await?;

        // Uncomment this line if you want to test without browser first:
        // self.run_simple_test().await?;

        // TODO: Replace with your actual automation logic:
        // self.run_actual_automation(fields, credentials, website_config).await?;

        Ok(())
    }

    /// Simple test without browser to verify async system works
    async fn run_simple_test(&self) -> Result<()> {
        self.log_progress("Running simple async test...").await;

        for i in 1..=5 {
            self.log_progress(format!("Test step {}/5", i)).await;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        self.log_progress("Simple test completed successfully!")
            .await;
        Ok(())
    }

    /// Simple example: Navigate to Google to test browser automation
    /// Replace this with your actual automation logic
    async fn run_google_navigation_example(&self) -> Result<()> {
        self.log_progress("Launching headless Chrome browser...")
            .await;

        // Launch the browser with headless options
        let browser = Browser::new(LaunchOptions {
            headless: true,
            sandbox: false,
            window_size: Some((1920, 1080)),
            ..Default::default()
        })
        .context("Failed to launch browser")?;

        self.log_progress("Browser launched successfully").await;

        // Create a new tab
        let tab = browser.new_tab().context("Failed to create new tab")?;

        self.log_progress("Created new browser tab").await;

        // Navigate to Google (replace with your target URL)
        self.log_progress("Navigating to google.com...").await;
        tab.navigate_to("https://www.google.com")
            .context("Failed to navigate to Google")?;

        // Wait for the page to load
        tab.wait_until_navigated()
            .context("Failed to wait for navigation")?;

        self.log_progress("Successfully navigated to Google").await;

        // Get the page title to verify we're there
        let title = tab.get_title().context("Failed to get page title")?;

        self.log_progress(format!("Page title: {}", title)).await;

        // Simulate some work time
        self.log_progress("Simulating form interaction...").await;
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Try to find the search box (just as an example)
        match tab.find_element("input[name='q']") {
            Ok(_) => {
                self.log_progress("Found Google search box").await;
                // Could type something here if you want
                // element.type_into("Hello from Rust automation!")?;
            }
            Err(_) => {
                self.log_progress("Could not find search box (page might have changed)")
                    .await;
            }
        }

        self.log_progress("Browser automation completed successfully")
            .await;

        // Browser will be automatically closed when it goes out of scope
        Ok(())
    }

    /// Your actual automation logic will go here
    /// This is where you'll implement the real form filling
    #[allow(dead_code)]
    async fn run_actual_automation(
        &self,
        fields: Vec<FormField>,
        credentials: Credentials,
        website_config: WebsiteConfig,
    ) -> Result<()> {
        self.log_progress("Starting actual website automation...")
            .await;

        // Launch browser
        let browser = Browser::new(LaunchOptions {
            headless: true,
            sandbox: false,
            window_size: Some((1920, 1080)),
            ..Default::default()
        })
        .context("Failed to launch browser")?;

        let tab = browser.new_tab().context("Failed to create new tab")?;

        // Navigate to login page
        self.log_progress(format!("Navigating to login: {}", website_config.login_url))
            .await;
        tab.navigate_to(&website_config.login_url)
            .context("Failed to navigate to login page")?;
        tab.wait_until_navigated()
            .context("Failed to wait for login page")?;

        // Fill login credentials
        self.log_progress("Filling login credentials...").await;

        // Find and fill username
        let username_input = tab
            .find_element(&website_config.username_selector)
            .context("Failed to find username field")?;
        username_input
            .type_into(&credentials.username)
            .context("Failed to type username")?;

        // Find and fill password
        let password_input = tab
            .find_element(&website_config.password_selector)
            .context("Failed to find password field")?;
        password_input
            .type_into(&credentials.password)
            .context("Failed to type password")?;

        // Submit login form
        self.log_progress("Submitting login form...").await;
        let submit_button = tab
            .find_element(&website_config.submit_selector)
            .context("Failed to find submit button")?;
        submit_button.click().context("Failed to click submit")?;

        // Wait for login to complete
        tokio::time::sleep(Duration::from_secs(3)).await;

        // Navigate to the form page
        self.log_progress(format!("Navigating to form: {}", website_config.form_url))
            .await;
        tab.navigate_to(&website_config.form_url)
            .context("Failed to navigate to form page")?;
        tab.wait_until_navigated()
            .context("Failed to wait for form page")?;

        // Fill each form field
        for field in fields {
            if !field.value.trim().is_empty() {
                self.log_progress(format!("Filling field: {}", field.name))
                    .await;

                match tab.find_element(&field.selector) {
                    Ok(element) => {
                        element
                            .type_into(&field.value)
                            .context(format!("Failed to type into field: {}", field.name))?;

                        self.log_progress(format!("✓ Filled {}: {}", field.name, field.value))
                            .await;
                    }
                    Err(_) => {
                        self.log_progress(format!("⚠️  Could not find field: {}", field.name))
                            .await;
                    }
                }

                // Small delay between fields
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }

        self.log_progress("All fields filled successfully").await;

        // Here you could submit the form or perform other actions
        // tab.find_element("input[type='submit']")?.click()?;

        Ok(())
    }

    /// Send a progress update to the UI
    async fn log_progress(&self, message: impl Into<String>) {
        let _ = self
            .message_sender
            .send(AppMessage::AutomationProgress(message.into()));
    }

    /// Send an error message to the UI
    #[allow(dead_code)]
    async fn log_error(&self, message: impl Into<String>) {
        let _ = self
            .message_sender
            .send(AppMessage::Log(LogLevel::Error, message.into()));
    }

    /// Send completion signal to the UI
    pub async fn send_completion(&self) {
        let _ = self.message_sender.send(AppMessage::AutomationComplete);
    }

    /// Send failure signal to the UI
    pub async fn send_failure(&self, error: impl Into<String>) {
        let _ = self
            .message_sender
            .send(AppMessage::AutomationFailed(error.into()));
    }
}
