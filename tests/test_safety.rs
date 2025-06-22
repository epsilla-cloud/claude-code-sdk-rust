use claude_code_sdk::{SafetyLimits, SafetyError};

#[test]
fn test_safety_limits_default() {
    let limits = SafetyLimits::default();
    
    assert_eq!(limits.max_line_size, 10 * 1024 * 1024); // 10MB
    assert_eq!(limits.max_text_block_size, 5 * 1024 * 1024); // 5MB
    assert_eq!(limits.max_buffer_size, 50 * 1024 * 1024); // 50MB
    assert_eq!(limits.max_log_preview_chars, 200);
    assert_eq!(limits.max_buffered_messages, 100);
    assert_eq!(limits.json_parse_timeout_ms, 5000);
}

#[test]
fn test_safety_limits_conservative() {
    let limits = SafetyLimits::conservative();
    
    assert_eq!(limits.max_line_size, 1024 * 1024); // 1MB
    assert_eq!(limits.max_text_block_size, 512 * 1024); // 512KB
    assert_eq!(limits.max_buffer_size, 10 * 1024 * 1024); // 10MB
    assert_eq!(limits.max_log_preview_chars, 100);
    assert_eq!(limits.max_buffered_messages, 50);
    assert_eq!(limits.json_parse_timeout_ms, 2000);
}

#[test]
fn test_safety_limits_generous() {
    let limits = SafetyLimits::generous();
    
    assert_eq!(limits.max_line_size, 50 * 1024 * 1024); // 50MB
    assert_eq!(limits.max_text_block_size, 25 * 1024 * 1024); // 25MB
    assert_eq!(limits.max_buffer_size, 200 * 1024 * 1024); // 200MB
    assert_eq!(limits.max_log_preview_chars, 500);
    assert_eq!(limits.max_buffered_messages, 200);
    assert_eq!(limits.json_parse_timeout_ms, 10000);
}

#[test]
fn test_line_size_safety_check() {
    let limits = SafetyLimits::conservative();
    
    // Safe sizes
    assert!(limits.is_line_size_safe(100));
    assert!(limits.is_line_size_safe(1024 * 1024)); // Exactly at limit
    
    // Unsafe sizes
    assert!(!limits.is_line_size_safe(1024 * 1024 + 1)); // Over limit
    assert!(!limits.is_line_size_safe(10 * 1024 * 1024)); // Way over limit
}

#[test]
fn test_text_block_safety_check() {
    let limits = SafetyLimits::conservative();
    
    // Safe sizes
    assert!(limits.is_text_block_safe(100));
    assert!(limits.is_text_block_safe(512 * 1024)); // Exactly at limit
    
    // Unsafe sizes
    assert!(!limits.is_text_block_safe(512 * 1024 + 1)); // Over limit
    assert!(!limits.is_text_block_safe(5 * 1024 * 1024)); // Way over limit
}

#[test]
fn test_safe_log_preview_short_text() {
    let limits = SafetyLimits::default();
    let short_text = "Hello, world!";
    
    let preview = limits.safe_log_preview(short_text);
    assert_eq!(preview, "Hello, world!");
}

#[test]
fn test_safe_log_preview_long_text() {
    let limits = SafetyLimits::default();
    let long_text = "A".repeat(500); // Longer than default preview limit
    
    let preview = limits.safe_log_preview(&long_text);
    assert!(preview.starts_with("A"));
    assert!(preview.contains("... (500 total chars)"));
    assert!(preview.len() < long_text.len());
}

#[test]
fn test_safe_log_preview_exact_limit() {
    let limits = SafetyLimits::default();
    let exact_text = "A".repeat(200); // Exactly at limit
    
    let preview = limits.safe_log_preview(&exact_text);
    assert_eq!(preview, exact_text); // Should not be truncated
}

#[test]
fn test_safety_error_line_too_large() {
    let error = SafetyError::LineTooLarge {
        actual: 2048,
        limit: 1024,
    };
    
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("2048"));
    assert!(error_msg.contains("1024"));
    assert!(error_msg.contains("exceeds limit"));
}

#[test]
fn test_safety_error_text_block_too_large() {
    let error = SafetyError::TextBlockTooLarge {
        actual: 1048576,
        limit: 524288,
    };
    
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("1048576"));
    assert!(error_msg.contains("524288"));
    assert!(error_msg.contains("exceeds limit"));
}

#[test]
fn test_safety_error_buffer_too_large() {
    let error = SafetyError::BufferTooLarge {
        actual: 104857600,
        limit: 52428800,
    };
    
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("104857600"));
    assert!(error_msg.contains("52428800"));
    assert!(error_msg.contains("exceeds limit"));
}

#[test]
fn test_safety_error_too_many_messages() {
    let error = SafetyError::TooManyMessages {
        actual: 150,
        limit: 100,
    };
    
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("150"));
    assert!(error_msg.contains("100"));
    assert!(error_msg.contains("Too many"));
}

#[test]
fn test_safety_error_parse_timeout() {
    let error = SafetyError::ParseTimeout {
        timeout_ms: 5000,
    };
    
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("5000"));
    assert!(error_msg.contains("timeout"));
}

#[test]
fn test_custom_safety_limits() {
    let custom = SafetyLimits {
        max_line_size: 2048,
        max_text_block_size: 1024,
        max_buffer_size: 4096,
        max_log_preview_chars: 50,
        max_buffered_messages: 10,
        json_parse_timeout_ms: 1000,
    };
    
    // Test line size checks
    assert!(custom.is_line_size_safe(2048));
    assert!(!custom.is_line_size_safe(2049));
    
    // Test text block checks
    assert!(custom.is_text_block_safe(1024));
    assert!(!custom.is_text_block_safe(1025));
    
    // Test log preview
    let long_text = "X".repeat(100);
    let preview = custom.safe_log_preview(&long_text);
    assert!(preview.contains("... (100 total chars)"));
}

#[test]
fn test_safety_limits_memory_estimation() {
    let limits = SafetyLimits::default();
    
    // Test realistic memory usage scenarios
    let small_response = 1000; // 1KB
    let medium_response = 100_000; // ~100KB
    let large_response = 1_000_000; // ~1MB
    let huge_response = 10_000_000; // ~10MB
    
    assert!(limits.is_text_block_safe(small_response));
    assert!(limits.is_text_block_safe(medium_response));
    assert!(limits.is_text_block_safe(large_response));
    assert!(!limits.is_text_block_safe(huge_response)); // Should exceed 5MB limit
}

#[test]
fn test_unicode_text_handling() {
    let limits = SafetyLimits::default();
    
    // Test with Unicode characters (emoji, Chinese, etc.)
    let unicode_text = "Hello 🌍! 你好世界! مرحبا بالعالم!";
    let preview = limits.safe_log_preview(unicode_text);
    
    // Should handle Unicode correctly
    assert_eq!(preview, unicode_text); // Short enough to not be truncated
    
    // Test with long Unicode text
    let long_unicode = "🚀".repeat(300); // Each emoji is multiple bytes
    let unicode_preview = limits.safe_log_preview(&long_unicode);
    assert!(unicode_preview.contains("total chars"));
    assert!(unicode_preview.len() < long_unicode.len());
}

#[test]
fn test_zero_limits_edge_case() {
    let zero_limits = SafetyLimits {
        max_line_size: 0,
        max_text_block_size: 0,
        max_buffer_size: 0,
        max_log_preview_chars: 0,
        max_buffered_messages: 0,
        json_parse_timeout_ms: 0,
    };
    
    // Everything should be unsafe with zero limits
    assert!(!zero_limits.is_line_size_safe(1));
    assert!(!zero_limits.is_text_block_safe(1));
    
    // Log preview should still work
    let preview = zero_limits.safe_log_preview("test");
    assert!(preview.contains("... (4 total chars)"));
} 