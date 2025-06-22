//! Safety demonstration for handling long text in Claude Code SDK
//! 
//! This example shows how the SDK protects against various risks when
//! processing long text responses:
//! 
//! 1. Memory exhaustion protection
//! 2. JSON parsing timeout monitoring  
//! 3. Log size management
//! 4. Buffer overflow prevention
//! 
//! Run examples:
//! ```bash
//! # Default safety limits
//! cargo run --example safety_demo
//! 
//! # Conservative limits (low memory)
//! cargo run --example safety_demo -- --conservative
//! 
//! # Generous limits (high memory)
//! cargo run --example safety_demo -- --generous
//! 
//! # Simulate large response
//! cargo run --example safety_demo -- --simulate-large
//! ```

use claude_code_sdk::{
    query, ClaudeCodeOptions, Message, SafetyLimits, SafetyError, ClaudeSDKError,
    ContentBlock, TextBlock
};
use std::env;
use tokio_stream::StreamExt;
use tracing::{info, warn, error, debug};

/// Demonstrate safety limits configuration
fn demonstrate_safety_limits() {
    println!("🛡️  Safety Limits Configuration");
    println!("===============================");
    
    let default_limits = SafetyLimits::default();
    println!("📋 Default Limits:");
    println!("  • Max line size: {} MB", default_limits.max_line_size / (1024 * 1024));
    println!("  • Max text block: {} MB", default_limits.max_text_block_size / (1024 * 1024));
    println!("  • Max buffer: {} MB", default_limits.max_buffer_size / (1024 * 1024));
    println!("  • Max buffered messages: {}", default_limits.max_buffered_messages);
    println!("  • JSON parse timeout: {}ms", default_limits.json_parse_timeout_ms);
    println!("  • Log preview chars: {}", default_limits.max_log_preview_chars);
    
    let conservative = SafetyLimits::conservative();
    println!("\n🔒 Conservative Limits (memory-constrained):");
    println!("  • Max line size: {} KB", conservative.max_line_size / 1024);
    println!("  • Max text block: {} KB", conservative.max_text_block_size / 1024);
    println!("  • Max buffer: {} MB", conservative.max_buffer_size / (1024 * 1024));
    
    let generous = SafetyLimits::generous();
    println!("\n🚀 Generous Limits (high-memory):");
    println!("  • Max line size: {} MB", generous.max_line_size / (1024 * 1024));
    println!("  • Max text block: {} MB", generous.max_text_block_size / (1024 * 1024));
    println!("  • Max buffer: {} MB", generous.max_buffer_size / (1024 * 1024));
    
    println!();
}

/// Demonstrate safe log preview functionality
fn demonstrate_log_safety() {
    println!("📝 Log Safety Demonstration");
    println!("===========================");
    
    let limits = SafetyLimits::default();
    
    let short_text = "This is a short message";
    let long_text = "A".repeat(1000);
    let very_long_text = "B".repeat(10000);
    
    println!("Short text preview: '{}'", limits.safe_log_preview(&short_text));
    println!("Long text preview: '{}'", limits.safe_log_preview(&long_text));
    println!("Very long text preview: '{}'", limits.safe_log_preview(&very_long_text));
    println!();
}

/// Simulate handling a large response safely
async fn simulate_large_response() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Simulating Large Response Handling");
    println!("=====================================");
    
    let limits = SafetyLimits::conservative(); // Use conservative limits for demo
    
    // Simulate different sizes of text content
    let test_cases = vec![
        ("Small", 100),
        ("Medium", 50_000),
        ("Large", 1_000_000),
        ("Very Large", 10_000_000),
    ];
    
    for (name, size) in test_cases {
        println!("\n📊 Testing {} text ({} chars):", name, size);
        
        // Check if size is within limits
        if limits.is_text_block_safe(size) {
            info!("✅ {} text is within safety limits", name);
        } else {
            warn!("⚠️  {} text exceeds safety limits ({} > {})", 
                name, size, limits.max_text_block_size);
        }
        
        // Demonstrate memory usage estimation
        let estimated_memory = size * 4; // Rough estimate: 4 bytes per char in UTF-8
        println!("   Estimated memory: {} KB", estimated_memory / 1024);
        
        // Demonstrate safe preview
        let sample_text = "X".repeat(size.min(1000)); // Only create small sample for demo
        let preview = limits.safe_log_preview(&sample_text);
        println!("   Preview: {}", preview);
    }
    
    Ok(())
}

/// Demonstrate error handling for safety violations
async fn demonstrate_safety_errors() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🚨 Safety Error Handling");
    println!("========================");
    
    // Create very restrictive limits for demonstration
    let strict_limits = SafetyLimits {
        max_line_size: 100,           // Very small
        max_text_block_size: 50,      // Very small
        max_buffer_size: 1024,        // Very small
        max_buffered_messages: 5,     // Very small
        json_parse_timeout_ms: 100,   // Very short
        max_log_preview_chars: 20,    // Very short
    };
    
    // Test line size violation
    let large_line = "x".repeat(200);
    if !strict_limits.is_line_size_safe(large_line.len()) {
        let error = SafetyError::LineTooLarge {
            actual: large_line.len(),
            limit: strict_limits.max_line_size,
        };
        println!("❌ Line size error: {}", error);
    }
    
    // Test text block violation
    let large_text = "y".repeat(100);
    if !strict_limits.is_text_block_safe(large_text.len()) {
        let error = SafetyError::TextBlockTooLarge {
            actual: large_text.len(),
            limit: strict_limits.max_text_block_size,
        };
        println!("❌ Text block error: {}", error);
    }
    
    // Test buffer size violation
    let error = SafetyError::BufferTooLarge {
        actual: 2048,
        limit: strict_limits.max_buffer_size,
    };
    println!("❌ Buffer size error: {}", error);
    
    // Test message count violation
    let error = SafetyError::TooManyMessages {
        actual: 10,
        limit: strict_limits.max_buffered_messages,
    };
    println!("❌ Message count error: {}", error);
    
    // Test timeout error
    let error = SafetyError::ParseTimeout {
        timeout_ms: strict_limits.json_parse_timeout_ms,
    };
    println!("❌ Parse timeout error: {}", error);
    
    Ok(())
}

/// Demonstrate real query with safety monitoring
async fn safe_query_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 Safe Query Demonstration");
    println!("===========================");
    
    // Note: This would normally connect to Claude CLI
    // For demo purposes, we'll show how errors would be handled
    
    let options = ClaudeCodeOptions {
        system_prompt: Some("You are a helpful assistant. Please provide a detailed response.".to_string()),
        max_turns: Some(1),
        ..Default::default()
    };
    
    info!("Starting safe query with monitoring...");
    
    match query("Explain how computers work in detail", Some(options)).await {
        Ok(mut stream) => {
            let mut message_count = 0;
            let mut total_text_length = 0;
            
            while let Some(message) = stream.next().await {
                message_count += 1;
                debug!(message_count, "Processing message");
                
                match message {
                    Message::Assistant(msg) => {
                        for content in &msg.content {
                            if let ContentBlock::Text(TextBlock { text }) = content {
                                total_text_length += text.len();
                                info!(
                                    text_length = text.len(),
                                    total_length = total_text_length,
                                    "Received text content"
                                );
                                
                                // Check if we're approaching limits
                                let limits = SafetyLimits::default();
                                if text.len() > limits.max_text_block_size / 2 {
                                    warn!("Text block is approaching size limit");
                                }
                                
                                println!("Assistant: {}", limits.safe_log_preview(text));
                            }
                        }
                    }
                    Message::Result(result) => {
                        info!(
                            duration_ms = result.duration_ms,
                            total_messages = message_count,
                            total_text_chars = total_text_length,
                            "Query completed safely"
                        );
                    }
                    _ => {
                        debug!("Other message type received");
                    }
                }
            }
        }
        Err(ClaudeSDKError::Safety(safety_error)) => {
            error!("Safety limit violation: {}", safety_error);
            println!("🛡️  Safety system prevented potential issue: {}", safety_error);
        }
        Err(other_error) => {
            error!("Other error: {}", other_error);
            println!("❌ Error: {}", other_error);
        }
    }
    
    Ok(())
}

/// Main function with different demo modes
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    claude_code_sdk::init_tracing();
    
    let args: Vec<String> = env::args().collect();
    
    println!("🛡️  Claude Code SDK - Safety Demonstration");
    println!("==========================================");
    
    match args.get(1).map(|s| s.as_str()) {
        Some("--conservative") => {
            demonstrate_safety_limits();
            println!("🔒 Running with CONSERVATIVE safety limits");
            // Would set conservative limits on transport here
        }
        Some("--generous") => {
            demonstrate_safety_limits();
            println!("🚀 Running with GENEROUS safety limits");
            // Would set generous limits on transport here
        }
        Some("--simulate-large") => {
            demonstrate_safety_limits();
            simulate_large_response().await?;
            demonstrate_safety_errors().await?;
        }
        _ => {
            demonstrate_safety_limits();
            demonstrate_log_safety();
            demonstrate_safety_errors().await?;
            
            println!("💡 Try running with --conservative, --generous, or --simulate-large");
            println!("\n🔍 Attempting safe query (may not work without Claude CLI)...");
            safe_query_demo().await?;
        }
    }
    
    println!("\n✅ Safety demonstration completed!");
    println!("\n📚 Key Safety Features:");
    println!("  • Automatic line size checking");
    println!("  • Text block size monitoring"); 
    println!("  • JSON parsing timeout detection");
    println!("  • Safe log previews (truncated)");
    println!("  • Buffer overflow prevention");
    println!("  • Memory usage estimation");
    println!("  • Graceful error handling");
    
    Ok(())
} 