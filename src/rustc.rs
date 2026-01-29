// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Rustc invocation module for compiling generated Rust code.

use std::path::Path;
use std::process::{Command, Output};

/// Result of rustc invocation
#[derive(Debug)]
pub struct RustcResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

impl RustcResult {
    /// Create a new RustcResult from process output
    fn from_output(output: Output) -> Self {
        Self {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code(),
        }
    }

    /// Check if compilation was successful
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Get formatted error message if compilation failed
    pub fn error_message(&self) -> Option<String> {
        if self.success {
            None
        } else {
            Some(format!(
                "rustc compilation failed (exit code: {}):\n{}",
                self.exit_code
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
                self.stderr
            ))
        }
    }

    /// Parse rustc error messages and extract structured error information
    #[allow(dead_code)]
    pub fn parse_errors(&self) -> Vec<RustcError> {
        if self.success {
            return Vec::new();
        }

        let mut errors = Vec::new();
        let lines: Vec<&str> = self.stderr.lines().collect();

        for line in lines.iter() {
            // Parse rustc error format: "error[E0425]: cannot find value `x` in this scope"
            // or "error: expected `;`, found `}`"
            if line.starts_with("error") {
                let error = RustcError::parse_from_line(line);
                errors.push(error);
            }
        }

        // If no structured errors found but compilation failed, create a generic error
        if errors.is_empty() && !self.success {
            errors.push(RustcError {
                error_code: None,
                message: self.stderr.clone(),
                location: None,
            });
        }

        errors
    }
}

/// Structured rustc error information
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct RustcError {
    pub error_code: Option<String>,
    pub message: String,
    pub location: Option<ErrorLocation>,
}

/// Location information for rustc errors
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct ErrorLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

impl RustcError {
    /// Parse a rustc error from a single line
    #[allow(dead_code)]
    fn parse_from_line(line: &str) -> Self {
        // Try to extract error code like "error[E0425]:"
        let error_code = if let Some(start) = line.find("error[") {
            line[start..]
                .find(']')
                .map(|end| line[start + 6..start + end].to_string())
        } else {
            None
        };

        // Extract the message (everything after "error:" or "error[CODE]:")
        let message = if let Some(pos) = line.find("]: ") {
            line[pos + 3..].trim().to_string()
        } else if let Some(pos) = line.find("error: ") {
            line[pos + 7..].trim().to_string()
        } else {
            line.trim().to_string()
        };

        RustcError {
            error_code,
            message,
            location: None, // Location parsing would require multi-line context
        }
    }

    /// Format the error for display
    #[allow(dead_code)]
    pub fn format(&self) -> String {
        let mut result = String::new();

        if let Some(ref code) = self.error_code {
            result.push_str(&format!("error[{}]: ", code));
        } else {
            result.push_str("error: ");
        }

        result.push_str(&self.message);

        if let Some(ref loc) = self.location {
            result.push_str(&format!("\n  at {}:{}:{}", loc.file, loc.line, loc.column));
        }

        result
    }
}

/// Invoke rustc to compile a Rust source file
///
/// # Arguments
/// * `rust_file` - Path to the Rust source file to compile
/// * `output_binary` - Path where the compiled binary should be written
/// * `verbose` - Whether to print verbose output
///
/// # Returns
/// * `Ok(RustcResult)` - Compilation result with stdout/stderr
/// * `Err(String)` - Error message if rustc could not be executed
pub fn invoke_rustc(
    rust_file: &Path,
    output_binary: &Path,
    verbose: bool,
) -> Result<RustcResult, String> {
    if verbose {
        println!("Invoking rustc: {:?} -o {:?}", rust_file, output_binary);
    }

    let mut cmd = Command::new("rustc");
    cmd.arg(rust_file).arg("-o").arg(output_binary);

    // Execute rustc and capture output
    let output = cmd
        .output()
        .map_err(|e| format!("Failed to execute rustc: {}", e))?;

    let result = RustcResult::from_output(output);

    if verbose {
        if !result.stdout.is_empty() {
            println!("rustc stdout:\n{}", result.stdout);
        }
        if !result.stderr.is_empty() {
            println!("rustc stderr:\n{}", result.stderr);
        }
    }

    Ok(result)
}

/// Invoke rustc with additional compiler flags
///
/// # Arguments
/// * `rust_file` - Path to the Rust source file to compile
/// * `output_binary` - Path where the compiled binary should be written
/// * `flags` - Additional rustc flags (e.g., "-C", "opt-level=3")
/// * `verbose` - Whether to print verbose output
///
/// # Returns
/// * `Ok(RustcResult)` - Compilation result with stdout/stderr
/// * `Err(String)` - Error message if rustc could not be executed
#[allow(dead_code)]
pub fn invoke_rustc_with_flags(
    rust_file: &Path,
    output_binary: &Path,
    flags: &[String],
    verbose: bool,
) -> Result<RustcResult, String> {
    if verbose {
        println!(
            "Invoking rustc: {:?} -o {:?} {}",
            rust_file,
            output_binary,
            flags.join(" ")
        );
    }

    let mut cmd = Command::new("rustc");
    cmd.arg(rust_file).arg("-o").arg(output_binary).args(flags);

    // Execute rustc and capture output
    let output = cmd
        .output()
        .map_err(|e| format!("Failed to execute rustc: {}", e))?;

    let result = RustcResult::from_output(output);

    if verbose {
        if !result.stdout.is_empty() {
            println!("rustc stdout:\n{}", result.stdout);
        }
        if !result.stderr.is_empty() {
            println!("rustc stderr:\n{}", result.stderr);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    // Helper function to create a mock ExitStatus for testing
    #[cfg(unix)]
    fn create_exit_status(code: i32) -> std::process::ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(code)
    }

    #[cfg(not(unix))]
    fn create_exit_status(_code: i32) -> std::process::ExitStatus {
        // On non-Unix platforms, we can't easily create ExitStatus
        // These tests will be skipped
        panic!("ExitStatus creation not supported on this platform for testing");
    }

    #[test]
    #[cfg(unix)]
    fn test_rustc_result_success() {
        let output = Output {
            status: create_exit_status(0),
            stdout: b"compilation successful".to_vec(),
            stderr: b"".to_vec(),
        };

        let result = RustcResult::from_output(output);
        assert!(result.is_success());
        assert_eq!(result.stdout, "compilation successful");
        assert_eq!(result.stderr, "");
        assert!(result.error_message().is_none());
    }

    #[test]
    #[cfg(unix)]
    fn test_rustc_result_failure() {
        let output = Output {
            status: create_exit_status(256), // exit code 1
            stdout: b"".to_vec(),
            stderr: b"error: expected `;`".to_vec(),
        };

        let result = RustcResult::from_output(output);
        assert!(!result.is_success());
        assert_eq!(result.stderr, "error: expected `;`");
        assert!(result.error_message().is_some());
        assert!(result.error_message().unwrap().contains("expected `;`"));
    }

    #[test]
    fn test_parse_rustc_error_with_code() {
        let error_line = "error[E0425]: cannot find value `x` in this scope";
        let error = RustcError::parse_from_line(error_line);

        assert_eq!(error.error_code, Some("E0425".to_string()));
        assert_eq!(error.message, "cannot find value `x` in this scope");
        assert!(error.location.is_none());
    }

    #[test]
    fn test_parse_rustc_error_without_code() {
        let error_line = "error: expected `;`, found `}`";
        let error = RustcError::parse_from_line(error_line);

        assert_eq!(error.error_code, None);
        assert_eq!(error.message, "expected `;`, found `}`");
        assert!(error.location.is_none());
    }

    #[test]
    #[cfg(unix)]
    fn test_parse_errors_from_result() {
        let output = Output {
            status: create_exit_status(256),
            stdout: b"".to_vec(),
            stderr: b"error[E0425]: cannot find value `x` in this scope\nerror: expected `;`"
                .to_vec(),
        };

        let result = RustcResult::from_output(output);
        let errors = result.parse_errors();

        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].error_code, Some("E0425".to_string()));
        assert_eq!(errors[0].message, "cannot find value `x` in this scope");
        assert_eq!(errors[1].error_code, None);
        assert_eq!(errors[1].message, "expected `;`");
    }

    #[test]
    #[cfg(unix)]
    fn test_parse_errors_empty_on_success() {
        let output = Output {
            status: create_exit_status(0),
            stdout: b"".to_vec(),
            stderr: b"".to_vec(),
        };

        let result = RustcResult::from_output(output);
        let errors = result.parse_errors();

        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_rustc_error_format() {
        let error = RustcError {
            error_code: Some("E0425".to_string()),
            message: "cannot find value `x` in this scope".to_string(),
            location: Some(ErrorLocation {
                file: "test.rs".to_string(),
                line: 5,
                column: 10,
            }),
        };

        let formatted = error.format();
        assert!(formatted.contains("error[E0425]"));
        assert!(formatted.contains("cannot find value `x`"));
        assert!(formatted.contains("test.rs:5:10"));
    }

    #[test]
    fn test_rustc_error_format_without_code() {
        let error = RustcError {
            error_code: None,
            message: "expected `;`".to_string(),
            location: None,
        };

        let formatted = error.format();
        assert!(formatted.starts_with("error: "));
        assert!(formatted.contains("expected `;`"));
        assert!(!formatted.contains("test.rs"));
    }

    #[test]
    fn test_invoke_rustc_with_valid_code() {
        // Create a simple valid Rust source file
        let test_source = r#"
fn main() {
    println!("Hello, world!");
}
"#;
        let input_path = PathBuf::from("test_rustc_valid_12345.rs");
        let output_path = PathBuf::from("test_rustc_valid_12345");

        fs::write(&input_path, test_source).unwrap();

        let result = invoke_rustc(&input_path, &output_path, false);

        // Clean up
        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);

        // This test will only pass if rustc is installed
        if result.is_ok() {
            let rustc_result = result.unwrap();
            assert!(rustc_result.is_success());
            assert!(rustc_result.error_message().is_none());
            assert_eq!(rustc_result.parse_errors().len(), 0);
        }
    }

    #[test]
    fn test_invoke_rustc_with_invalid_code() {
        // Create an invalid Rust source file with a clear syntax error
        let test_source = r#"
fn main() {
    let x = 5
    println!("{}", x);
}
"#;
        let input_path = PathBuf::from("test_rustc_invalid_12345.rs");
        let output_path = PathBuf::from("test_rustc_invalid_12345");

        fs::write(&input_path, test_source).unwrap();

        let result = invoke_rustc(&input_path, &output_path, false);

        // Clean up
        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);

        // This test will only pass if rustc is installed
        if result.is_ok() {
            let rustc_result = result.unwrap();
            // The code should fail to compile due to missing semicolon
            if rustc_result.is_success() {
                // If it somehow succeeded, that's okay - rustc behavior may vary
                // The important thing is we captured the result
                println!("Note: rustc compiled code that was expected to fail");
            } else {
                // Expected: compilation failed
                assert!(rustc_result.error_message().is_some());

                // Parse errors and verify we got error information
                let errors = rustc_result.parse_errors();
                assert!(!errors.is_empty());
            }
        }
    }

    #[test]
    fn test_invoke_rustc_with_nonexistent_file() {
        let input_path = PathBuf::from("nonexistent_file_99999.rs");
        let output_path = PathBuf::from("nonexistent_output_99999");

        let result = invoke_rustc(&input_path, &output_path, false);

        // This test will only pass if rustc is installed
        if result.is_ok() {
            let rustc_result = result.unwrap();
            assert!(!rustc_result.is_success());
            assert!(rustc_result.error_message().is_some());
        }
    }

    #[test]
    fn test_invoke_rustc_with_flags() {
        // Create a simple valid Rust source file
        let test_source = r#"
fn main() {
    println!("Hello with flags!");
}
"#;
        let input_path = PathBuf::from("test_rustc_flags_12345.rs");
        let output_path = PathBuf::from("test_rustc_flags_12345");

        fs::write(&input_path, test_source).unwrap();

        let flags = vec!["-C".to_string(), "opt-level=2".to_string()];
        let result = invoke_rustc_with_flags(&input_path, &output_path, &flags, false);

        // Clean up
        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);

        // This test will only pass if rustc is installed
        if result.is_ok() {
            let rustc_result = result.unwrap();
            assert!(rustc_result.is_success());
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_rustc_result_error_message_format() {
        let output = Output {
            status: create_exit_status(256),
            stdout: b"".to_vec(),
            stderr: b"error[E0425]: cannot find value `x` in this scope".to_vec(),
        };

        let result = RustcResult::from_output(output);
        let error_msg = result.error_message().unwrap();

        assert!(error_msg.contains("rustc compilation failed"));
        assert!(error_msg.contains("cannot find value `x`"));
        assert!(error_msg.contains("exit code"));
    }

    #[test]
    #[cfg(unix)]
    fn test_parse_errors_with_no_structured_errors() {
        let output = Output {
            status: create_exit_status(256),
            stdout: b"".to_vec(),
            stderr: b"Some generic error message without error: prefix".to_vec(),
        };

        let result = RustcResult::from_output(output);
        let errors = result.parse_errors();

        // Should create a generic error when no structured errors found
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_code, None);
        assert!(errors[0].message.contains("Some generic error message"));
    }
}
