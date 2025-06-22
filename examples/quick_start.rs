//! Quick start example for Claude Code SDK.

use claude_code_sdk::{
    query, AssistantMessage, ClaudeCodeOptions, ContentBlock, Message, PermissionMode,
    ResultMessage, TextBlock,
};
use tokio_stream::StreamExt;

/// Basic example - simple question
async fn basic_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic Example ===");

    let mut stream = query("What is 2 + 2?", None).await?;

    while let Some(message) = stream.next().await {
        if let Message::Assistant(AssistantMessage { content }) = message {
            for block in content {
                if let ContentBlock::Text(TextBlock { text }) = block {
                    println!("Claude: {}", text);
                }
            }
        }
    }
    println!();
    Ok(())
}

/// Example with custom options
async fn with_options_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== With Options Example ===");

    let options = ClaudeCodeOptions {
        system_prompt: Some("You are a helpful assistant that explains things simply.".to_string()),
        max_turns: Some(1),
        ..Default::default()
    };

    let mut stream = query("Explain what Rust is in one sentence.", Some(options)).await?;

    while let Some(message) = stream.next().await {
        if let Message::Assistant(AssistantMessage { content }) = message {
            for block in content {
                if let ContentBlock::Text(TextBlock { text }) = block {
                    println!("Claude: {}", text);
                }
            }
        }
    }
    println!();
    Ok(())
}

/// Example using tools
async fn with_tools_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== With Tools Example ===");

    let options = ClaudeCodeOptions {
        allowed_tools: vec!["Read".to_string(), "Write".to_string()],
        system_prompt: Some("You are a helpful file assistant.".to_string()),
        permission_mode: Some(PermissionMode::AcceptEdits),
        ..Default::default()
    };

    let mut stream = query(
        "Create a file called hello.txt with 'Hello, World!' in it",
        Some(options),
    )
    .await?;

    while let Some(message) = stream.next().await {
        match message {
            Message::Assistant(AssistantMessage { content }) => {
                for block in content {
                    if let ContentBlock::Text(TextBlock { text }) = block {
                        println!("Claude: {}", text);
                    }
                }
            }
            Message::Result(ResultMessage {
                total_cost_usd: Some(cost),
                ..
            }) if cost > 0.0 => {
                println!("\nCost: ${:.4}", cost);
            }
            _ => {}
        }
    }
    println!();
    Ok(())
}

/// Run all examples
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    basic_example().await?;
    with_options_example().await?;
    with_tools_example().await?;
    Ok(())
} 