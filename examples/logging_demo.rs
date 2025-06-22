//! Advanced logging demonstration for Claude Code SDK
//! 
//! This example shows various logging configurations and features:
//! 1. JSON structured logging
//! 2. Custom log formatters
//! 3. File logging
//! 4. Performance monitoring
//! 5. Log filtering and levels
//! 
//! Run examples:
//! ```bash
//! # JSON logging
//! cargo run --example logging_demo -- --json
//! 
//! # File logging
//! cargo run --example logging_demo -- --file logs/claude_sdk.log
//! 
//! # Custom format
//! cargo run --example logging_demo -- --custom
//! 
//! # Performance monitoring
//! cargo run --example logging_demo -- --perf
//! ```

use claude_code_sdk::{query, ClaudeCodeOptions, Message};
use std::env;
use std::path::Path;
use tokio_stream::StreamExt;
use tracing::{debug, error, info, warn, Instrument};
use tracing_subscriber::{
    fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

/// Setup JSON structured logging
fn setup_json_logging() {
    println!("🔧 Setting up JSON structured logging...");

    tracing_subscriber::registry()
        .with(EnvFilter::new("claude_code_sdk=debug"))
        .with(
            fmt::layer()
                .json()
                .with_target(true)
                .with_current_span(false)
                .with_span_list(true),
        )
        .init();

    info!("JSON logging initialized");
}

/// Setup file logging
fn setup_file_logging(log_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("📁 Setting up file logging to: {}", log_file);

    // Create directory if it doesn't exist
    if let Some(parent) = Path::new(log_file).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_file)?;

    tracing_subscriber::registry()
        .with(EnvFilter::new("claude_code_sdk=trace"))
        .with(
            fmt::layer()
                .with_writer(file)
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .with_ansi(false), // No colors in file
        )
        .with(
            fmt::layer()
                .with_writer(std::io::stdout)
                .with_target(false)
                .compact(),
        )
        .init();

    info!("File logging initialized, writing to: {}", log_file);
    Ok(())
}

/// Setup custom formatted logging
fn setup_custom_logging() {
    println!("🎨 Setting up custom formatted logging...");

    tracing_subscriber::registry()
        .with(EnvFilter::new("claude_code_sdk=debug"))
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .with_level(true)
                .with_timer(fmt::time::ChronoUtc::rfc_3339())
                .pretty(),
        )
        .init();

    info!("Custom logging initialized");
}

/// Setup performance monitoring
fn setup_performance_logging() {
    println!("⚡ Setting up performance monitoring logging...");

    tracing_subscriber::registry()
        .with(EnvFilter::new("claude_code_sdk=trace"))
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_timer(fmt::time::ChronoUtc::rfc_3339())
                .json(),
        )
        .init();

    info!("Performance logging initialized");
}

/// Demonstrate query with performance tracking
async fn performance_demo() -> Result<(), Box<dyn std::error::Error>> {
    let span = tracing::info_span!("performance_demo", demo_type = "simple_query");
    
    async move {
        info!("Starting performance demonstration");
        let start_time = std::time::Instant::now();

        let options = ClaudeCodeOptions {
            system_prompt: Some("You are a helpful assistant.".to_string()),
            max_turns: Some(1),
            ..Default::default()
        };

        let mut stream = query("What is the capital of France?", Some(options)).await?;
        let mut message_count = 0u64;
        let mut total_text_length = 0usize;

        while let Some(message) = stream.next().await {
            message_count += 1;
            
            match message {
                Message::Assistant(msg) => {
                    for content in &msg.content {
                        if let claude_code_sdk::ContentBlock::Text(text_block) = content {
                            total_text_length += text_block.text.len();
                            debug!(
                                text_length = text_block.text.len(),
                                total_length = total_text_length,
                                "Assistant text received"
                            );
                        }
                    }
                }
                Message::Result(result) => {
                    info!(
                        duration_ms = result.duration_ms,
                        api_duration_ms = result.duration_api_ms,
                        turns = result.num_turns,
                        cost_usd = result.total_cost_usd,
                        is_error = result.is_error,
                        "Query result metrics"
                    );
                }
                _ => {}
            }
        }

        let elapsed = start_time.elapsed();
        info!(
            total_duration_ms = elapsed.as_millis(),
            message_count = message_count,
            total_text_length = total_text_length,
            avg_text_per_message = if message_count > 0 { total_text_length / message_count as usize } else { 0 },
            "Performance demo completed"
        );

        Ok(())
    }
    .instrument(span)
    .await
}

/// Demonstrate error logging
#[allow(dead_code)]
async fn error_demo() -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting error demonstration");

    // Try to use an invalid CLI path to trigger an error
    let options = ClaudeCodeOptions {
        ..Default::default()
    };

    match query("This should fail", Some(options)).await {
        Ok(mut stream) => {
            warn!("Expected error but got success");
            while let Some(message) = stream.next().await {
                debug!(?message, "Unexpected message received");
            }
        }
        Err(e) => {
            error!(error = %e, "Expected error occurred (this is good for demo)");
            println!("✓ Error logging demonstrated successfully");
        }
    }

    Ok(())
}

/// Main function with argument parsing
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("--json") => {
            setup_json_logging();
            println!("📊 Running with JSON logging");
        }
        Some("--file") => {
            let default_log_file = "claude_sdk.log".to_string();
            let log_file = args.get(2).unwrap_or(&default_log_file);
            setup_file_logging(log_file)?;
            println!("📁 Running with file logging");
        }
        Some("--custom") => {
            setup_custom_logging();
            println!("🎨 Running with custom formatting");
        }
        Some("--perf") => {
            setup_performance_logging();
            println!("⚡ Running with performance monitoring");
            performance_demo().await?;
            return Ok(());
        }
        _ => {
            // Default logging
            claude_code_sdk::init_tracing();
            println!("🔍 Running with default logging");
            println!("💡 Try: --json, --file <path>, --custom, or --perf");
        }
    }

    println!("\n=== Claude Code SDK Logging Demo ===");

    // Basic query demo
    info!("Starting basic query demonstration");
    let mut stream = query("Hello! Can you tell me what 5 + 3 equals?", None).await?;

    while let Some(message) = stream.next().await {
        match message {
            Message::Assistant(msg) => {
                info!("Assistant response received with {} content blocks", msg.content.len());
                for content in &msg.content {
                    if let claude_code_sdk::ContentBlock::Text(text_block) = content {
                        println!("Assistant: {}", text_block.text);
                        debug!(response_length = text_block.text.len(), "Response text metrics");
                    }
                }
            }
            Message::System(msg) => {
                debug!(subtype = %msg.subtype, "System message: {}", msg.subtype);
            }
            Message::Result(msg) => {
                info!(
                    duration_ms = msg.duration_ms,
                    turns = msg.num_turns,
                    cost = msg.total_cost_usd,
                    is_error = msg.is_error,
                    "Query completed"
                );
                
                if msg.is_error {
                    error!("Query failed: {:?}", msg.result);
                } else {
                    info!("Query succeeded");
                }
            }
            _ => {
                debug!("Other message type received");
            }
        }
    }

    // Error demonstration (comment out if you don't want to see errors)
    // error_demo().await?;

    info!("Logging demonstration completed");
    println!("\n✅ Logging demo completed successfully!");
    println!("Check the logs above to see the different log levels and formats.");

    Ok(())
} 