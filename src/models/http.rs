use crate::models::http_client::{
    HttpHeader, HttpMethod, HttpQueryParam, HttpRequest, HttpRequestBody, HttpResponse,
};

/// Current tab in the request editor
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpRequestTab {
    Headers,
    Body,
    QueryParams,
    Auth,
    Settings,
}

impl HttpRequestTab {
    pub fn all() -> Vec<HttpRequestTab> {
        vec![
            HttpRequestTab::Headers,
            HttpRequestTab::Body,
            HttpRequestTab::QueryParams,
            HttpRequestTab::Auth,
            HttpRequestTab::Settings,
        ]
    }

    pub fn title(&self) -> &'static str {
        match self {
            HttpRequestTab::Headers => "Headers",
            HttpRequestTab::Body => "Body",
            HttpRequestTab::QueryParams => "Query",
            HttpRequestTab::Auth => "Auth",
            HttpRequestTab::Settings => "Settings",
        }
    }
}

/// Current tab in the response viewer
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpResponseTab {
    Body,
    Headers,
    Info,
}

impl HttpResponseTab {
    pub fn all() -> Vec<HttpResponseTab> {
        vec![
            HttpResponseTab::Body,
            HttpResponseTab::Headers,
            HttpResponseTab::Info,
        ]
    }

    pub fn title(&self) -> &'static str {
        match self {
            HttpResponseTab::Body => "Body",
            HttpResponseTab::Headers => "Headers",
            HttpResponseTab::Info => "Info",
        }
    }
}

/// Body content type for editing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BodyContentType {
    None,
    Json,
    Text,
    Form,
    Raw,
}

impl BodyContentType {
    pub fn all() -> Vec<BodyContentType> {
        vec![
            BodyContentType::None,
            BodyContentType::Json,
            BodyContentType::Text,
            BodyContentType::Form,
            BodyContentType::Raw,
        ]
    }

    pub fn title(&self) -> &'static str {
        match self {
            BodyContentType::None => "None",
            BodyContentType::Json => "JSON",
            BodyContentType::Text => "Text",
            BodyContentType::Form => "Form",
            BodyContentType::Raw => "Raw",
        }
    }
}

/// HTTP client state (similar to AutomationState)
#[derive(Debug, Clone)]
pub struct HttpState {
    /// Currently edited request
    pub current_request: HttpRequest,

    /// Current tab in request editor
    pub current_request_tab: HttpRequestTab,

    /// Current tab in response viewer
    pub current_response_tab: HttpResponseTab,

    /// Current body content type being edited
    pub current_body_type: BodyContentType,

    /// Currently focused field index (for navigation)
    pub focused_field: usize,

    /// Most recent response (if any)
    pub last_response: Option<HttpResponse>,

    /// Whether a request is currently being sent
    pub is_sending: bool,

    /// List of recent URLs for autocomplete
    pub recent_urls: Vec<String>,

    /// Environment variables for template substitution
    pub environment_vars: std::collections::HashMap<String, String>,
}

impl HttpState {
    pub fn new() -> Self {
        Self {
            current_request: HttpRequest::new("New Request"),
            current_request_tab: HttpRequestTab::Headers,
            current_response_tab: HttpResponseTab::Body,
            current_body_type: BodyContentType::None,
            focused_field: 0,
            last_response: None,
            is_sending: false,
            recent_urls: Vec::new(),
            environment_vars: std::collections::HashMap::new(),
        }
    }

    /// Switch to next request tab
    pub fn next_request_tab(&mut self) {
        let tabs = HttpRequestTab::all();
        let current_index = tabs
            .iter()
            .position(|t| t == &self.current_request_tab)
            .unwrap_or(0);
        self.current_request_tab = tabs[(current_index + 1) % tabs.len()].clone();
    }

    /// Switch to previous request tab
    pub fn prev_request_tab(&mut self) {
        let tabs = HttpRequestTab::all();
        let current_index = tabs
            .iter()
            .position(|t| t == &self.current_request_tab)
            .unwrap_or(0);
        let prev_index = if current_index == 0 {
            tabs.len() - 1
        } else {
            current_index - 1
        };
        self.current_request_tab = tabs[prev_index].clone();
    }

    /// Switch to next response tab
    pub fn next_response_tab(&mut self) {
        let tabs = HttpResponseTab::all();
        let current_index = tabs
            .iter()
            .position(|t| t == &self.current_response_tab)
            .unwrap_or(0);
        self.current_response_tab = tabs[(current_index + 1) % tabs.len()].clone();
    }

    /// Switch to previous response tab
    pub fn prev_response_tab(&mut self) {
        let tabs = HttpResponseTab::all();
        let current_index = tabs
            .iter()
            .position(|t| t == &self.current_response_tab)
            .unwrap_or(0);
        let prev_index = if current_index == 0 {
            tabs.len() - 1
        } else {
            current_index - 1
        };
        self.current_response_tab = tabs[prev_index].clone();
    }

    /// Add a header to the current request
    pub fn add_header(&mut self, name: String, value: String) {
        self.current_request
            .headers
            .push(HttpHeader::new(name, value));
    }

    /// Remove a header by index
    pub fn remove_header(&mut self, index: usize) {
        if index < self.current_request.headers.len() {
            self.current_request.headers.remove(index);
        }
    }

    /// Add a query parameter to the current request
    pub fn add_query_param(&mut self, name: String, value: String) {
        self.current_request
            .query_params
            .push(HttpQueryParam::new(name, value));
    }

    /// Remove a query parameter by index
    pub fn remove_query_param(&mut self, index: usize) {
        if index < self.current_request.query_params.len() {
            self.current_request.query_params.remove(index);
        }
    }

    /// Set request method
    pub fn set_method(&mut self, method: HttpMethod) {
        self.current_request.method = method;
    }

    /// Set request URL
    pub fn set_url(&mut self, url: String) {
        // Add to recent URLs if not already there
        if !url.is_empty() && !self.recent_urls.contains(&url) {
            self.recent_urls.insert(0, url.clone());
            // Keep only last 20 URLs
            self.recent_urls.truncate(20);
        }
        self.current_request.url = url;
    }

    /// Set request body
    pub fn set_body(&mut self, body: HttpRequestBody) {
        self.current_request.body = body;
    }

    /// Get body content as string for editing
    pub fn get_body_content(&self) -> String {
        match &self.current_request.body {
            HttpRequestBody::None => String::new(),
            HttpRequestBody::Text { content, .. } => content.clone(),
            HttpRequestBody::Json { content } => content.clone(),
            HttpRequestBody::Raw { content } => content.clone(),
            HttpRequestBody::Form { .. } => String::new(), // Form handled separately
        }
    }

    /// Update body content from editor
    pub fn update_body_content(&mut self, content: String) {
        match self.current_body_type {
            BodyContentType::None => {
                self.current_request.body = HttpRequestBody::None;
            }
            BodyContentType::Json => {
                self.current_request.body = HttpRequestBody::Json { content };
            }
            BodyContentType::Text => {
                self.current_request.body = HttpRequestBody::Text {
                    content,
                    content_type: "text/plain".to_string(),
                };
            }
            BodyContentType::Raw => {
                self.current_request.body = HttpRequestBody::Raw { content };
            }
            BodyContentType::Form => {
                // Form data handled separately through add_form_field
            }
        }
    }

    /// Check if request is valid for sending
    pub fn is_valid(&self) -> bool {
        !self.current_request.url.trim().is_empty()
    }

    /// Get validation errors
    pub fn get_validation_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.current_request.url.trim().is_empty() {
            errors.push("URL is required".to_string());
        }

        // Add URL format validation
        if !self.current_request.url.is_empty()
            && !self.current_request.url.starts_with("http://")
            && !self.current_request.url.starts_with("https://")
        {
            errors.push("URL must start with http:// or https://".to_string());
        }

        errors
    }

    /// Load a request into the editor
    pub fn load_request(&mut self, request: HttpRequest) {
        self.current_request = request;

        // Reset UI state
        self.current_request_tab = HttpRequestTab::Headers;
        self.current_response_tab = HttpResponseTab::Body;
        self.focused_field = 0;

        // Set appropriate body type
        self.current_body_type = match &self.current_request.body {
            HttpRequestBody::None => BodyContentType::None,
            HttpRequestBody::Json { .. } => BodyContentType::Json,
            HttpRequestBody::Text { .. } => BodyContentType::Text,
            HttpRequestBody::Form { .. } => BodyContentType::Form,
            HttpRequestBody::Raw { .. } => BodyContentType::Raw,
        };
    }

    /// Create a new empty request
    pub fn new_request(&mut self) {
        self.current_request = HttpRequest::new("New Request");
        self.current_request_tab = HttpRequestTab::Headers;
        self.current_response_tab = HttpResponseTab::Body;
        self.current_body_type = BodyContentType::None;
        self.focused_field = 0;
        self.last_response = None;
    }
}
