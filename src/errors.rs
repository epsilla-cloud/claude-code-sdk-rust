//! Error types for Claude SDK.

use thiserror::Error;
use tracing::error;
use crate::config::SafetyError;

/// Base error type for all Claude SDK errors
#[derive(Error, Debug)]
pub enum ClaudeSDKError {
    #[error("CLI connection error: {0}")]
    CLIConnection(#[from] CLIConnectionError),
    
    #[error("CLI not found: {0}")]
    CLINotFound(#[from] CLINotFoundError),
    
    #[error("Process error: {0}")]
    Process(#[from] ProcessError),
    
    #[error("JSON decode error: {0}")]
    CLIJSONDecode(#[from] CLIJSONDecodeError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Safety limit violation: {0}")]
    Safety(#[from] SafetyError),
    
    #[error("Other error: {0}")]
    Other(String),
}

/// Raised when unable to connect to Claude Code
#[derive(Error, Debug)]
#[error("Unable to connect to Claude Code: {message}")]
pub struct CLIConnectionError {
    pub message: String,
}

impl CLIConnectionError {
    pub fn new(message: impl Into<String>) -> Self {
        let message_str = message.into();
        error!(message = %message_str, "CLI connection error occurred");
        Self {
            message: message_str,
        }
    }
}

/// Raised when Claude Code is not found or not installed
#[derive(Error, Debug)]
#[error("Claude Code not found: {message}")]
pub struct CLINotFoundError {
    pub message: String,
    pub cli_path: Option<String>,
}

impl CLINotFoundError {
    pub fn new(message: impl Into<String>) -> Self {
        let message_str = message.into();
        error!(message = %message_str, "Claude CLI not found");
        Self {
            message: message_str,
            cli_path: None,
        }
    }
    
    pub fn with_path(message: impl Into<String>, cli_path: impl Into<String>) -> Self {
        let cli_path_string = cli_path.into();
        let message_str = message.into();
        error!(
            message = %message_str,
            cli_path = %cli_path_string,
            "Claude CLI not found at specified path"
        );
        Self {
            message: format!("{}: {}", message_str, cli_path_string),
            cli_path: Some(cli_path_string),
        }
    }
}

/// Raised when the CLI process fails
#[derive(Error, Debug)]
pub struct ProcessError {
    pub message: String,
    pub exit_code: Option<i32>,
    pub stderr: Option<String>,
}

impl ProcessError {
    pub fn new(message: impl Into<String>) -> Self {
        let message_str = message.into();
        error!(message = %message_str, "Process error occurred");
        Self {
            message: message_str,
            exit_code: None,
            stderr: None,
        }
    }
    
    pub fn with_exit_code(message: impl Into<String>, exit_code: i32) -> Self {
        let message_str = message.into();
        error!(
            message = %message_str,
            exit_code = exit_code,
            "Process error with exit code"
        );
        Self {
            message: message_str,
            exit_code: Some(exit_code),
            stderr: None,
        }
    }
    
    pub fn with_stderr(message: impl Into<String>, exit_code: Option<i32>, stderr: impl Into<String>) -> Self {
        let message_str = message.into();
        let stderr_str = stderr.into();
        error!(
            message = %message_str,
            exit_code = exit_code,
            stderr_preview = %stderr_str.chars().take(200).collect::<String>(),
            "Process error with stderr output"
        );
        Self {
            message: message_str,
            exit_code,
            stderr: Some(stderr_str),
        }
    }
}

impl std::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        
        if let Some(exit_code) = self.exit_code {
            write!(f, " (exit code: {})", exit_code)?;
        }
        
        if let Some(stderr) = &self.stderr {
            write!(f, "\nError output: {}", stderr)?;
        }
        
        Ok(())
    }
}

/// Raised when unable to decode JSON from CLI output
#[derive(Error, Debug)]
pub struct CLIJSONDecodeError {
    pub line: String,
    pub original_error: serde_json::Error,
}

impl CLIJSONDecodeError {
    pub fn new(line: impl Into<String>, original_error: serde_json::Error) -> Self {
        let line_str = line.into();
        error!(
            line_preview = %line_str.chars().take(100).collect::<String>(),
            parse_error = %original_error,
            "Failed to decode JSON from CLI output"
        );
        Self {
            line: line_str,
            original_error,
        }
    }
}

impl std::fmt::Display for CLIJSONDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let line_preview = if self.line.len() > 100 {
            format!("{}...", &self.line[..100])
        } else {
            self.line.clone()
        };
        
        write!(f, "Failed to decode JSON: {}", line_preview)
    }
} 