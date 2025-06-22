//! Claude SDK for Rust
//! 
//! Rust SDK for interacting with Claude Code.
//! 
//! This SDK provides structured logging through the `tracing` crate.
//! To enable logging, initialize a tracing subscriber before using the SDK:
//! 
//! ```rust,no_run
//! use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
//! 
//! tracing_subscriber::registry()
//!     .with(tracing_subscriber::EnvFilter::new("claude_code_sdk=debug"))
//!     .with(tracing_subscriber::fmt::layer())
//!     .init();
//! ```

use std::pin::Pin;
use futures::Stream;
use tracing::{debug, info, instrument};

pub mod types;
pub mod errors;
pub mod config;
mod client;
mod transport;

pub use types::*;
pub use errors::*;
pub use config::*;

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
#[instrument(
    level = "info",
    skip(options),
    fields(
        prompt_length = prompt.len(),
        has_options = options.is_some(),
        system_prompt = options.as_ref().and_then(|o| o.system_prompt.as_deref()),
        allowed_tools = options.as_ref().map(|o| o.allowed_tools.len()),
        permission_mode = ?options.as_ref().and_then(|o| o.permission_mode.as_ref()),
        cwd = ?options.as_ref().and_then(|o| o.cwd.as_ref()),
    )
)]
pub async fn query(
    prompt: &str,
    options: Option<ClaudeCodeOptions>,
) -> Result<Pin<Box<dyn Stream<Item = Message> + Send>>, ClaudeSDKError> {
    info!("Starting Claude Code query");
    
    let options = options.unwrap_or_default();
    debug!(?options, "Using query options");
    
    // Set environment variable
    std::env::set_var("CLAUDE_CODE_ENTRYPOINT", "sdk-rust");
    debug!("Set CLAUDE_CODE_ENTRYPOINT environment variable");
    
    let client = InternalClient::new();
    info!("Created internal client");
    
    let stream = client.process_query(prompt, options).await?;
    info!("Successfully created message stream");
    
    Ok(Box::pin(stream))
}

/// Initialize default tracing subscriber for development and testing.
/// 
/// This is a convenience function that sets up a basic tracing subscriber
/// with environment filter support. For production use, you may want to
/// configure logging more specifically.
/// 
/// # Example
/// 
/// ```rust,no_run
/// claude_code_sdk::init_tracing();
/// ```
pub fn init_tracing() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("claude_code_sdk=info"));
    
    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().with_target(true))
        .init();
} 