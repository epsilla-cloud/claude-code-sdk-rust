//! Subprocess transport implementation using Claude Code CLI.

use futures::Stream;
use std::collections::HashMap;
use std::path::PathBuf;
use std::pin::Pin;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio_stream::{wrappers::LinesStream, StreamExt};

use crate::{
    errors::*,
    types::{ClaudeCodeOptions, PermissionMode},
    transport::Transport,
};

/// Subprocess transport using Claude Code CLI
pub struct SubprocessCLITransport {
    prompt: String,
    options: ClaudeCodeOptions,
    cli_path: String,
    cwd: Option<PathBuf>,
    process: Option<Child>,
}

impl SubprocessCLITransport {
    /// Create a new subprocess transport
    pub fn new(
        prompt: &str,
        options: ClaudeCodeOptions,
        cli_path: Option<&str>,
    ) -> Result<Self, ClaudeSDKError> {
        let cli_path = if let Some(path) = cli_path {
            path.to_string()
        } else {
            Self::find_cli()?
        };
        
        let cwd = options.cwd.clone();
        
        Ok(Self {
            prompt: prompt.to_string(),
            options,
            cli_path,
            cwd,
            process: None,
        })
    }

    /// Find Claude Code CLI binary
    fn find_cli() -> Result<String, ClaudeSDKError> {
        // Check if claude is in PATH
        if let Ok(path) = which::which("claude") {
            return Ok(path.to_string_lossy().to_string());
        }

        // Check common locations
        let home_dir = home::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let locations = vec![
            home_dir.join(".npm-global/bin/claude"),
            PathBuf::from("/usr/local/bin/claude"),
            home_dir.join(".local/bin/claude"),
            home_dir.join("node_modules/.bin/claude"),
            home_dir.join(".yarn/bin/claude"),
        ];

        for path in locations {
            if path.exists() && path.is_file() {
                return Ok(path.to_string_lossy().to_string());
            }
        }

        // Check if Node.js is installed
        let node_installed = which::which("node").is_ok();

        if !node_installed {
            let error_msg = "Claude Code requires Node.js, which is not installed.\n\n\
                           Install Node.js from: https://nodejs.org/\n\
                           \nAfter installing Node.js, install Claude Code:\n\
                           npm install -g @anthropic-ai/claude-code";
            return Err(ClaudeSDKError::CLINotFound(CLINotFoundError::new(error_msg)));
        }

        let error_msg = "Claude Code not found. Install with:\n\
                        npm install -g @anthropic-ai/claude-code\n\
                        \nIf already installed locally, try:\n\
                        export PATH=\"$HOME/node_modules/.bin:$PATH\"\n\
                        \nOr specify the path when creating transport";
        Err(ClaudeSDKError::CLINotFound(CLINotFoundError::new(error_msg)))
    }

    /// Build CLI command with arguments
    fn build_command(&self) -> Vec<String> {
        let mut cmd = vec![
            self.cli_path.clone(),
            "--output-format".to_string(),
            "stream-json".to_string(),
            "--verbose".to_string(),
        ];

        if let Some(system_prompt) = &self.options.system_prompt {
            cmd.extend(["--system-prompt".to_string(), system_prompt.clone()]);
        }

        if let Some(append_system_prompt) = &self.options.append_system_prompt {
            cmd.extend(["--append-system-prompt".to_string(), append_system_prompt.clone()]);
        }

        if !self.options.allowed_tools.is_empty() {
            cmd.extend([
                "--allowedTools".to_string(),
                self.options.allowed_tools.join(","),
            ]);
        }

        if let Some(max_turns) = self.options.max_turns {
            cmd.extend(["--max-turns".to_string(), max_turns.to_string()]);
        }

        if !self.options.disallowed_tools.is_empty() {
            cmd.extend([
                "--disallowedTools".to_string(),
                self.options.disallowed_tools.join(","),
            ]);
        }

        if let Some(model) = &self.options.model {
            cmd.extend(["--model".to_string(), model.clone()]);
        }

        if let Some(permission_prompt_tool_name) = &self.options.permission_prompt_tool_name {
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
            cmd.extend(["--permission-mode".to_string(), mode_str.to_string()]);
        }

        if self.options.continue_conversation {
            cmd.push("--continue".to_string());
        }

        if let Some(resume) = &self.options.resume {
            cmd.extend(["--resume".to_string(), resume.clone()]);
        }

        if !self.options.mcp_servers.is_empty() {
            let mcp_config = serde_json::json!({
                "mcpServers": self.options.mcp_servers
            });
            cmd.extend([
                "--mcp-config".to_string(),
                mcp_config.to_string(),
            ]);
        }

        cmd.extend(["--print".to_string(), self.prompt.clone()]);
        cmd
    }
}

#[async_trait::async_trait]
impl Transport for SubprocessCLITransport {
    /// Start subprocess
    async fn connect(&mut self) -> Result<(), ClaudeSDKError> {
        if self.process.is_some() {
            return Ok(());
        }

        let cmd_args = self.build_command();
        let mut command = Command::new(&cmd_args[0]);
        command
            .args(&cmd_args[1..])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("CLAUDE_CODE_ENTRYPOINT", "sdk-rust");

        if let Some(cwd) = &self.cwd {
            command.current_dir(cwd);
        }

        let process = command.spawn().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ClaudeSDKError::CLINotFound(CLINotFoundError::with_path(
                    "Claude Code not found at",
                    &self.cli_path,
                ))
            } else {
                ClaudeSDKError::CLIConnection(CLIConnectionError::new(format!(
                    "Failed to start Claude Code: {}",
                    e
                )))
            }
        })?;

        self.process = Some(process);
        Ok(())
    }

    /// Terminate subprocess
    async fn disconnect(&mut self) -> Result<(), ClaudeSDKError> {
        if let Some(mut process) = self.process.take() {
            if let Ok(Some(_)) = process.try_wait() {
                return Ok(()); // Already finished
            }

            // Try graceful termination
            if let Err(_) = process.kill().await {
                // Process might have already exited
            }
            
            let _ = process.wait().await;
        }
        Ok(())
    }

    /// Receive messages from CLI
    fn receive_messages(&mut self) -> Pin<Box<dyn Stream<Item = Result<HashMap<String, serde_json::Value>, ClaudeSDKError>> + Send + '_>> {
        if let Some(process) = &mut self.process {
            if let Some(stdout) = process.stdout.take() {
                let reader = BufReader::new(stdout);
                let lines_stream = LinesStream::new(reader.lines());
                
                let stream = lines_stream.map(|line_result| {
                    match line_result {
                        Ok(line) => {
                            let line = line.trim();
                            if line.is_empty() {
                                return Err(ClaudeSDKError::Other("Empty line".to_string()));
                            }
                            
                            match serde_json::from_str::<HashMap<String, serde_json::Value>>(line) {
                                Ok(data) => Ok(data),
                                Err(e) => {
                                    if line.starts_with('{') || line.starts_with('[') {
                                        Err(ClaudeSDKError::CLIJSONDecode(CLIJSONDecodeError::new(line, e)))
                                    } else {
                                        Err(ClaudeSDKError::Other("Non-JSON line".to_string()))
                                    }
                                }
                            }
                        }
                        Err(e) => Err(ClaudeSDKError::Io(e)),
                    }
                }).filter_map(|result| match result {
                    Ok(data) => Some(Ok(data)),
                    Err(ClaudeSDKError::Other(_)) => None, // Skip non-JSON lines
                    Err(e) => Some(Err(e)),
                });
                
                return Box::pin(stream);
            }
        }
        
        // Return empty stream if no process or stdout
        Box::pin(tokio_stream::empty())
    }

    /// Check if subprocess is running
    fn is_connected(&self) -> bool {
        if let Some(_process) = &self.process {
            // We can't call try_wait on an immutable reference
            // For now, just assume connected if process exists
            // In a real implementation, we'd need better state tracking
            true
        } else {
            false
        }
    }
} 