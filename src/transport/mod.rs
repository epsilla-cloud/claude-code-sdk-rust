//! Transport implementations for Claude SDK.

use futures::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use crate::errors::ClaudeSDKError;

pub mod subprocess_cli;

/// Abstract transport for Claude communication
#[async_trait::async_trait]
pub trait Transport {
    /// Initialize connection
    async fn connect(&mut self) -> Result<(), ClaudeSDKError>;
    
    /// Close connection
    async fn disconnect(&mut self) -> Result<(), ClaudeSDKError>;
    
    /// Send request to Claude (not used for CLI transport - args passed via command line)
    #[allow(dead_code)]
    async fn send_request(
        &mut self,
        _messages: Vec<HashMap<String, serde_json::Value>>,
        _options: HashMap<String, serde_json::Value>,
    ) -> Result<(), ClaudeSDKError> {
        Ok(())
    }
    
    /// Receive messages from Claude
    fn receive_messages(&mut self) -> Pin<Box<dyn Stream<Item = Result<HashMap<String, serde_json::Value>, ClaudeSDKError>> + Send + '_>>;
    
    /// Check if transport is connected
    #[allow(dead_code)]
    fn is_connected(&self) -> bool;
} 