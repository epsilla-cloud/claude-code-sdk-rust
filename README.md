# Claude Code SDK for Rust

Rust SDK for Claude Code. See the [Claude Code SDK documentation](https://docs.anthropic.com/en/docs/claude-code/sdk) for more information.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
claude-code-sdk = "0.0.3"
tokio = { version = "1.0", features = ["full"] }
```

**Prerequisites:**
- Rust 1.70+
- Node.js 
- Claude Code: `npm install -g @anthropic-ai/claude-code`

## Quick Start

```rust
use claude_code_sdk::query;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Optional: Initialize logging to see detailed SDK operations
    claude_code_sdk::init_tracing();
    
    let mut stream = query("What is 2 + 2?", None).await?;
    
    while let Some(message) = stream.next().await {
        println!("{:?}", message);
    }
    
    Ok(())
}
```

## Usage

### Basic Query

```rust
use claude_code_sdk::{query, ClaudeCodeOptions, Message};
use tokio_stream::StreamExt;

// Simple query
let mut stream = query("Hello Claude", None).await?;
while let Some(message) = stream.next().await {
    match message {
        Message::Assistant(msg) => {
            for block in &msg.content {
                if let ContentBlock::Text(text_block) = block {
                    println!("Claude: {}", text_block.text);
                }
            }
        }
        _ => {}
    }
}

// With options
let options = ClaudeCodeOptions {
    system_prompt: Some("You are a helpful assistant".to_string()),
    max_turns: Some(1),
    ..Default::default()
};

let mut stream = query("Tell me a joke", Some(options)).await?;
while let Some(message) = stream.next().await {
    println!("{:?}", message);
}
```

### Using Tools

```rust
let options = ClaudeCodeOptions {
    allowed_tools: vec!["Read".to_string(), "Write".to_string()],
    permission_mode: Some(PermissionMode::AcceptEdits),
    ..Default::default()
};

let mut stream = query("Create a hello.rs file", Some(options)).await?;
while let Some(message) = stream.next().await {
    // Process tool use and results
}
```

### Working Directory

```rust
use std::path::PathBuf;

let options = ClaudeCodeOptions {
    cwd: Some(PathBuf::from("/path/to/project")),
    ..Default::default()
};
```

## Logging

The Claude Code SDK provides comprehensive structured logging using the [`tracing`](https://tracing.rs/) ecosystem. This helps with debugging, monitoring, and understanding SDK operations.

### Quick Setup

```rust
use claude_code_sdk;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize default logging
    claude_code_sdk::init_tracing();
    
    // Your code here...
    Ok(())
}
```

### Environment Variables

Control logging levels with the `RUST_LOG` environment variable:

```bash
# Show info and above (default)
RUST_LOG=claude_code_sdk=info cargo run

# Show debug messages (recommended for development)
RUST_LOG=claude_code_sdk=debug cargo run

# Show all messages including trace
RUST_LOG=claude_code_sdk=trace cargo run

# Show only errors
RUST_LOG=claude_code_sdk=error cargo run

# Multiple modules
RUST_LOG=claude_code_sdk=debug,tokio=info cargo run
```

### Custom Logging Setup

For more control over logging, set up `tracing-subscriber` directly:

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// JSON structured logging
tracing_subscriber::registry()
    .with(tracing_subscriber::EnvFilter::new("claude_code_sdk=debug"))
    .with(tracing_subscriber::fmt::layer().json())
    .init();

// Custom formatting
tracing_subscriber::registry()
    .with(tracing_subscriber::EnvFilter::new("claude_code_sdk=debug"))
    .with(
        tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .pretty()
    )
    .init();

// File logging
let file = std::fs::OpenOptions::new()
    .create(true)
    .write(true)
    .truncate(true)
    .open("claude_sdk.log")?;

tracing_subscriber::registry()
    .with(tracing_subscriber::EnvFilter::new("claude_code_sdk=trace"))
    .with(tracing_subscriber::fmt::layer().with_writer(file))
    .init();
```

### What Gets Logged

The SDK logs various operations at different levels:

- **ERROR**: CLI not found, connection failures, JSON decode errors
- **WARN**: Process termination issues, unexpected states
- **INFO**: Query start/completion, connection events, major operations
- **DEBUG**: Message processing, command building, subprocess management
- **TRACE**: Individual message parsing, detailed state changes

### Log Fields

Structured logs include useful fields:

```json
{
  "timestamp": "2024-01-01T12:00:00.000Z",
  "level": "INFO",
  "target": "claude_code_sdk::client",
  "message": "Processing query through transport",
  "fields": {
    "prompt_length": 25,
    "has_options": true,
    "system_prompt": "You are helpful",
    "allowed_tools": 2,
    "permission_mode": "AcceptEdits"
  }
}
```

### Performance Monitoring

Enable performance monitoring with tracing spans:

```rust
use tracing::{info, Instrument};

async fn my_function() -> Result<(), Box<dyn std::error::Error>> {
    let span = tracing::info_span!("my_operation", operation_type = "query");
    
    async move {
        let start = std::time::Instant::now();
        
        // Your Claude SDK operations here
        let mut stream = query("Hello", None).await?;
        // ... process stream
        
        info!(duration_ms = start.elapsed().as_millis(), "Operation completed");
        Ok(())
    }
    .instrument(span)
    .await
}
```

## Safety & Long Text Handling

The Claude Code SDK includes comprehensive safety mechanisms to handle long text responses safely and prevent common issues like memory exhaustion and performance degradation.

### Safety Limits

Configure safety limits to protect against resource exhaustion:

```rust
use claude_code_sdk::{SafetyLimits, SafetyError, ClaudeSDKError};

// Default limits (balanced)
let default_limits = SafetyLimits::default();

// Conservative limits (memory-constrained environments)
let conservative = SafetyLimits::conservative();

// Generous limits (high-memory servers)
let generous = SafetyLimits::generous();

// Custom limits
let custom = SafetyLimits {
    max_line_size: 5 * 1024 * 1024,      // 5MB per line
    max_text_block_size: 2 * 1024 * 1024, // 2MB per text block
    max_buffer_size: 20 * 1024 * 1024,   // 20MB total buffer
    max_buffered_messages: 50,            // 50 messages max
    json_parse_timeout_ms: 3000,          // 3 second timeout
    max_log_preview_chars: 150,           // 150 char preview
};
```

### Protected Operations

The SDK automatically protects against:

- **Memory Exhaustion**: Line size and text block limits prevent runaway memory usage
- **JSON Parsing Timeouts**: Large JSON objects are monitored for parsing time
- **Log Explosion**: Automatic truncation of log messages with size indicators
- **Buffer Overflow**: Channel size limits prevent message queue buildup

### Error Handling

Safety violations are reported through structured errors:

```rust
match query("Generate a very long response", None).await {
    Ok(stream) => {
        // Process normally
    }
    Err(ClaudeSDKError::Safety(SafetyError::LineTooLarge { actual, limit })) => {
        println!("Line too large: {} bytes (limit: {})", actual, limit);
    }
    Err(ClaudeSDKError::Safety(SafetyError::TextBlockTooLarge { actual, limit })) => {
        println!("Text block too large: {} bytes (limit: {})", actual, limit);
    }
    Err(ClaudeSDKError::Safety(SafetyError::ParseTimeout { timeout_ms })) => {
        println!("JSON parsing timeout after {}ms", timeout_ms);
    }
    Err(other) => {
        println!("Other error: {}", other);
    }
}
```

### Risk Mitigation

| Risk | Mitigation | Configuration |
|------|------------|---------------|
| **Memory exhaustion** | Line and text block size limits | `max_line_size`, `max_text_block_size` |
| **JSON parsing hangs** | Parsing timeout monitoring | `json_parse_timeout_ms` |
| **Log file explosion** | Safe preview truncation | `max_log_preview_chars` |
| **Message queue buildup** | Buffer size and count limits | `max_buffer_size`, `max_buffered_messages` |
| **Process blocking** | Async processing with backpressure | Automatic |

### Monitoring

The SDK provides detailed monitoring of resource usage:

```rust
use tracing::{info, warn};

// Automatic logging of resource usage
// INFO: Text block size monitoring
// WARN: Approaching safety limits  
// ERROR: Safety limit violations

// Example log output:
// INFO text_length=1048576 limit=5242880 "Large text block detected"
// WARN "Text block is approaching size limit"
// ERROR actual=10485760 limit=5242880 "Line exceeds safety limit"
```

## API Reference

### `query(prompt: &str, options: Option<ClaudeCodeOptions>)`

Main async function for querying Claude.

**Parameters:**
- `prompt`: The prompt to send to Claude
- `options`: Optional configuration

**Returns:** `Result<Pin<Box<dyn Stream<Item = Message>>>, ClaudeSDKError>`

### Types

See [src/types.rs](src/types.rs) for complete type definitions:
- `ClaudeCodeOptions` - Configuration options
- `Message` variants - `Assistant`, `User`, `System`, `Result`
- `ContentBlock` variants - `Text`, `ToolUse`, `ToolResult`

## Error Handling

```rust
use claude_code_sdk::{
    ClaudeSDKError,      // Base error
    CLINotFoundError,    // Claude Code not installed
    CLIConnectionError,  // Connection issues
    ProcessError,        // Process failed
    CLIJSONDecodeError,  // JSON parsing issues
};

match query("Hello", None).await {
    Ok(mut stream) => {
        // Process messages
    }
    Err(ClaudeSDKError::CLINotFound(e)) => {
        println!("Please install Claude Code: {}", e);
    }
    Err(ClaudeSDKError::Process(e)) => {
        println!("Process failed with exit code: {:?}", e.exit_code);
    }
    Err(e) => {
        println!("Error: {}", e);
    }
}
```

## Available Tools

See the [Claude Code documentation](https://docs.anthropic.com/en/docs/claude-code/security#tools-available-to-claude) for a complete list of available tools.

## Examples

- **[examples/quick_start.rs](examples/quick_start.rs)** - Complete working example with logging
- **[examples/logging_demo.rs](examples/logging_demo.rs)** - Advanced logging configurations
- **[examples/safety_demo.rs](examples/safety_demo.rs)** - Safety mechanisms and long text handling

### Running Examples

```bash
# Basic example with default logging
cargo run --example quick_start

# Advanced logging demo with JSON output
cargo run --example logging_demo -- --json

# File logging
cargo run --example logging_demo -- --file logs/claude.log

# Performance monitoring
RUST_LOG=claude_code_sdk=debug cargo run --example logging_demo -- --perf

# Safety demonstration
cargo run --example safety_demo -- --simulate-large
```

## License

MIT 