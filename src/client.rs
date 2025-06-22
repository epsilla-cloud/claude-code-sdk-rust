//! Internal client implementation.

use futures::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

use crate::{
    errors::ClaudeSDKError,
    transport::{subprocess_cli::SubprocessCLITransport, Transport},
    types::*,
};

/// Internal client implementation
pub struct InternalClient;

impl InternalClient {
    /// Create a new internal client
    pub fn new() -> Self {
        Self
    }

    /// Process a query through transport
    pub async fn process_query(
        &self,
        prompt: &str,
        options: ClaudeCodeOptions,
    ) -> Result<Pin<Box<dyn Stream<Item = Message> + Send>>, ClaudeSDKError> {
        let transport = SubprocessCLITransport::new(prompt, options, None)?;

        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let stream = ReceiverStream::new(rx);

        tokio::spawn(async move {
            let mut transport = transport;
            
            if let Err(e) = transport.connect().await {
                let _ = tx.send(Message::Result(ResultMessage {
                    subtype: "error".to_string(),
                    duration_ms: 0,
                    duration_api_ms: 0,
                    is_error: true,
                    num_turns: 0,
                    session_id: "error".to_string(),
                    total_cost_usd: None,
                    usage: None,
                    result: Some(format!("Connection error: {}", e)),
                })).await;
                return;
            }

            {
                let mut message_stream = transport.receive_messages();
                
                while let Some(data_result) = message_stream.next().await {
                    match data_result {
                        Ok(data) => {
                            if let Some(message) = Self::parse_message(data) {
                                if tx.send(message).await.is_err() {
                                    break; // Receiver dropped
                                }
                            }
                        }
                        Err(e) => {
                            let error_message = Message::Result(ResultMessage {
                                subtype: "error".to_string(),
                                duration_ms: 0,
                                duration_api_ms: 0,
                                is_error: true,
                                num_turns: 0,
                                session_id: "error".to_string(),
                                total_cost_usd: None,
                                usage: None,
                                result: Some(format!("Stream error: {}", e)),
                            });
                            let _ = tx.send(error_message).await;
                            break;
                        }
                    }
                }
            } // Drop message_stream here

            let _ = transport.disconnect().await;
        });

        Ok(Box::pin(stream))
    }

    /// Parse message from CLI output, trusting the structure
    fn parse_message(data: HashMap<String, serde_json::Value>) -> Option<Message> {
        let message_type = data.get("type")?.as_str()?;

        match message_type {
            "user" => {
                let content = data
                    .get("message")?
                    .get("content")?
                    .as_str()?
                    .to_string();
                Some(Message::User(UserMessage { content }))
            }
            "assistant" => {
                let message_data = data.get("message")?;
                let content_array = message_data.get("content")?.as_array()?;
                
                let mut content_blocks = Vec::new();
                for block in content_array {
                    let block_type = block.get("type")?.as_str()?;
                    
                    match block_type {
                        "text" => {
                            let text = block.get("text")?.as_str()?.to_string();
                            content_blocks.push(ContentBlock::Text(TextBlock { text }));
                        }
                        "tool_use" => {
                            let id = block.get("id")?.as_str()?.to_string();
                            let name = block.get("name")?.as_str()?.to_string();
                            let input = block.get("input")?.as_object()?.clone();
                            let input_map: HashMap<String, serde_json::Value> = 
                                input.into_iter().collect();
                            
                            content_blocks.push(ContentBlock::ToolUse(ToolUseBlock {
                                id,
                                name,
                                input: input_map,
                            }));
                        }
                        "tool_result" => {
                            let tool_use_id = block.get("tool_use_id")?.as_str()?.to_string();
                            let content = block.get("content").and_then(|c| {
                                if let Some(s) = c.as_str() {
                                    Some(ToolResultContent::Text(s.to_string()))
                                } else if let Some(arr) = c.as_array() {
                                    let structured: Option<Vec<HashMap<String, serde_json::Value>>> = 
                                        arr.iter()
                                           .map(|v| v.as_object().map(|o| o.clone().into_iter().collect()))
                                           .collect();
                                    structured.map(ToolResultContent::Structured)
                                } else {
                                    None
                                }
                            });
                            let is_error = block.get("is_error").and_then(|v| v.as_bool());
                            
                            content_blocks.push(ContentBlock::ToolResult(ToolResultBlock {
                                tool_use_id,
                                content,
                                is_error,
                            }));
                        }
                        _ => continue,
                    }
                }
                
                Some(Message::Assistant(AssistantMessage {
                    content: content_blocks,
                }))
            }
            "system" => {
                let subtype = data.get("subtype")?.as_str()?.to_string();
                let data_map: HashMap<String, serde_json::Value> = data.into_iter().collect();
                
                Some(Message::System(SystemMessage {
                    subtype,
                    data: data_map,
                }))
            }
            "result" => {
                let subtype = data.get("subtype")?.as_str()?.to_string();
                let duration_ms = data.get("duration_ms")?.as_u64()?;
                let duration_api_ms = data.get("duration_api_ms")?.as_u64()?;
                let is_error = data.get("is_error")?.as_bool()?;
                let num_turns = data.get("num_turns")?.as_u64()? as u32;
                let session_id = data.get("session_id")?.as_str()?.to_string();
                let total_cost_usd = data.get("total_cost_usd").and_then(|v| v.as_f64());
                let usage = data.get("usage").and_then(|v| {
                    v.as_object().map(|o| o.clone().into_iter().collect())
                });
                let result = data.get("result").and_then(|v| v.as_str().map(|s| s.to_string()));
                
                Some(Message::Result(ResultMessage {
                    subtype,
                    duration_ms,
                    duration_api_ms,
                    is_error,
                    num_turns,
                    session_id,
                    total_cost_usd,
                    usage,
                    result,
                }))
            }
            _ => None,
        }
    }
} 