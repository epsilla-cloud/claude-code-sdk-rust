//! Configuration and safety limits for Claude SDK

/// Safety limits for text processing
#[derive(Debug, Clone)]
pub struct SafetyLimits {
    /// Maximum size for a single line from CLI output (bytes)
    pub max_line_size: usize,
    
    /// Maximum size for a single text content block (bytes)
    pub max_text_block_size: usize,
    
    /// Maximum total memory usage for buffered messages (bytes)
    pub max_buffer_size: usize,
    
    /// Maximum length for log previews (characters)
    pub max_log_preview_chars: usize,
    
    /// Maximum number of messages to buffer before applying backpressure
    pub max_buffered_messages: usize,
    
    /// Timeout for JSON parsing operations (milliseconds)
    pub json_parse_timeout_ms: u64,
}

impl Default for SafetyLimits {
    fn default() -> Self {
        Self {
            // 10MB per line - should handle most reasonable responses
            max_line_size: 10 * 1024 * 1024,
            
            // 5MB per text block - reasonable for code generation
            max_text_block_size: 5 * 1024 * 1024,
            
            // 50MB total buffer - prevents runaway memory usage
            max_buffer_size: 50 * 1024 * 1024,
            
            // 200 chars for log previews - enough for context
            max_log_preview_chars: 200,
            
            // 100 messages max - prevents queue buildup
            max_buffered_messages: 100,
            
            // 5 second timeout for JSON parsing
            json_parse_timeout_ms: 5000,
        }
    }
}

impl SafetyLimits {
    /// Create new safety limits with custom values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create conservative limits for memory-constrained environments
    pub fn conservative() -> Self {
        Self {
            max_line_size: 1024 * 1024,        // 1MB
            max_text_block_size: 512 * 1024,   // 512KB
            max_buffer_size: 10 * 1024 * 1024, // 10MB
            max_log_preview_chars: 100,
            max_buffered_messages: 50,
            json_parse_timeout_ms: 2000,
        }
    }
    
    /// Create generous limits for high-memory environments
    pub fn generous() -> Self {
        Self {
            max_line_size: 50 * 1024 * 1024,   // 50MB
            max_text_block_size: 25 * 1024 * 1024, // 25MB
            max_buffer_size: 200 * 1024 * 1024, // 200MB
            max_log_preview_chars: 500,
            max_buffered_messages: 200,
            json_parse_timeout_ms: 10000,
        }
    }
    
    /// Check if a line size is within limits
    pub fn is_line_size_safe(&self, size: usize) -> bool {
        size <= self.max_line_size
    }
    
    /// Check if a text block size is within limits
    pub fn is_text_block_safe(&self, size: usize) -> bool {
        size <= self.max_text_block_size
    }
    
    /// Get safe log preview for a string
    pub fn safe_log_preview(&self, text: &str) -> String {
        if text.len() <= self.max_log_preview_chars {
            text.to_string()
        } else {
            format!("{}... ({} total chars)", 
                text.chars().take(self.max_log_preview_chars).collect::<String>(),
                text.len())
        }
    }
}

/// Errors related to safety limit violations
#[derive(Debug, thiserror::Error)]
pub enum SafetyError {
    #[error("Line size {actual} bytes exceeds limit of {limit} bytes")]
    LineTooLarge { actual: usize, limit: usize },
    
    #[error("Text block size {actual} bytes exceeds limit of {limit} bytes")]
    TextBlockTooLarge { actual: usize, limit: usize },
    
    #[error("Buffer size {actual} bytes exceeds limit of {limit} bytes")]
    BufferTooLarge { actual: usize, limit: usize },
    
    #[error("Too many buffered messages: {actual}, limit: {limit}")]
    TooManyMessages { actual: usize, limit: usize },
    
    #[error("JSON parsing timeout after {timeout_ms}ms")]
    ParseTimeout { timeout_ms: u64 },
} 