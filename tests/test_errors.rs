//! Tests for Claude SDK error types.

use claude_code_sdk::{CLIConnectionError, CLIJSONDecodeError, CLINotFoundError, ProcessError};

#[test]
fn test_cli_not_found_error() {
    let error = CLINotFoundError::new("Claude Code not found");
    assert_eq!(error.message, "Claude Code not found");
    assert!(error.cli_path.is_none());
    
    let error_with_path = CLINotFoundError::with_path("Claude Code not found", "/usr/bin/claude");
    assert_eq!(error_with_path.message, "Claude Code not found: /usr/bin/claude");
    assert_eq!(error_with_path.cli_path.as_ref().unwrap(), "/usr/bin/claude");
}

#[test]
fn test_cli_connection_error() {
    let error = CLIConnectionError::new("Connection failed");
    assert_eq!(error.message, "Connection failed");
}

#[test]
fn test_process_error() {
    let error = ProcessError::new("Process failed");
    assert_eq!(error.message, "Process failed");
    assert!(error.exit_code.is_none());
    assert!(error.stderr.is_none());
    
    let error_with_code = ProcessError::with_exit_code("Process failed", 1);
    assert_eq!(error_with_code.message, "Process failed");
    assert_eq!(error_with_code.exit_code, Some(1));
    
    let error_with_stderr = ProcessError::with_stderr("Process failed", Some(1), "Error output");
    assert_eq!(error_with_stderr.message, "Process failed");
    assert_eq!(error_with_stderr.exit_code, Some(1));
    assert_eq!(error_with_stderr.stderr.as_ref().unwrap(), "Error output");
}

#[test]
fn test_cli_json_decode_error() {
    let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let error = CLIJSONDecodeError::new("invalid json", json_error);
    
    assert_eq!(error.line, "invalid json");
    assert!(!error.original_error.to_string().is_empty());
}

#[test]
fn test_error_display() {
    let error = ProcessError::with_stderr("Process failed", Some(1), "Error output");
    let display_string = format!("{}", error);
    
    assert!(display_string.contains("Process failed"));
    assert!(display_string.contains("exit code: 1"));
    assert!(display_string.contains("Error output"));
}

#[test]
fn test_cli_json_decode_error_display() {
    let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let error = CLIJSONDecodeError::new("invalid json data", json_error);
    let display_string = format!("{}", error);
    
    assert!(display_string.contains("Failed to decode JSON"));
    assert!(display_string.contains("invalid json data"));
}

#[test]  
fn test_cli_json_decode_error_truncation() {
    let long_line = "a".repeat(200);
    let json_error = serde_json::from_str::<serde_json::Value>(&long_line).unwrap_err();
    let error = CLIJSONDecodeError::new(&long_line, json_error);
    let display_string = format!("{}", error);
    
    assert!(display_string.contains("..."));
    assert!(display_string.len() < long_line.len() + 50); // Should be truncated
} 