//! Quick start example for Claude Code SDK with comprehensive logging.
//! 
//! This example demonstrates:
//! 1. Basic usage with default logging
//! 2. Custom logging configuration
//! 3. Structured logging with different levels
//! 
//! Run with different log levels:
//! ```bash
//! # Show info and above
//! RUST_LOG=claude_code_sdk=info cargo run --example quick_start
//! 
//! # Show debug and above (recommended for development)
//! RUST_LOG=claude_code_sdk=debug cargo run --example quick_start
//! 
//! # Show all logs including trace
//! RUST_LOG=claude_code_sdk=trace cargo run --example quick_start
//! 
//! # Show only errors
//! RUST_LOG=claude_code_sdk=error cargo run --example quick_start
//! ```

use claude_code_sdk::{
    query, AssistantMessage, ClaudeCodeOptions, ContentBlock, Message, PermissionMode,
    ResultMessage, TextBlock,
};
use tokio_stream::StreamExt;
use tracing::{info, debug, error};

/// Basic example - simple question with logging
async fn basic_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic Example ===");
    info!("Starting basic example");

    let mut stream = query("What is 2 + 2?", None).await?;
    let mut message_count = 0;

    while let Some(message) = stream.next().await {
        message_count += 1;
        debug!(message_count, "Received message from stream");
        
        match message {
            Message::Assistant(AssistantMessage { content }) => {
                info!("Received assistant message with {} content blocks", content.len());
                for block in content {
                    if let ContentBlock::Text(TextBlock { text }) = block {
                        println!("Claude: {}", text);
                    }
                }
            }
            Message::System(msg) => {
                debug!(subtype = %msg.subtype, "Received system message");
            }
            Message::Result(msg) => {
                info!(
                    duration_ms = msg.duration_ms,
                    num_turns = msg.num_turns,
                    is_error = msg.is_error,
                    "Query completed"
                );
                if let Some(cost) = msg.total_cost_usd {
                    info!(cost_usd = cost, "Query cost");
                }
            }
            _ => {
                debug!("Received other message type");
            }
        }
    }
    
    info!(total_messages = message_count, "Basic example completed");
    println!();
    Ok(())
}

/// Example with custom options and detailed logging
async fn with_options_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== With Options Example ===");
    info!("Starting options example with custom configuration");

    let options = ClaudeCodeOptions {
        system_prompt: Some("You are a helpful assistant that explains things simply.".to_string()),
        max_turns: Some(1),
        ..Default::default()
    };

    debug!(?options, "Using custom options");
    let mut stream = query("Explain what Rust is in one sentence.", Some(options)).await?;

    while let Some(message) = stream.next().await {
        match message {
            Message::Assistant(AssistantMessage { content }) => {
                info!("Assistant response received");
                for block in content {
                    if let ContentBlock::Text(TextBlock { text }) = block {
                        debug!(response_length = text.len(), "Response text length");
                        println!("Claude: {}", text);
                    }
                }
            }
            Message::Result(msg) => {
                if msg.is_error {
                    error!("Query failed: {:?}", msg.result);
                } else {
                    info!("Query succeeded");
                }
            }
            _ => {
                debug!("Received non-assistant message");
            }
        }
    }
    
    info!("Options example completed");
    println!();
    Ok(())
}

/// Example using tools with comprehensive logging
async fn with_tools_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== With Tools Example ===");
    info!("Starting tools example with file operations");

    let options = ClaudeCodeOptions {
        allowed_tools: vec!["Read".to_string(), "Write".to_string()],
        system_prompt: Some("You are a helpful file assistant.".to_string()),
        permission_mode: Some(PermissionMode::AcceptEdits),
        ..Default::default()
    };

    info!(
        allowed_tools = ?options.allowed_tools,
        permission_mode = ?options.permission_mode,
        "Configured tools and permissions"
    );

    let mut stream = query(
        "Create a file called hello.txt with 'Hello, World!' in it",
        Some(options),
    )
    .await?;

    let mut tool_use_count = 0;
    while let Some(message) = stream.next().await {
        match message {
            Message::Assistant(AssistantMessage { content }) => {
                debug!("Processing assistant message with {} content blocks", content.len());
                for block in content {
                    match block {
                        ContentBlock::Text(TextBlock { text }) => {
                            println!("Claude: {}", text);
                        }
                        ContentBlock::ToolUse(tool_use_block) => {
                            tool_use_count += 1;
                            info!(
                                tool_name = %tool_use_block.name,
                                tool_count = tool_use_count,
                                "Tool used by assistant"
                            );
                            debug!(tool_input = ?tool_use_block.input, "Tool input details");
                            println!("Tool used: {} with input: {:?}", tool_use_block.name, tool_use_block.input);
                        }
                        _ => {
                            debug!("Other content block type received");
                        }
                    }
                }
            }
            Message::System(msg) => {
                debug!(system_subtype = %msg.subtype, "System message received");
            }
            Message::Result(ResultMessage {
                total_cost_usd: Some(cost),
                duration_ms,
                num_turns,
                is_error,
                ..
            }) => {
                if is_error {
                    error!(duration_ms, num_turns, "Query failed");
                } else {
                    info!(
                        cost_usd = cost,
                        duration_ms,
                        num_turns,
                        tools_used = tool_use_count,
                        "Query completed successfully"
                    );
                    if cost > 0.0 {
                        println!("\nCost: ${:.4}", cost);
                    }
                }
            }
            _ => {
                debug!("Other message type received");
            }
        }
    }
    
    info!(total_tools_used = tool_use_count, "Tools example completed");
    println!();
    Ok(())
}

/// Demonstrate custom logging setup
fn setup_custom_logging() {
    println!("=== Custom Logging Setup Demo ===");
    
    // This is an alternative to claude_code_sdk::init_tracing()
    // You can customize the logging format, level, and output
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, fmt, EnvFilter};
    
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("claude_code_sdk=info"));
    
    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
        )
        .init();
    
    info!("Custom logging initialized");
    println!("Custom logging setup complete!");
    println!();
}

/// Run all examples with comprehensive logging
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing/logging - comment this out if you want to use custom setup instead
    claude_code_sdk::init_tracing();
    
    // Alternative: use custom logging setup
    // setup_custom_logging();
    
    info!("Starting Claude Code SDK examples with logging");
    
    println!("Claude Code SDK - Logging Examples");
    println!("==================================");
    println!("💡 Tip: Set RUST_LOG=claude_code_sdk=debug for detailed logs");
    println!("💡 Tip: Set RUST_LOG=claude_code_sdk=trace for all logs");
    println!();
    
    match basic_example().await {
        Ok(()) => info!("Basic example completed successfully"),
        Err(e) => error!(error = %e, "Basic example failed"),
    }
    
    match with_options_example().await {
        Ok(()) => info!("Options example completed successfully"), 
        Err(e) => error!(error = %e, "Options example failed"),
    }
    
    match with_tools_example().await {
        Ok(()) => info!("Tools example completed successfully"),
        Err(e) => error!(error = %e, "Tools example failed"),
    }
    
    info!("All examples completed");
    
    println!("Examples completed! Check the logs above for detailed information.");
    println!("\nLogging features demonstrated:");
    println!("✓ Structured logging with fields");
    println!("✓ Different log levels (info, debug, warn, error)");
    println!("✓ Message counting and metrics");
    println!("✓ Tool usage tracking"); 
    println!("✓ Performance monitoring (duration, cost)");
    println!("✓ Error handling and reporting");
    
    Ok(())
} 