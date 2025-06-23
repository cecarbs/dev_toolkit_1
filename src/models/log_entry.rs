use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Log levels with different priorities and colors
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Success,
}

impl LogLevel {
    /// Get the color style for ratatui based on log level
    pub fn style(&self) -> ratatui::style::Style {
        use ratatui::style::{Color, Style};

        match self {
            LogLevel::Debug => Style::default().fg(Color::Gray),
            LogLevel::Info => Style::default().fg(Color::White),
            LogLevel::Warn => Style::default().fg(Color::Yellow),
            LogLevel::Error => Style::default().fg(Color::Red),
            LogLevel::Success => Style::default().fg(Color::Green),
        }
    }

    /// Get the display string for the log level
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Success => "SUCCESS",
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A single log entry with timestamp, level, and message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Local>,
    pub level: LogLevel,
    pub message: String,
}

impl LogEntry {
    /// Create a new log entry with the current timestamp
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            timestamp: Local::now(),
            level,
            message: message.into(),
        }
    }

    /// Format the log entry for display in the UI
    pub fn formatted(&self) -> String {
        format!(
            "[{}] {:>7} {}",
            self.timestamp.format("%H:%M:%S"),
            self.level.as_str(),
            self.message
        )
    }

    /// Check if this log entry matches a search query (case-insensitive)
    pub fn matches_search(&self, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }

        let query_lower = query.to_lowercase();
        self.message.to_lowercase().contains(&query_lower)
            || self.level.as_str().to_lowercase().contains(&query_lower)
    }
}
