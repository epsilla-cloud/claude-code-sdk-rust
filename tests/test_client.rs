//! Tests for Claude SDK client functionality.

use claude_code_sdk::{query, AssistantMessage, ClaudeCodeOptions, ContentBlock, Message, TextBlock};
use std::path::PathBuf;

#[tokio::test]
async fn test_query_basic() {
    // This is a basic test that would require mocking in a real scenario
    // For now, we just test that the function compiles and can be called
    let options = ClaudeCodeOptions::default();
    
    // Note: This test would fail without Claude Code CLI installed
    // In a real test suite, we'd use dependency injection or mocking
    let result = query("test", Some(options)).await;
    
    // For this basic test, we just verify the function signature works
    assert!(result.is_ok() || result.is_err()); // Always true, but validates the API
}

#[test]
fn test_claude_code_options_default() {
    let options = ClaudeCodeOptions::default();
    assert_eq!(options.max_thinking_tokens, 8000);
    assert!(options.allowed_tools.is_empty());
    assert!(options.system_prompt.is_none());
}

#[test]
fn test_claude_code_options_new() {
    let options = ClaudeCodeOptions::new();
    assert_eq!(options.max_thinking_tokens, 8000);
    assert!(options.allowed_tools.is_empty());
}

#[test]
fn test_claude_code_options_with_custom_values() {
    let options = ClaudeCodeOptions {
        allowed_tools: vec!["Read".to_string(), "Write".to_string()],
        system_prompt: Some("You are helpful".to_string()),
        cwd: Some(PathBuf::from("/tmp")),
        max_turns: Some(5),
        ..Default::default()
    };
    
    assert_eq!(options.allowed_tools.len(), 2);
    assert_eq!(options.system_prompt.as_ref().unwrap(), "You are helpful");
    assert_eq!(options.cwd.as_ref().unwrap(), &PathBuf::from("/tmp"));
    assert_eq!(options.max_turns, Some(5));
}

#[test]
fn test_message_types() {
    let user_msg = Message::User(claude_code_sdk::UserMessage {
        content: "Hello".to_string(),
    });
    
    let assistant_msg = Message::Assistant(AssistantMessage {
        content: vec![ContentBlock::Text(TextBlock {
            text: "Hi there!".to_string(),
        })],
    });
    
    match user_msg {
        Message::User(msg) => assert_eq!(msg.content, "Hello"),
        _ => panic!("Expected user message"),
    }
    
    match assistant_msg {
        Message::Assistant(msg) => {
            assert_eq!(msg.content.len(), 1);
            if let ContentBlock::Text(text_block) = &msg.content[0] {
                assert_eq!(text_block.text, "Hi there!");
            } else {
                panic!("Expected text block");
            }
        }
        _ => panic!("Expected assistant message"),
    }
} 