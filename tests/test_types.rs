//! Tests for Claude SDK types.

use claude_code_sdk::{
    AssistantMessage, ContentBlock, Message, PermissionMode, TextBlock,
    ToolResultContent, ToolUseBlock, UserMessage,
};
use std::collections::HashMap;

#[test]
fn test_permission_mode_serialization() {
    let mode = PermissionMode::Default;
    let serialized = serde_json::to_string(&mode).unwrap();
    assert_eq!(serialized, "\"default\"");
    
    let mode = PermissionMode::AcceptEdits;
    let serialized = serde_json::to_string(&mode).unwrap();
    assert_eq!(serialized, "\"acceptEdits\"");
    
    let mode = PermissionMode::BypassPermissions;
    let serialized = serde_json::to_string(&mode).unwrap();
    assert_eq!(serialized, "\"bypassPermissions\"");
}

#[test]
fn test_content_block_serialization() {
    let text_block = ContentBlock::Text(TextBlock {
        text: "Hello".to_string(),
    });
    
    let serialized = serde_json::to_string(&text_block).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(parsed["type"], "text");
    assert_eq!(parsed["text"], "Hello");
}

#[test]
fn test_tool_use_block_serialization() {
    let mut input = HashMap::new();
    input.insert("param1".to_string(), serde_json::json!("value1"));
    input.insert("param2".to_string(), serde_json::json!(42));
    
    let tool_use_block = ContentBlock::ToolUse(ToolUseBlock {
        id: "tool_123".to_string(),
        name: "file_reader".to_string(),
        input,
    });
    
    let serialized = serde_json::to_string(&tool_use_block).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(parsed["type"], "tool_use");
    assert_eq!(parsed["id"], "tool_123");
    assert_eq!(parsed["name"], "file_reader");
    assert_eq!(parsed["input"]["param1"], "value1");
    assert_eq!(parsed["input"]["param2"], 42);
}

#[test]
fn test_tool_result_content_variants() {
    let text_content = ToolResultContent::Text("Result text".to_string());
    let serialized = serde_json::to_string(&text_content).unwrap();
    assert_eq!(serialized, "\"Result text\"");
    
    let structured_data = vec![{
        let mut map = HashMap::new();
        map.insert("key".to_string(), serde_json::json!("value"));
        map
    }];
    let structured_content = ToolResultContent::Structured(structured_data);
    let serialized = serde_json::to_string(&structured_content).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(parsed[0]["key"], "value");
}

#[test]
fn test_message_serialization() {
    let user_message = Message::User(UserMessage {
        content: "Hello".to_string(),
    });
    
    let serialized = serde_json::to_string(&user_message).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(parsed["type"], "user");
    assert_eq!(parsed["content"], "Hello");
}

#[test]
fn test_assistant_message_serialization() {
    let assistant_message = Message::Assistant(AssistantMessage {
        content: vec![
            ContentBlock::Text(TextBlock {
                text: "Hello there!".to_string(),
            }),
            ContentBlock::ToolUse(ToolUseBlock {
                id: "tool_456".to_string(),
                name: "calculator".to_string(),
                input: {
                    let mut map = HashMap::new();
                    map.insert("operation".to_string(), serde_json::json!("add"));
                    map.insert("numbers".to_string(), serde_json::json!([1, 2]));
                    map
                },
            }),
        ],
    });
    
    let serialized = serde_json::to_string(&assistant_message).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(parsed["type"], "assistant");
    assert_eq!(parsed["content"].as_array().unwrap().len(), 2);
    assert_eq!(parsed["content"][0]["type"], "text");
    assert_eq!(parsed["content"][0]["text"], "Hello there!");
    assert_eq!(parsed["content"][1]["type"], "tool_use");
    assert_eq!(parsed["content"][1]["name"], "calculator");
}

#[test]
fn test_message_deserialization() {
    let json_data = r#"{
        "type": "assistant",
        "content": [
            {
                "type": "text", 
                "text": "The result is 4"
            }
        ]
    }"#;
    
    let message: Message = serde_json::from_str(json_data).unwrap();
    
    match message {
        Message::Assistant(AssistantMessage { content }) => {
            assert_eq!(content.len(), 1);
            if let ContentBlock::Text(TextBlock { text }) = &content[0] {
                assert_eq!(text, "The result is 4");
            } else {
                panic!("Expected text content block");
            }
        }
        _ => panic!("Expected assistant message"),
    }
} 