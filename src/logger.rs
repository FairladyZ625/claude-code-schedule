use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct LogEntry {
    pub timestamp: DateTime<Local>,
    pub action: String,
    pub status: String,
    pub message: Option<String>,
    pub response_content: Option<String>,
    pub cycle_number: Option<u32>,
}

impl LogEntry {
    #[allow(dead_code)]
    pub fn new(action: &str, status: &str, message: Option<String>) -> Self {
        Self {
            timestamp: Local::now(),
            action: action.to_string(),
            status: status.to_string(),
            message,
            response_content: None,
            cycle_number: None,
        }
    }

    pub fn new_with_response(
        action: &str,
        status: &str,
        message: Option<String>,
        response_content: Option<String>,
        cycle_number: Option<u32>,
    ) -> Self {
        Self {
            timestamp: Local::now(),
            action: action.to_string(),
            status: status.to_string(),
            message,
            response_content,
            cycle_number,
        }
    }

    #[allow(dead_code)]
    pub fn success(action: &str, message: Option<String>) -> Self {
        Self::new(action, "success", message)
    }

    pub fn success_with_response(
        action: &str,
        message: Option<String>,
        response_content: Option<String>,
        cycle_number: Option<u32>,
    ) -> Self {
        Self::new_with_response(action, "success", message, response_content, cycle_number)
    }

    #[allow(dead_code)]
    pub fn error(action: &str, message: Option<String>) -> Self {
        Self::new(action, "error", message)
    }

    pub fn error_with_response(
        action: &str,
        message: Option<String>,
        response_content: Option<String>,
        cycle_number: Option<u32>,
    ) -> Self {
        Self::new_with_response(action, "error", message, response_content, cycle_number)
    }
}

pub struct Logger {
    log_dir: String,
}

impl Logger {
    pub fn new(log_dir: &str) -> Self {
        Self {
            log_dir: log_dir.to_string(),
        }
    }

    pub fn init(&self) -> Result<()> {
        // Create log directory if it doesn't exist
        if !Path::new(&self.log_dir).exists() {
            fs::create_dir_all(&self.log_dir).context("Failed to create log directory")?;
        }
        Ok(())
    }

    pub fn log(&self, entry: LogEntry) -> Result<()> {
        let date_str = entry.timestamp.format("%Y-%m-%d").to_string();
        let log_file_path = format!("{}/{}.log", self.log_dir, date_str);

        let json_line = serde_json::to_string(&entry).context("Failed to serialize log entry")?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file_path)
            .context("Failed to open log file")?;

        writeln!(file, "{json_line}").context("Failed to write to log file")?;

        // Also print to console for immediate feedback
        println!(
            "LOG: {} - {} - {}",
            entry.timestamp.format("%H:%M:%S"),
            entry.action,
            entry.status
        );

        if let Some(msg) = &entry.message {
            println!("     {msg}");
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn log_ping_success(&self) -> Result<()> {
        let entry = LogEntry::success("ping", Some("Ping sent successfully".to_string()));
        self.log(entry)
    }

    pub fn log_ping_success_with_response(
        &self,
        response: &str,
        cycle_number: Option<u32>,
    ) -> Result<()> {
        let entry = LogEntry::success_with_response(
            "ping",
            Some("Ping sent successfully".to_string()),
            Some(response.to_string()),
            cycle_number,
        );
        self.log(entry)
    }

    #[allow(dead_code)]
    pub fn log_ping_error(&self, error_msg: &str) -> Result<()> {
        let entry = LogEntry::error("ping", Some(error_msg.to_string()));
        self.log(entry)
    }

    pub fn log_ping_error_with_cycle(&self, error_msg: &str, cycle_number: Option<u32>) -> Result<()> {
        let entry = LogEntry::error_with_response(
            "ping",
            Some(error_msg.to_string()),
            None,
            cycle_number,
        );
        self.log(entry)
    }

    #[allow(dead_code)]
    pub fn log_claude_success(&self) -> Result<()> {
        let entry = LogEntry::success(
            "claude",
            Some("Claude command executed successfully".to_string()),
        );
        self.log(entry)
    }

    pub fn log_claude_success_with_response(
        &self,
        response: &str,
        cycle_number: Option<u32>,
    ) -> Result<()> {
        let entry = LogEntry::success_with_response(
            "claude",
            Some("Claude command executed successfully".to_string()),
            Some(response.to_string()),
            cycle_number,
        );
        self.log(entry)
    }

    #[allow(dead_code)]
    pub fn log_claude_error(&self, error_msg: &str) -> Result<()> {
        let entry = LogEntry::error("claude", Some(error_msg.to_string()));
        self.log(entry)
    }

    pub fn log_claude_error_with_cycle(&self, error_msg: &str, cycle_number: Option<u32>) -> Result<()> {
        let entry = LogEntry::error_with_response(
            "claude",
            Some(error_msg.to_string()),
            None,
            cycle_number,
        );
        self.log(entry)
    }

    pub fn log_cycle_start(&self, cycle_number: u32) -> Result<()> {
        let entry = LogEntry::new_with_response(
            "cycle",
            "start",
            Some(format!("Starting cycle {cycle_number}")),
            None,
            Some(cycle_number),
        );
        self.log(entry)
    }

    pub fn log_cycle_end(&self, cycle_number: u32) -> Result<()> {
        let entry = LogEntry::new_with_response(
            "cycle",
            "end",
            Some(format!("Completed cycle {cycle_number}")),
            None,
            Some(cycle_number),
        );
        self.log(entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::tempdir;

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::success("test", Some("test message".to_string()));
        assert_eq!(entry.action, "test");
        assert_eq!(entry.status, "success");
        assert_eq!(entry.message, Some("test message".to_string()));
        assert_eq!(entry.response_content, None);
        assert_eq!(entry.cycle_number, None);
    }

    #[test]
    fn test_log_entry_with_response() {
        let entry = LogEntry::success_with_response(
            "ping",
            Some("test message".to_string()),
            Some("response content".to_string()),
            Some(5),
        );
        assert_eq!(entry.action, "ping");
        assert_eq!(entry.status, "success");
        assert_eq!(entry.message, Some("test message".to_string()));
        assert_eq!(entry.response_content, Some("response content".to_string()));
        assert_eq!(entry.cycle_number, Some(5));
    }

    #[test]
    fn test_logger_init() {
        let temp_dir = tempdir().unwrap();
        let log_dir = temp_dir.path().join("logs").to_string_lossy().to_string();

        let logger = Logger::new(&log_dir);
        assert!(logger.init().is_ok());
        assert!(Path::new(&log_dir).exists());
    }

    #[test]
    fn test_logger_log() {
        let temp_dir = tempdir().unwrap();
        let log_dir = temp_dir.path().to_string_lossy().to_string();

        let logger = Logger::new(&log_dir);
        logger.init().unwrap();

        let entry = LogEntry::success("test", Some("test message".to_string()));
        assert!(logger.log(entry).is_ok());

        // Check if log file was created
        let date_str = Local::now().format("%Y-%m-%d").to_string();
        let log_file_path = format!("{}/{}.log", log_dir, date_str);
        assert!(Path::new(&log_file_path).exists());
    }
}
