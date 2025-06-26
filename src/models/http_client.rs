use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// HTTP request methods
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
        }
    }

    pub fn all() -> Vec<HttpMethod> {
        vec![
            HttpMethod::GET,
            HttpMethod::POST,
            HttpMethod::PUT,
            HttpMethod::PATCH,
            HttpMethod::DELETE,
            HttpMethod::HEAD,
            HttpMethod::OPTIONS,
        ]
    }
}

/// HTTP header key-value pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
    pub enabled: bool,
}

impl HttpHeader {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            enabled: true,
        }
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Query parameter key-value pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpQueryParam {
    pub name: String,
    pub value: String,
    pub enabled: bool,
}

impl HttpQueryParam {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            enabled: true,
        }
    }
}

/// Request body types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HttpRequestBody {
    None,
    Text {
        content: String,
        content_type: String,
    },
    Json {
        content: String,
    },
    Form {
        fields: Vec<HttpFormField>,
    },
    Raw {
        content: String,
    },
}

/// Form field for form-data requests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpFormField {
    pub name: String,
    pub value: String,
    pub enabled: bool,
}

impl HttpFormField {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            enabled: true,
        }
    }
}

/// Authentication types for HTTP requests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HttpAuth {
    None,
    Basic {
        username: String,
        password: String,
    },
    Bearer {
        token: String,
    },
    ApiKey {
        key: String,
        value: String,
        location: ApiKeyLocation,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApiKeyLocation {
    Header,
    QueryParam,
}

/// HTTP request model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub name: String,
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<HttpHeader>,
    pub query_params: Vec<HttpQueryParam>,
    pub body: HttpRequestBody,
    pub auth: HttpAuth,
    pub description: String,
}

impl HttpRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            method: HttpMethod::GET,
            url: String::new(),
            headers: Vec::new(),
            query_params: Vec::new(),
            body: HttpRequestBody::None,
            auth: HttpAuth::None,
            description: String::new(),
        }
    }

    pub fn with_method(mut self, method: HttpMethod) -> Self {
        self.method = method;
        self
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = url.into();
        self
    }

    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push(HttpHeader::new(name, value));
        self
    }
}

/// HTTP response model
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub status_text: String,
    pub headers: Vec<HttpHeader>,
    pub body: String,
    pub content_type: String,
    pub duration_ms: u64,
}

impl HttpResponse {
    pub fn is_success(&self) -> bool {
        self.status_code >= 200 && self.status_code < 300
    }

    pub fn is_error(&self) -> bool {
        self.status_code >= 400
    }

    pub fn status_color(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        match self.status_code {
            200..=299 => Color::Green,
            300..=399 => Color::Yellow,
            400..=499 => Color::Red,
            500..=599 => Color::Magenta,
            _ => Color::Gray,
        }
    }
}

/// HTTP collection (similar to template folders)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpCollection {
    pub name: String,
    pub requests: Vec<HttpRequest>,
    pub folders: Vec<HttpCollection>,
    pub description: String,
}

impl HttpCollection {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            requests: Vec::new(),
            folders: Vec::new(),
            description: String::new(),
        }
    }

    pub fn add_request(&mut self, request: HttpRequest) {
        self.requests.push(request);
    }

    pub fn add_folder(&mut self, folder: HttpCollection) {
        self.folders.push(folder);
    }
}
