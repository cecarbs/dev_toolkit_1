# ===== scripts/automation_script.py (Main Entry Point) =====
#!/usr/bin/env python3
"""
Main automation script - entry point for Rust TUI
This is the file Rust will execute
"""

import sys
import json
import argparse
from browser_automation.automation_runner import AutomationRunner
from utils.tui_logger import TUILogger


def main():
    parser = argparse.ArgumentParser(description="Browser Automation Script")
    parser.add_argument(
        "--json-input", action="store_true", help="Read JSON automation data from stdin"
    )

    args = parser.parse_args()

    logger = TUILogger()

    try:
        if args.json_input:
            # Read automation data from Rust
            input_data = json.loads(sys.stdin.read())
            logger.debug("Received automation data from Rust TUI")
        else:
            logger.error("This script requires --json-input flag")
            sys.exit(1)

        # Create and run automation
        automation = AutomationRunner(input_data, logger)
        success = automation.run()

        if success:
            logger.success("🎉 Automation completed successfully!")
            sys.exit(0)
        else:
            sys.exit(1)

    except json.JSONDecodeError as e:
        logger.error(f"Invalid JSON input: {e}")
        sys.exit(1)
    except Exception as e:
        logger.error(f"Unexpected error: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()

# ===== scripts/utils/tui_logger.py =====
"""
Logging utilities that send formatted messages back to Rust TUI
"""


class TUILogger:
    """Logger that sends messages back to Rust TUI logging panel"""

    def progress(self, msg):
        """Show progress updates in TUI"""
        print(f"PROGRESS: {msg}", flush=True)

    def success(self, msg):
        """Show success messages in green"""
        print(f"SUCCESS: {msg}", flush=True)

    def error(self, msg):
        """Show error messages in red"""
        print(f"ERROR: {msg}", flush=True)

    def info(self, msg):
        """Show info messages in white"""
        print(f"INFO: {msg}", flush=True)

    def debug(self, msg):
        """Show debug messages in gray"""
        print(f"DEBUG: {msg}", flush=True)

    def warn(self, msg):
        """Show warning messages in yellow"""
        print(f"WARN: {msg}", flush=True)


# ===== scripts/browser_automation/automation_runner.py =====
"""
Main automation runner that orchestrates the entire process
"""

from .chrome_driver import ChromeDriver
from .form_filler import FormFiller
from config.website_config import WebsiteConfig


class AutomationRunner:
    def __init__(self, automation_data, logger):
        self.logger = logger
        self.fields = automation_data["fields"]
        self.credentials = automation_data["credentials"]
        self.config = WebsiteConfig(automation_data["website_config"])
        self.driver = None
        self.form_filler = None

    def run(self):
        """Run the complete automation process"""
        try:
            self.logger.info(f"Starting automation for {len(self.fields)} fields")

            # Initialize browser
            self.driver = ChromeDriver(self.logger)
            self.form_filler = FormFiller(self.driver, self.logger)

            # Step 1: Login
            self._perform_login()

            # Step 2: Navigate to form
            self._navigate_to_form()

            # Step 3: Fill form
            filled_count = self._fill_form_fields()

            # Step 4: Submit form
            self._submit_form()

            self.logger.success(f"Automation completed! Filled {filled_count} fields.")
            return True

        except Exception as e:
            self.logger.error(f"Automation failed: {e}")
            return False
        finally:
            if self.driver:
                self.driver.quit()

    def _perform_login(self):
        """Handle the login process"""
        self.logger.progress("Starting login process...")

        username = self.credentials["username"]
        password = self.credentials["password"]

        self.driver.navigate_to(self.config.login_url)
        self.driver.login(username, password, self.config)

        self.logger.success(f"Successfully logged in as {username}")

    def _navigate_to_form(self):
        """Navigate to the target form"""
        self.logger.progress("Navigating to form page...")
        self.driver.navigate_to(self.config.form_url)
        self.logger.info("Reached form page")

    def _fill_form_fields(self):
        """Fill all form fields with user data"""
        self.logger.progress(f"Filling {len(self.fields)} form fields...")
        return self.form_filler.fill_all_fields(self.fields)

    def _submit_form(self):
        """Submit the completed form"""
        self.logger.progress("Submitting form...")
        self.form_filler.submit_form(self.config.submit_selector)


# ===== scripts/browser_automation/chrome_driver.py =====
"""
Chrome WebDriver management and navigation
"""

from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
from selenium.webdriver.chrome.options import Options
from webdriver_manager.chrome import ChromeDriverManager
from selenium.webdriver.chrome.service import Service


class ChromeDriver:
    def __init__(self, logger):
        self.logger = logger
        self.driver = None
        self.wait = None
        self._start_browser()

    def _start_browser(self):
        """Initialize Chrome WebDriver"""
        self.logger.progress("Starting Chrome browser...")

        # Configure Chrome options
        options = Options()
        options.add_argument("--headless")  # Run in background
        options.add_argument("--no-sandbox")
        options.add_argument("--disable-dev-shm-usage")
        options.add_argument("--disable-gpu")
        options.add_argument("--window-size=1920,1080")

        # Use webdriver-manager to handle ChromeDriver installation
        service = Service(ChromeDriverManager().install())

        self.driver = webdriver.Chrome(service=service, options=options)
        self.wait = WebDriverWait(self.driver, 10)

        self.logger.success("Chrome browser started successfully")

    def navigate_to(self, url):
        """Navigate to a specific URL"""
        self.logger.debug(f"Navigating to: {url}")
        self.driver.get(url)

    def login(self, username, password, config):
        """Perform login using the provided credentials"""
        self.logger.progress("Filling login credentials...")

        # Wait for and fill username
        username_field = self.wait.until(
            EC.presence_of_element_located((By.CSS_SELECTOR, config.username_selector))
        )
        username_field.clear()
        username_field.send_keys(username)

        # Fill password
        password_field = self.driver.find_element(
            By.CSS_SELECTOR, config.password_selector
        )
        password_field.clear()
        password_field.send_keys(password)

        # Click submit
        submit_btn = self.driver.find_element(By.CSS_SELECTOR, config.submit_selector)
        submit_btn.click()

        # Wait for login to complete (you might need to adjust this)
        self.wait.until(EC.url_changes(self.driver.current_url))

    def find_element_safe(self, selector):
        """Safely find an element with error handling"""
        try:
            return self.wait.until(
                EC.presence_of_element_located((By.CSS_SELECTOR, selector))
            )
        except Exception as e:
            self.logger.warn(f"Could not find element with selector '{selector}': {e}")
            return None

    def quit(self):
        """Clean up and close the browser"""
        if self.driver:
            self.logger.debug("Closing browser...")
            self.driver.quit()


# ===== scripts/browser_automation/form_filler.py =====
"""
Form filling logic and field handling
"""

from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import Select
from selenium.common.exceptions import NoSuchElementException, TimeoutException


class FormFiller:
    def __init__(self, chrome_driver, logger):
        self.driver = chrome_driver.driver
        self.chrome_driver = chrome_driver
        self.logger = logger

    def fill_all_fields(self, fields):
        """Fill all provided form fields"""
        filled_count = 0

        for field in fields:
            if field["value"] and field["value"].strip():
                if self._fill_single_field(field):
                    filled_count += 1
            elif field["is_required"]:
                self.logger.warn(f"Required field '{field['name']}' is empty")

        self.logger.info(f"Successfully filled {filled_count}/{len(fields)} fields")
        return filled_count

    def _fill_single_field(self, field):
        """Fill a single form field"""
        field_name = field["name"]
        field_selector = field["selector"]
        field_value = field["value"]
        field_type = field["field_type"]

        try:
            self.logger.debug(f"Filling '{field_name}' with '{field_value}'")

            element = self.chrome_driver.find_element_safe(field_selector)
            if not element:
                return False

            # Handle different field types
            if field_type == "Select":
                self._fill_select_field(element, field_value, field_name)
            elif field_type == "Textarea":
                self._fill_textarea_field(element, field_value, field_name)
            else:
                self._fill_text_field(element, field_value, field_name)

            self.logger.success(f"✓ Filled '{field_name}'")
            return True

        except Exception as e:
            self.logger.error(f"Failed to fill '{field_name}': {e}")
            return False

    def _fill_select_field(self, element, value, field_name):
        """Handle dropdown/select fields"""
        select = Select(element)
        try:
            select.select_by_visible_text(value)
        except NoSuchElementException:
            # Try selecting by value if visible text doesn't work
            select.select_by_value(value)

    def _fill_textarea_field(self, element, value, field_name):
        """Handle textarea fields"""
        element.clear()
        element.send_keys(value)

    def _fill_text_field(self, element, value, field_name):
        """Handle regular text input fields"""
        element.clear()
        element.send_keys(value)

    def submit_form(self, submit_selector):
        """Submit the form using the provided selector"""
        try:
            submit_btn = self.chrome_driver.find_element_safe(submit_selector)
            if submit_btn:
                submit_btn.click()
                self.logger.success("Form submitted successfully!")
            else:
                self.logger.error(
                    f"Could not find submit button with selector: {submit_selector}"
                )
        except Exception as e:
            self.logger.error(f"Failed to submit form: {e}")


# ===== scripts/config/website_config.py =====
"""
Website configuration management
"""


class WebsiteConfig:
    def __init__(self, config_data):
        self.name = config_data["name"]
        self.url = config_data["url"]
        self.login_url = config_data["login_url"]
        self.form_url = config_data["form_url"]
        self.username_selector = config_data["username_selector"]
        self.password_selector = config_data["password_selector"]
        self.submit_selector = config_data["submit_selector"]

    def __str__(self):
        return f"WebsiteConfig(name='{self.name}', login_url='{self.login_url}')"
