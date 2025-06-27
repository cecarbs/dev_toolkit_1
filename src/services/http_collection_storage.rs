use crate::models::{AppConfig, http_client::*};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Postman Collection v2.1 format structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanCollection {
    pub info: PostmanInfo,
    pub item: Vec<PostmanItem>,
    pub auth: Option<PostmanAuth>,
    pub event: Option<Vec<PostmanEvent>>,
    pub variable: Option<Vec<PostmanVariable>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanInfo {
    #[serde(rename = "_postman_id")]
    pub postman_id: String,
    pub name: String,
    pub description: Option<String>,
    pub schema: String, // "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PostmanItem {
    Request(PostmanRequest),
    Folder(PostmanFolder),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanRequest {
    pub name: String,
    pub request: PostmanRequestDetails,
    pub response: Option<Vec<PostmanResponse>>,
    pub event: Option<Vec<PostmanEvent>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanFolder {
    pub name: String,
    pub item: Vec<PostmanItem>,
    pub description: Option<String>,
    pub auth: Option<PostmanAuth>,
    pub event: Option<Vec<PostmanEvent>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanRequestDetails {
    pub method: String,
    pub header: Option<Vec<PostmanHeader>>,
    pub body: Option<PostmanBody>,
    pub url: PostmanUrl,
    pub auth: Option<PostmanAuth>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PostmanUrl {
    String(String),
    Object(PostmanUrlObject),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanUrlObject {
    pub raw: String,
    pub protocol: Option<String>,
    pub host: Option<Vec<String>>,
    pub port: Option<String>,
    pub path: Option<Vec<String>>,
    pub query: Option<Vec<PostmanQueryParam>>,
    pub hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanHeader {
    pub key: String,
    pub value: String,
    pub disabled: Option<bool>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanQueryParam {
    pub key: String,
    pub value: String,
    pub disabled: Option<bool>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanBody {
    pub mode: String, // "raw", "formdata", "urlencoded", etc.
    pub raw: Option<String>,
    pub formdata: Option<Vec<PostmanFormData>>,
    pub urlencoded: Option<Vec<PostmanUrlEncoded>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanFormData {
    pub key: String,
    pub value: String,
    pub disabled: Option<bool>,
    #[serde(rename = "type")]
    pub field_type: Option<String>, // "text", "file"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanUrlEncoded {
    pub key: String,
    pub value: String,
    pub disabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanAuth {
    #[serde(rename = "type")]
    pub auth_type: String, // "basic", "bearer", "apikey", etc.
    pub basic: Option<Vec<PostmanAuthItem>>,
    pub bearer: Option<Vec<PostmanAuthItem>>,
    pub apikey: Option<Vec<PostmanAuthItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanAuthItem {
    pub key: String,
    pub value: String,
    #[serde(rename = "type")]
    pub value_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanResponse {
    pub name: String,
    pub original_request: PostmanRequestDetails,
    pub status: String,
    pub code: u16,
    pub header: Option<Vec<PostmanHeader>>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanEvent {
    pub listen: String, // "prerequest", "test"
    pub script: PostmanScript,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanScript {
    #[serde(rename = "type")]
    pub script_type: String, // "text/javascript"
    pub exec: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmanVariable {
    pub key: String,
    pub value: String,
    #[serde(rename = "type")]
    pub var_type: Option<String>,
}

/// Stored HTTP request with metadata (similar to StoredTemplate)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredHttpRequest {
    /// The HTTP request data
    pub request: HttpRequest,

    /// When the request was created
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// When the request was last modified
    pub modified_at: chrono::DateTime<chrono::Utc>,

    /// When the request was last used
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Tags for organization and search
    pub tags: Vec<String>,

    /// File format version for future compatibility
    pub version: String,
}

impl StoredHttpRequest {
    pub fn new(request: HttpRequest) -> Self {
        let now = chrono::Utc::now();
        Self {
            request,
            created_at: now,
            modified_at: now,
            last_used_at: None,
            tags: Vec::new(),
            version: "1.0".to_string(),
        }
    }

    pub fn mark_as_used(&mut self) {
        self.last_used_at = Some(chrono::Utc::now());
        self.modified_at = chrono::Utc::now();
    }
}

/// HTTP Collection storage service
pub struct HttpCollectionStorage {
    config: AppConfig,
}

impl HttpCollectionStorage {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// Initialize the storage (create directories, demo collections, etc.)
    pub fn initialize(&self) -> Result<()> {
        self.ensure_collections_directory()?;
        self.create_demo_collections_if_needed()?;
        Ok(())
    }

    /// Ensure the HTTP collections directory exists
    fn ensure_collections_directory(&self) -> Result<()> {
        let collections_dir = self.get_collections_directory();
        std::fs::create_dir_all(&collections_dir)
            .context("Failed to create HTTP collections directory")?;
        Ok(())
    }

    /// Get the HTTP collections directory
    pub fn get_collections_directory(&self) -> PathBuf {
        self.config
            .get_templates_directory()
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("http-collections")
    }

    /// Save an HTTP request to disk
    pub fn save_request(
        &self,
        folder_path: &str,
        request_name: &str,
        request: HttpRequest,
    ) -> Result<PathBuf> {
        let stored_request = StoredHttpRequest::new(request);

        // Build the full path
        let collections_dir = self.get_collections_directory();
        let folder_dir = collections_dir.join(folder_path);

        // Create the folder if it doesn't exist
        std::fs::create_dir_all(&folder_dir).context("Failed to create collection folder")?;

        // Create the filename (sanitize the name)
        let filename = sanitize_filename(request_name) + ".json";
        let file_path = folder_dir.join(filename);

        // Serialize and save
        let json_content = serde_json::to_string_pretty(&stored_request)
            .context("Failed to serialize HTTP request")?;

        std::fs::write(&file_path, json_content).context("Failed to write request file")?;

        Ok(file_path)
    }

    /// Load a specific HTTP request from disk
    pub fn load_request(&self, folder_path: &str, request_name: &str) -> Result<StoredHttpRequest> {
        let collections_dir = self.get_collections_directory();
        let filename = sanitize_filename(request_name) + ".json";
        let file_path = collections_dir.join(folder_path).join(filename);

        let json_content =
            std::fs::read_to_string(&file_path).context("Failed to read request file")?;

        let mut stored_request: StoredHttpRequest =
            serde_json::from_str(&json_content).context("Failed to parse request file")?;

        // Mark as used
        stored_request.mark_as_used();

        // Save the updated usage info
        let json_content = serde_json::to_string_pretty(&stored_request)
            .context("Failed to serialize updated request")?;
        std::fs::write(&file_path, json_content).context("Failed to update request file")?;

        Ok(stored_request)
    }

    /// Delete an HTTP request from disk
    pub fn delete_request(&self, folder_path: &str, request_name: &str) -> Result<()> {
        let collections_dir = self.get_collections_directory();
        let filename = sanitize_filename(request_name) + ".json";
        let file_path = collections_dir.join(folder_path).join(filename);

        std::fs::remove_file(&file_path).context("Failed to delete request file")?;
        Ok(())
    }

    /// Get all HTTP requests in a folder
    pub fn list_requests_in_folder(&self, folder_path: &str) -> Result<Vec<String>> {
        let collections_dir = self.get_collections_directory();
        let folder_dir = collections_dir.join(folder_path);

        if !folder_dir.exists() {
            return Ok(Vec::new());
        }

        let mut requests = Vec::new();

        for entry in std::fs::read_dir(&folder_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    requests.push(stem.to_string());
                }
            }
        }

        requests.sort();
        Ok(requests)
    }

    /// Get all folders in the collections directory
    pub fn list_all_folders(&self) -> Result<Vec<String>> {
        let collections_dir = self.get_collections_directory();
        let mut folders = Vec::new();

        self.scan_folders_recursive(&collections_dir, "", &mut folders)?;

        folders.sort();
        Ok(folders)
    }

    /// Recursively scan for folders
    fn scan_folders_recursive(
        &self,
        dir: &Path,
        current_path: &str,
        folders: &mut Vec<String>,
    ) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let folder_name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown");

                let full_path = if current_path.is_empty() {
                    folder_name.to_string()
                } else {
                    format!("{}/{}", current_path, folder_name)
                };

                folders.push(full_path.clone());

                // Recursively scan subfolders
                self.scan_folders_recursive(&path, &full_path, folders)?;
            }
        }

        Ok(())
    }

    /// Import a Postman collection from file
    pub fn import_postman_collection(&self, file_path: &Path) -> Result<()> {
        let json_content =
            std::fs::read_to_string(file_path).context("Failed to read Postman collection file")?;

        let collection: PostmanCollection =
            serde_json::from_str(&json_content).context("Failed to parse Postman collection")?;

        self.process_postman_collection(&collection, "")?;

        Ok(())
    }

    /// Process a Postman collection and save items
    fn process_postman_collection(
        &self,
        collection: &PostmanCollection,
        base_path: &str,
    ) -> Result<()> {
        let collection_path = if base_path.is_empty() {
            sanitize_filename(&collection.info.name)
        } else {
            format!("{}/{}", base_path, sanitize_filename(&collection.info.name))
        };

        // Create collection folder
        let collections_dir = self.get_collections_directory();
        let folder_path = collections_dir.join(&collection_path);
        std::fs::create_dir_all(&folder_path)?;

        // Process all items in the collection
        for item in &collection.item {
            self.process_postman_item(item, &collection_path)?;
        }

        Ok(())
    }

    /// Process a Postman item (request or folder)
    fn process_postman_item(&self, item: &PostmanItem, current_path: &str) -> Result<()> {
        match item {
            PostmanItem::Request(postman_request) => {
                // Convert Postman request to our HttpRequest format
                let http_request = self.convert_postman_request(postman_request)?;
                self.save_request(current_path, &postman_request.name, http_request)?;
            }
            PostmanItem::Folder(postman_folder) => {
                let folder_path = format!(
                    "{}/{}",
                    current_path,
                    sanitize_filename(&postman_folder.name)
                );

                // Create subfolder
                let collections_dir = self.get_collections_directory();
                let full_folder_path = collections_dir.join(&folder_path);
                std::fs::create_dir_all(&full_folder_path)?;

                // Process items in the subfolder
                for sub_item in &postman_folder.item {
                    self.process_postman_item(sub_item, &folder_path)?;
                }
            }
        }
        Ok(())
    }

    /// Convert Postman request to our HttpRequest format
    fn convert_postman_request(&self, postman_request: &PostmanRequest) -> Result<HttpRequest> {
        let details = &postman_request.request;

        // Convert method
        let method = match details.method.to_uppercase().as_str() {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "PATCH" => HttpMethod::PATCH,
            "DELETE" => HttpMethod::DELETE,
            "HEAD" => HttpMethod::HEAD,
            "OPTIONS" => HttpMethod::OPTIONS,
            _ => HttpMethod::GET, // Default fallback
        };

        // Extract URL
        let url = match &details.url {
            PostmanUrl::String(url_str) => url_str.clone(),
            PostmanUrl::Object(url_obj) => url_obj.raw.clone(),
        };

        // Convert headers
        let headers = details.header.as_ref().map_or_else(Vec::new, |headers| {
            headers
                .iter()
                .map(|h| HttpHeader {
                    name: h.key.clone(),
                    value: h.value.clone(),
                    enabled: !h.disabled.unwrap_or(false),
                })
                .collect()
        });

        // Convert query parameters
        let query_params = match &details.url {
            PostmanUrl::Object(url_obj) => url_obj.query.as_ref().map_or_else(Vec::new, |params| {
                params
                    .iter()
                    .map(|p| HttpQueryParam {
                        name: p.key.clone(),
                        value: p.value.clone(),
                        enabled: !p.disabled.unwrap_or(false),
                    })
                    .collect()
            }),
            _ => Vec::new(),
        };

        // Convert body
        let body = details
            .body
            .as_ref()
            .map_or(HttpRequestBody::None, |b| match b.mode.as_str() {
                "raw" => HttpRequestBody::Raw {
                    content: b.raw.clone().unwrap_or_default(),
                },
                "formdata" => HttpRequestBody::Form {
                    fields: b.formdata.as_ref().map_or_else(Vec::new, |fields| {
                        fields
                            .iter()
                            .map(|f| crate::models::http_client::HttpFormField {
                                name: f.key.clone(),
                                value: f.value.clone(),
                                enabled: !f.disabled.unwrap_or(false),
                            })
                            .collect()
                    }),
                },
                _ => HttpRequestBody::None,
            });

        // Convert auth
        let auth = details
            .auth
            .as_ref()
            .map_or(HttpAuth::None, |a| match a.auth_type.as_str() {
                "bearer" => {
                    if let Some(bearer_items) = &a.bearer {
                        if let Some(token_item) =
                            bearer_items.iter().find(|item| item.key == "token")
                        {
                            return HttpAuth::Bearer {
                                token: token_item.value.clone(),
                            };
                        }
                    }
                    HttpAuth::None
                }
                "basic" => {
                    let username = a
                        .basic
                        .as_ref()
                        .and_then(|items| items.iter().find(|item| item.key == "username"))
                        .map(|item| item.value.clone())
                        .unwrap_or_default();
                    let password = a
                        .basic
                        .as_ref()
                        .and_then(|items| items.iter().find(|item| item.key == "password"))
                        .map(|item| item.value.clone())
                        .unwrap_or_default();
                    HttpAuth::Basic { username, password }
                }
                _ => HttpAuth::None,
            });

        Ok(HttpRequest {
            name: postman_request.name.clone(),
            method,
            url,
            headers,
            query_params,
            body,
            auth,
            description: details.description.clone().unwrap_or_default(),
        })
    }

    /// Create demo collections if none exist
    fn create_demo_collections_if_needed(&self) -> Result<()> {
        let collections_dir = self.get_collections_directory();

        // Check if any collections already exist
        let existing_folders = self.list_all_folders().unwrap_or_default();
        if !existing_folders.is_empty() {
            return Ok(()); // Collections already exist, don't create demos
        }

        // Create demo HTTP requests
        self.create_demo_request(
            "API Testing",
            "Chuck Norris Jokes",
            HttpRequest::new("Random Joke")
                .with_method(HttpMethod::GET)
                .with_url("https://api.chucknorris.io/jokes/random"),
        )?;

        self.create_demo_request(
            "API Testing",
            "JSONPlaceholder Posts",
            HttpRequest::new("Get Posts")
                .with_method(HttpMethod::GET)
                .with_url("https://jsonplaceholder.typicode.com/posts")
                .with_header("Accept", "application/json"),
        )?;

        self.create_demo_request(
            "Weather",
            "OpenWeatherMap",
            HttpRequest::new("Current Weather")
                .with_method(HttpMethod::GET)
                .with_url(
                    "https://api.openweathermap.org/data/2.5/weather?q=London&appid=YOUR_API_KEY",
                ),
        )?;

        Ok(())
    }

    /// Helper to create a demo request
    fn create_demo_request(
        &self,
        folder_path: &str,
        name: &str,
        request: HttpRequest,
    ) -> Result<()> {
        self.save_request(folder_path, name, request)?;
        Ok(())
    }

    /// Get display path for collections directory
    pub fn get_collections_directory_display(&self) -> String {
        let path = self.get_collections_directory();

        // Try to show a shortened version for common directories
        if let Some(home_dir) = dirs::home_dir() {
            if let Ok(relative) = path.strip_prefix(&home_dir) {
                return format!("~/{}", relative.display());
            }
        }

        path.display().to_string()
    }
}

/// Sanitize a filename by removing/replacing invalid characters
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect()
}
