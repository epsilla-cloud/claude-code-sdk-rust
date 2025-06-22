# Claude Code SDK for Rust

Rust SDK for Claude Code. See the [Claude Code SDK documentation](https://docs.anthropic.com/en/docs/claude-code/sdk) for more information.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
claude-code-sdk = "0.0.10"
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

See [examples/quick_start.rs](examples/quick_start.rs) for a complete working example.

## License

MIT 