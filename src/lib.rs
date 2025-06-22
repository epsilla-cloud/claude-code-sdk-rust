//! Claude SDK for Rust
//! 
//! Rust SDK for interacting with Claude Code.

use std::pin::Pin;
use futures::Stream;

pub mod types;
pub mod errors;
mod client;
mod transport;

pub use types::*;
pub use errors::*;

use client::InternalClient;

/// Query Claude Code.
/// 
/// Rust SDK for interacting with Claude Code.
/// 
/// # Arguments
/// 
/// * `prompt` - The prompt to send to Claude
/// * `options` - Optional configuration (defaults to ClaudeCodeOptions::default() if None).
///              Set options.permission_mode to control tool execution:
///              - `Default`: CLI prompts for dangerous tools  
///              - `AcceptEdits`: Auto-accept file edits
///              - `BypassPermissions`: Allow all tools (use with caution)
///              Set options.cwd for working directory.
/// 
/// # Returns
/// 
/// A stream of messages from the conversation
/// 
/// # Example
/// 
/// ```rust,no_run
/// use claude_code_sdk::{query, ClaudeCodeOptions};
/// use tokio_stream::StreamExt;
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Simple usage
///     let mut stream = query("Hello", None).await?;
///     while let Some(message) = stream.next().await {
///         println!("{:?}", message);
///     }
///     
///     // With options
///     let options = ClaudeCodeOptions {
///         system_prompt: Some("You are helpful".to_string()),
///         cwd: Some("/home/user".into()),
///         ..Default::default()
///     };
///     
///     let mut stream = query("Hello", Some(options)).await?;
///     while let Some(message) = stream.next().await {
///         println!("{:?}", message);
///     }
///     
///     Ok(())
/// }
/// ```
pub async fn query(
    prompt: &str,
    options: Option<ClaudeCodeOptions>,
) -> Result<Pin<Box<dyn Stream<Item = Message> + Send>>, ClaudeSDKError> {
    let options = options.unwrap_or_default();
    
    // Set environment variable
    std::env::set_var("CLAUDE_CODE_ENTRYPOINT", "sdk-rust");
    
    let client = InternalClient::new();
    let stream = client.process_query(prompt, options).await?;
    
    Ok(Box::pin(stream))
} 