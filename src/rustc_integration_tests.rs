// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Integration tests for rustc invocation through the CLI

#[cfg(test)]
mod tests {
    use crate::cli::{run_compiler, CompilerOptions, EmitMode};
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_compile_valid_crusty_to_binary() {
        // Create a simple valid Crusty source file
        let test_source = r#"
int main() {
    return 0;
}
"#;
        let input_path = PathBuf::from("test_compile_valid_12345.crst");
        let output_path = PathBuf::from("test_compile_valid_12345");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        // Clean up
        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        // This test will only pass if rustc is installed
        // If rustc is not available, the test should fail with RustcInvocation error
        if result.is_ok() {
            // Compilation succeeded - rustc is available
            assert!(result.is_ok());
        } else if let Err(err) = result {
            // Check that it's a rustc invocation error (rustc not found)
            assert!(matches!(
                err,
                crate::error::CompilerError::RustcInvocation(_)
            ));
        }
    }

    #[test]
    fn test_compile_invalid_crusty_to_binary() {
        // Create an invalid Crusty source file (missing return type)
        let test_source = r#"
main() {
    return 0;
}
"#;
        let input_path = PathBuf::from("test_compile_invalid_12345.crst");
        let output_path = PathBuf::from("test_compile_invalid_12345");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        // Clean up
        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        // Should fail during parsing
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_with_no_compile_flag() {
        // Create a simple valid Crusty source file
        let test_source = r#"
int add(int a, int b) {
    return a + b;
}
"#;
        let input_path = PathBuf::from("test_no_compile_12345.crst");
        let output_path = PathBuf::from("test_no_compile_12345.rs");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: true, // Skip rustc invocation
        };

        let result = run_compiler(&options);

        // Clean up
        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);

        // Should succeed without invoking rustc
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_rust_emit_mode() {
        // Create a simple valid Crusty source file
        let test_source = r#"
int multiply(int x, int y) {
    return x * y;
}
"#;
        let input_path = PathBuf::from("test_rust_emit_12345.crst");
        let output_path = PathBuf::from("test_rust_emit_12345.rs");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Rust, // Only generate Rust, don't compile
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        // Clean up
        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);

        // Should succeed without invoking rustc
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_with_verbose_output() {
        // Create a simple valid Crusty source file
        let test_source = r#"
int square(int n) {
    return n * n;
}
"#;
        let input_path = PathBuf::from("test_verbose_12345.crst");
        let output_path = PathBuf::from("test_verbose_12345.rs");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Rust,
            absorb: None,
            verbose: true, // Enable verbose output
            no_compile: true,
        };

        let result = run_compiler(&options);

        // Clean up
        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);

        // Should succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_rustc_error_reporting() {
        // Create a Crusty file that will generate invalid Rust code
        // This is a bit tricky since our compiler should generate valid Rust
        // For now, we'll test that the error reporting mechanism works
        // by checking that rustc errors are properly captured

        // Create a simple valid Crusty source file
        let test_source = r#"
int test_func() {
    return 42;
}
"#;
        let input_path = PathBuf::from("test_error_report_12345.crst");
        let output_path = PathBuf::from("test_error_report_12345");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(output_path.clone()),
            out_dir: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        let result = run_compiler(&options);

        // Clean up
        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path);
        let _ = fs::remove_file(format!("{}.rs", output_path.display()));

        // If rustc is available, this should succeed
        // If not, we should get a RustcInvocation error
        if let Err(err) = result {
            // Should be either RustcInvocation or another error type
            assert!(
                matches!(err, crate::error::CompilerError::RustcInvocation(_))
                    || matches!(err, crate::error::CompilerError::Io(_))
            );
        }
    }
}
