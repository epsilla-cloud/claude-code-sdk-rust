//! Subprocess transport implementation using Claude Code CLI.

use futures::Stream;
use std::collections::HashMap;
use std::path::PathBuf;
use std::pin::Pin;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio_stream::{wrappers::LinesStream, StreamExt};
use tracing::{debug, error, info, warn, instrument};

use crate::{
    errors::*,
    types::{ClaudeCodeOptions, PermissionMode},
    transport::Transport,
    SafetyLimits, SafetyError,
};

/// Subprocess transport using Claude Code CLI
pub struct SubprocessCLITransport {
    prompt: String,
    options: ClaudeCodeOptions,
    cli_path: String,
    cwd: Option<PathBuf>,
    process: Option<Child>,
    safety_limits: SafetyLimits,
    current_buffer_size: usize,
}

impl SubprocessCLITransport {
    /// Create a new subprocess transport
    #[instrument(level = "debug", skip(prompt, options))]
    pub fn new(
        prompt: &str,
        options: ClaudeCodeOptions,
        cli_path: Option<&str>,
    ) -> Result<Self, ClaudeSDKError> {
        info!("Creating new subprocess CLI transport");
        debug!(
            prompt_length = prompt.len(),
            cli_path = cli_path,
            cwd = ?options.cwd,
            "Transport configuration"
        );

        let cli_path = if let Some(path) = cli_path {
            debug!(provided_path = path, "Using provided CLI path");
            path.to_string()
        } else {
            debug!("Searching for CLI path");
            Self::find_cli()?
        };
        
        let cwd = options.cwd.clone();
        
        info!(cli_path = %cli_path, "Successfully created subprocess transport");
        Ok(Self {
            prompt: prompt.to_string(),
            options,
            cli_path,
            cwd,
            process: None,
            safety_limits: SafetyLimits::default(),
            current_buffer_size: 0,
        })
    }
    
    /// Set custom safety limits for this transport
    pub fn with_safety_limits(mut self, limits: SafetyLimits) -> Self {
        info!(?limits, "Setting custom safety limits");
        self.safety_limits = limits;
        self
    }

    /// Find Claude Code CLI binary
    #[instrument(level = "debug")]
    fn find_cli() -> Result<String, ClaudeSDKError> {
        debug!("Searching for Claude Code CLI binary");
        
        // Check if claude is in PATH
        debug!("Checking PATH for 'claude' executable");
        if let Ok(path) = which::which("claude") {
            let path_str = path.to_string_lossy().to_string();
            info!(path = %path_str, "Found Claude CLI in PATH");
            return Ok(path_str);
        }
        debug!("Claude CLI not found in PATH");

        // Check common locations
        let home_dir = home::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        debug!(home_dir = %home_dir.display(), "Using home directory");
        
        let locations = vec![
            home_dir.join(".npm-global/bin/claude"),
            PathBuf::from("/usr/local/bin/claude"),
            home_dir.join(".local/bin/claude"),
            home_dir.join("node_modules/.bin/claude"),
            home_dir.join(".yarn/bin/claude"),
        ];

        debug!(locations_count = locations.len(), "Checking common installation locations");
        for path in &locations {
            debug!(path = %path.display(), "Checking location");
            if path.exists() && path.is_file() {
                let path_str = path.to_string_lossy().to_string();
                info!(path = %path_str, "Found Claude CLI at common location");
                return Ok(path_str);
            }
        }
        debug!("Claude CLI not found in common locations");

        // Check if Node.js is installed
        debug!("Checking if Node.js is available");
        let node_installed = which::which("node").is_ok();

        if !node_installed {
            error!("Node.js is not installed");
            let error_msg = "Claude Code requires Node.js, which is not installed.\n\n\
                           Install Node.js from: https://nodejs.org/\n\
                           \nAfter installing Node.js, install Claude Code:\n\
                           npm install -g @anthropic-ai/claude-code";
            return Err(ClaudeSDKError::CLINotFound(CLINotFoundError::new(error_msg)));
        }
        debug!("Node.js is available");

        error!("Claude Code CLI not found in any location");
        let error_msg = "Claude Code not found. Install with:\n\
                        npm install -g @anthropic-ai/claude-code\n\
                        \nIf already installed locally, try:\n\
                        export PATH=\"$HOME/node_modules/.bin:$PATH\"\n\
                        \nOr specify the path when creating transport";
        Err(ClaudeSDKError::CLINotFound(CLINotFoundError::new(error_msg)))
    }

    /// Build CLI command with arguments
    #[instrument(level = "trace", skip(self))]
    fn build_command(&self) -> Vec<String> {
        debug!("Building CLI command with arguments");
        let mut cmd = vec![
            self.cli_path.clone(),
            "--output-format".to_string(),
            "stream-json".to_string(),
            "--verbose".to_string(),
        ];

        if let Some(system_prompt) = &self.options.system_prompt {
            debug!(system_prompt_length = system_prompt.len(), "Adding system prompt");
            cmd.extend(["--system-prompt".to_string(), system_prompt.clone()]);
        }

        if let Some(append_system_prompt) = &self.options.append_system_prompt {
            debug!(append_system_prompt_length = append_system_prompt.len(), "Adding append system prompt");
            cmd.extend(["--append-system-prompt".to_string(), append_system_prompt.clone()]);
        }

        if !self.options.allowed_tools.is_empty() {
            debug!(allowed_tools = ?self.options.allowed_tools, "Adding allowed tools");
            cmd.extend([
                "--allowedTools".to_string(),
                self.options.allowed_tools.join(","),
            ]);
        }

        if let Some(max_turns) = self.options.max_turns {
            debug!(max_turns, "Adding max turns limit");
            cmd.extend(["--max-turns".to_string(), max_turns.to_string()]);
        }

        if !self.options.disallowed_tools.is_empty() {
            debug!(disallowed_tools = ?self.options.disallowed_tools, "Adding disallowed tools");
            cmd.extend([
                "--disallowedTools".to_string(),
                self.options.disallowed_tools.join(","),
            ]);
        }

        if let Some(model) = &self.options.model {
            debug!(model = %model, "Adding model specification");
            cmd.extend(["--model".to_string(), model.clone()]);
        }

        if let Some(permission_prompt_tool_name) = &self.options.permission_prompt_tool_name {
            debug!(tool_name = %permission_prompt_tool_name, "Adding permission prompt tool");
            cmd.extend([
                "--permission-prompt-tool".to_string(),
                permission_prompt_tool_name.clone(),
            ]);
        }

        if let Some(permission_mode) = &self.options.permission_mode {
            let mode_str = match permission_mode {
                PermissionMode::Default => "default",
                PermissionMode::AcceptEdits => "acceptEdits",
                PermissionMode::BypassPermissions => "bypassPermissions",
            };
            debug!(permission_mode = mode_str, "Adding permission mode");
            cmd.extend(["--permission-mode".to_string(), mode_str.to_string()]);
        }

        if self.options.continue_conversation {
            debug!("Adding continue conversation flag");
            cmd.push("--continue".to_string());
        }

        if let Some(resume) = &self.options.resume {
            debug!(resume = %resume, "Adding resume option");
            cmd.extend(["--resume".to_string(), resume.clone()]);
        }

        if !self.options.mcp_servers.is_empty() {
            debug!(mcp_servers_count = self.options.mcp_servers.len(), "Adding MCP servers configuration");
            let mcp_config = serde_json::json!({
                "mcpServers": self.options.mcp_servers
            });
            cmd.extend([
                "--mcp-config".to_string(),
                mcp_config.to_string(),
            ]);
        }

        cmd.extend(["--print".to_string(), self.prompt.clone()]);
        debug!(total_args = cmd.len(), "Built complete CLI command");
        cmd
    }
}

#[async_trait::async_trait]
impl Transport for SubprocessCLITransport {
    /// Start subprocess
    #[instrument(level = "info", skip(self))]
    async fn connect(&mut self) -> Result<(), ClaudeSDKError> {
        if self.process.is_some() {
            debug!("Process already connected, skipping connection");
            return Ok(());
        }

        info!("Starting Claude CLI subprocess");
        let cmd_args = self.build_command();
        debug!(args_count = cmd_args.len(), "Built command arguments");

        let mut command = Command::new(&cmd_args[0]);
        command
            .args(&cmd_args[1..])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("CLAUDE_CODE_ENTRYPOINT", "sdk-rust");

        if let Some(cwd) = &self.cwd {
            debug!(cwd = %cwd.display(), "Setting working directory");
            command.current_dir(cwd);
        }

        debug!("Spawning subprocess");
        let process = command.spawn().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                error!(
                    error = %e,
                    cli_path = %self.cli_path,
                    "Claude Code CLI not found"
                );
                ClaudeSDKError::CLINotFound(CLINotFoundError::with_path(
                    "Claude Code not found at",
                    &self.cli_path,
                ))
            } else {
                error!(error = %e, "Failed to spawn Claude Code subprocess");
                ClaudeSDKError::CLIConnection(CLIConnectionError::new(format!(
                    "Failed to start Claude Code: {}",
                    e
                )))
            }
        })?;

        info!(pid = process.id(), "Successfully started Claude CLI subprocess");
        self.process = Some(process);
        Ok(())
    }

    /// Terminate subprocess
    #[instrument(level = "info", skip(self))]
    async fn disconnect(&mut self) -> Result<(), ClaudeSDKError> {
        if let Some(mut process) = self.process.take() {
            info!(pid = process.id(), "Disconnecting from Claude CLI subprocess");
            
            // Check if process has already finished
            if let Ok(Some(status)) = process.try_wait() {
                if status.success() {
                    info!("Process already finished successfully");
                } else {
                    warn!(exit_code = status.code(), "Process already finished with error");
                }
                return Ok(());
            }

            // Try graceful termination
            debug!("Killing subprocess");
            if let Err(e) = process.kill().await {
                warn!(error = %e, "Failed to kill subprocess (might have already exited)");
            }
            
            debug!("Waiting for subprocess to exit");
            match process.wait().await {
                Ok(status) => {
                    if status.success() {
                        info!("Subprocess terminated successfully");
                    } else {
                        warn!(exit_code = status.code(), "Subprocess terminated with error");
                    }
                }
                Err(e) => {
                    warn!(error = %e, "Error waiting for subprocess to terminate");
                }
            }
        } else {
            debug!("No active subprocess to disconnect");
        }
        Ok(())
    }

    /// Receive messages from CLI
    #[instrument(level = "debug", skip(self))]
    fn receive_messages(&mut self) -> Pin<Box<dyn Stream<Item = Result<HashMap<String, serde_json::Value>, ClaudeSDKError>> + Send + '_>> {
        if let Some(process) = &mut self.process {
            if let Some(stdout) = process.stdout.take() {
                debug!("Setting up message stream from subprocess stdout");
                let reader = BufReader::new(stdout);
                let lines_stream = LinesStream::new(reader.lines());
                
                let safety_limits = self.safety_limits.clone();
                let stream = lines_stream.map(move |line_result| {
                    match line_result {
                        Ok(line) => {
                            let line = line.trim();
                            if line.is_empty() {
                                debug!("Skipping empty line");
                                return Err(ClaudeSDKError::Other("Empty line".to_string()));
                            }
                            
                            // Safety check: line size
                            let line_size = line.len();
                            if !safety_limits.is_line_size_safe(line_size) {
                                error!(
                                    line_size = line_size,
                                    limit = safety_limits.max_line_size,
                                    "Line exceeds safety limit"
                                );
                                return Err(ClaudeSDKError::Safety(SafetyError::LineTooLarge {
                                    actual: line_size,
                                    limit: safety_limits.max_line_size,
                                }));
                            }
                            
                            debug!(line_length = line_size, "Processing line from subprocess");
                            
                            // Safe JSON parsing with timeout simulation
                            let parse_start = std::time::Instant::now();
                            let parse_result = serde_json::from_str::<HashMap<String, serde_json::Value>>(line);
                            let parse_duration = parse_start.elapsed();
                            
                            if parse_duration.as_millis() > safety_limits.json_parse_timeout_ms as u128 {
                                warn!(
                                    duration_ms = parse_duration.as_millis(),
                                    timeout_ms = safety_limits.json_parse_timeout_ms,
                                    "JSON parsing took longer than expected"
                                );
                            }
                            
                            match parse_result {
                                Ok(data) => {
                                    debug!(
                                        fields_count = data.len(),
                                        parse_duration_ms = parse_duration.as_millis(),
                                        "Successfully parsed JSON message"
                                    );
                                    
                                    // Check if this contains large text content
                                    if let Some(message_obj) = data.get("message") {
                                        if let Some(content_arr) = message_obj.get("content").and_then(|c| c.as_array()) {
                                            for content_item in content_arr {
                                                if let Some(text) = content_item.get("text").and_then(|t| t.as_str()) {
                                                    let text_size = text.len();
                                                    if !safety_limits.is_text_block_safe(text_size) {
                                                        warn!(
                                                            text_size = text_size,
                                                            limit = safety_limits.max_text_block_size,
                                                            text_preview = %safety_limits.safe_log_preview(text),
                                                            "Large text block detected"
                                                        );
                                                        // Don't fail, but log the warning
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    
                                    Ok(data)
                                }
                                Err(e) => {
                                    if line.starts_with('{') || line.starts_with('[') {
                                        error!(
                                            error = %e,
                                            line_preview = %safety_limits.safe_log_preview(line),
                                            parse_duration_ms = parse_duration.as_millis(),
                                            "Failed to parse JSON message"
                                        );
                                        Err(ClaudeSDKError::CLIJSONDecode(CLIJSONDecodeError::new(line, e)))
                                    } else {
                                        debug!(
                                            line_preview = %safety_limits.safe_log_preview(line),
                                            "Skipping non-JSON line"
                                        );
                                        Err(ClaudeSDKError::Other("Non-JSON line".to_string()))
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!(error = %e, "Error reading line from subprocess stdout");
                            Err(ClaudeSDKError::Io(e))
                        }
                    }
                }).filter_map(|result| match result {
                    Ok(data) => Some(Ok(data)),
                    Err(ClaudeSDKError::Other(_)) => None, // Skip non-JSON lines
                    Err(e) => Some(Err(e)),
                });
                
                return Box::pin(stream);
            } else {
                warn!("No stdout available from subprocess");
            }
        } else {
            warn!("No active subprocess to receive messages from");
        }
        
        // Return empty stream if no process or stdout
        debug!("Returning empty message stream");
        Box::pin(tokio_stream::empty())
    }

    /// Check if subprocess is running
    #[instrument(level = "trace", skip(self))]
    fn is_connected(&self) -> bool {
        let is_connected = if let Some(_process) = &self.process {
            // We can't call try_wait on an immutable reference
            // For now, just assume connected if process exists
            // In a real implementation, we'd need better state tracking
            true
        } else {
            false
        };
        debug!(is_connected, "Checked connection status");
        is_connected
    }
} 