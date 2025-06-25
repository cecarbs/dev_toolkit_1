use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::models::NodeType;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClipboardOperation {
    Cut,
    Copy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub operation: ClipboardOperation,
    pub item_type: NodeType,
    pub name: String,
    pub path: String,
    pub full_file_path: PathBuf, // For filesystem operations
}
