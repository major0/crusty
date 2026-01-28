// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Command-line interface module for crustyc compiler.

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Crusty compiler - bidirectional transpiler between Crusty and Rust
#[derive(Parser, Debug)]
#[command(name = "crustyc")]
#[command(author, version, about, long_about = None)]
pub struct CompilerOptions {
    /// Input source file path
    pub input_file: PathBuf,

    /// Output file path (like rustc -o)
    #[arg(short = 'o', long = "out")]
    pub output_file: Option<PathBuf>,

    /// Output mode: what to emit (auto, rust, binary, ast)
    /// Auto mode detects from output file extension or defaults to binary
    #[arg(long = "emit", default_value = "auto")]
    pub emit: EmitMode,

    /// Absorb/parse source language (auto-detected from file extension if not specified)
    /// Use this to override auto-detection (e.g., --absorb=rust for .crst files)
    #[arg(long = "absorb")]
    pub absorb: Option<SourceLanguage>,

    /// Enable verbose output
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Skip rustc invocation (only generate code)
    #[arg(long = "no-compile")]
    pub no_compile: bool,
}

/// Output mode for the compiler
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum EmitMode {
    /// Auto-detect from output file extension or default to binary
    Auto,
    /// Generate Rust source code only
    Rust,
    /// Generate Rust source and compile to binary
    Binary,
    /// Output AST in human-readable format
    Ast,
}

/// Source language for parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SourceLanguage {
    /// Crusty source code
    Crusty,
    /// Rust source code
    Rust,
}

impl CompilerOptions {
    /// Parse command-line arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }

    /// Detect source language from input file extension
    /// Returns the detected language or the explicitly specified one via --absorb
    pub fn get_source_language(&self) -> SourceLanguage {
        // If explicitly specified via --absorb, use that
        if let Some(lang) = self.absorb {
            return lang;
        }

        // Auto-detect from file extension
        if let Some(ext) = self.input_file.extension().and_then(|e| e.to_str()) {
            match ext {
                "rs" => SourceLanguage::Rust,
                "crst" => SourceLanguage::Crusty,
                _ => SourceLanguage::Crusty, // Default to Crusty for unknown extensions
            }
        } else {
            SourceLanguage::Crusty // Default to Crusty if no extension
        }
    }

    /// Resolve the actual emit mode (convert Auto to concrete mode)
    pub fn get_emit_mode(&self) -> EmitMode {
        match self.emit {
            EmitMode::Auto => {
                // Auto-detect from output file extension if specified
                if let Some(ref output) = self.output_file {
                    if let Some(ext) = output.extension().and_then(|e| e.to_str()) {
                        match ext {
                            "rs" => EmitMode::Rust,
                            "ast" => EmitMode::Ast,
                            _ => EmitMode::Binary, // Default to binary for executables
                        }
                    } else {
                        EmitMode::Binary // No extension = binary
                    }
                } else {
                    EmitMode::Binary // No output specified = binary
                }
            }
            other => other, // Use explicitly specified mode
        }
    }

    /// Get the output file path, using a default if not specified
    pub fn get_output_path(&self) -> PathBuf {
        if let Some(ref path) = self.output_file {
            path.clone()
        } else {
            // Default output path based on input file and emit mode
            let input_stem = self
                .input_file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");

            match self.get_emit_mode() {
                EmitMode::Auto => PathBuf::from(input_stem), // Should not happen after get_emit_mode()
                EmitMode::Rust => PathBuf::from(format!("{}.rs", input_stem)),
                EmitMode::Binary => PathBuf::from(input_stem),
                EmitMode::Ast => PathBuf::from(format!("{}.ast", input_stem)),
            }
        }
    }
}

/// Read source file from disk
pub fn read_source_file(path: &PathBuf) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}

/// Write generated code to output file
pub fn write_output_file(path: &PathBuf, content: &str) -> Result<(), std::io::Error> {
    std::fs::write(path, content)
}

/// Run the compiler with the given options
pub fn run_compiler(options: &CompilerOptions) -> crate::error::Result<()> {
    use crate::ast::File;
    use crate::codegen::{CodeGenerator, TargetLanguage};
    use crate::error::CompilerError;
    use crate::parser::Parser;
    use crate::semantic::SemanticAnalyzer;

    let source_lang = options.get_source_language();
    let emit_mode = options.get_emit_mode();

    if options.verbose {
        println!("Compiling: {:?}", options.input_file);
        println!("Source language: {:?} ({})", 
            source_lang,
            if options.absorb.is_some() { "explicit" } else { "auto-detected" }
        );
        println!("Emit mode: {:?} ({})",
            emit_mode,
            if options.emit == EmitMode::Auto { "auto-detected" } else { "explicit" }
        );
    }

    // Step 1: Read source file
    let source = read_source_file(&options.input_file)?;

    if options.verbose {
        println!("Read {} bytes from source file", source.len());
    }

    // Step 2: Parse source based on detected/specified language
    let ast: File = match source_lang {
        SourceLanguage::Crusty => {
            if options.verbose {
                println!("Parsing Crusty source...");
            }
            let mut parser = Parser::new(&source)?;
            parser.parse_file()?
        }
        SourceLanguage::Rust => {
            // TODO: Implement Rust parsing with syn crate (task 20)
            return Err(CompilerError::CodeGen(crate::error::CodeGenError::new(
                "Rust source parsing not yet implemented",
            )));
        }
    };

    if options.verbose {
        println!("Parsed {} items", ast.items.len());
    }

    // Step 3: Handle AST emit mode
    if emit_mode == EmitMode::Ast {
        let ast_output = format!("{:#?}", ast);
        let output_path = options.get_output_path();
        write_output_file(&output_path, &ast_output)?;

        if options.verbose {
            println!("Wrote AST to: {:?}", output_path);
        }
        return Ok(());
    }

    // Step 4: Run semantic analysis
    if options.verbose {
        println!("Running semantic analysis...");
    }

    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&ast)?;

    if options.verbose {
        println!("Semantic analysis passed");
    }

    // Step 5: Generate target code (always Rust for now)
    if options.verbose {
        println!("Generating Rust code...");
    }

    let mut generator = CodeGenerator::new(TargetLanguage::Rust);
    let generated_code = generator.generate(&ast);

    if options.verbose {
        println!("Generated {} bytes of code", generated_code.len());
    }

    // Step 6: Write output file
    let output_path = options.get_output_path();
    let rust_output_path = if emit_mode == EmitMode::Binary {
        // For binary mode, write to a temporary .rs file
        PathBuf::from(format!(
            "{}.rs",
            output_path.file_stem().and_then(|s| s.to_str()).unwrap_or("output")
        ))
    } else {
        output_path.clone()
    };

    write_output_file(&rust_output_path, &generated_code)?;

    if options.verbose {
        println!("Wrote Rust code to: {:?}", rust_output_path);
    }

    // Step 7: Optionally invoke rustc
    if emit_mode == EmitMode::Binary && !options.no_compile {
        if options.verbose {
            println!("Invoking rustc...");
        }

        let rustc_result = invoke_rustc(&rust_output_path, &output_path, options.verbose);

        if let Err(e) = rustc_result {
            return Err(CompilerError::RustcInvocation(e));
        }

        if options.verbose {
            println!("Compilation successful: {:?}", output_path);
        }
    }

    Ok(())
}

/// Invoke rustc to compile generated Rust code
fn invoke_rustc(
    rust_file: &PathBuf,
    output_binary: &PathBuf,
    verbose: bool,
) -> Result<(), String> {
    use std::process::Command;

    let mut cmd = Command::new("rustc");
    cmd.arg(rust_file).arg("-o").arg(output_binary);

    if verbose {
        println!("Running: rustc {:?} -o {:?}", rust_file, output_binary);
    }

    let output = cmd.output().map_err(|e| format!("Failed to execute rustc: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("rustc compilation failed:\n{}", stderr));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_mode_values() {
        // Test that emit modes can be created
        let auto = EmitMode::Auto;
        let rust = EmitMode::Rust;
        let binary = EmitMode::Binary;
        let ast = EmitMode::Ast;

        assert_eq!(auto, EmitMode::Auto);
        assert_eq!(rust, EmitMode::Rust);
        assert_eq!(binary, EmitMode::Binary);
        assert_eq!(ast, EmitMode::Ast);
    }

    #[test]
    fn test_source_language_values() {
        let crusty = SourceLanguage::Crusty;
        let rust = SourceLanguage::Rust;

        assert_eq!(crusty, SourceLanguage::Crusty);
        assert_eq!(rust, SourceLanguage::Rust);
    }

    #[test]
    fn test_auto_detect_source_language_crusty() {
        let opts = CompilerOptions {
            input_file: PathBuf::from("test.crst"),
            output_file: None,
            emit: EmitMode::Auto,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        assert_eq!(opts.get_source_language(), SourceLanguage::Crusty);
    }

    #[test]
    fn test_auto_detect_source_language_rust() {
        let opts = CompilerOptions {
            input_file: PathBuf::from("test.rs"),
            output_file: None,
            emit: EmitMode::Auto,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        assert_eq!(opts.get_source_language(), SourceLanguage::Rust);
    }

    #[test]
    fn test_explicit_absorb_overrides_detection() {
        let opts = CompilerOptions {
            input_file: PathBuf::from("test.crst"),
            output_file: None,
            emit: EmitMode::Auto,
            absorb: Some(SourceLanguage::Rust),
            verbose: false,
            no_compile: false,
        };

        assert_eq!(opts.get_source_language(), SourceLanguage::Rust);
    }

    #[test]
    fn test_auto_detect_emit_mode_from_rs_extension() {
        let opts = CompilerOptions {
            input_file: PathBuf::from("test.crst"),
            output_file: Some(PathBuf::from("output.rs")),
            emit: EmitMode::Auto,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        assert_eq!(opts.get_emit_mode(), EmitMode::Rust);
    }

    #[test]
    fn test_auto_detect_emit_mode_from_ast_extension() {
        let opts = CompilerOptions {
            input_file: PathBuf::from("test.crst"),
            output_file: Some(PathBuf::from("output.ast")),
            emit: EmitMode::Auto,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        assert_eq!(opts.get_emit_mode(), EmitMode::Ast);
    }

    #[test]
    fn test_auto_detect_emit_mode_defaults_to_binary() {
        let opts = CompilerOptions {
            input_file: PathBuf::from("test.crst"),
            output_file: None,
            emit: EmitMode::Auto,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        assert_eq!(opts.get_emit_mode(), EmitMode::Binary);
    }

    #[test]
    fn test_explicit_emit_mode_overrides_auto() {
        let opts = CompilerOptions {
            input_file: PathBuf::from("test.crst"),
            output_file: Some(PathBuf::from("output.rs")),
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        assert_eq!(opts.get_emit_mode(), EmitMode::Binary);
    }

    #[test]
    fn test_get_output_path_with_explicit_output() {
        let opts = CompilerOptions {
            input_file: PathBuf::from("test.crst"),
            output_file: Some(PathBuf::from("custom_output.rs")),
            emit: EmitMode::Auto,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        assert_eq!(opts.get_output_path(), PathBuf::from("custom_output.rs"));
    }

    #[test]
    fn test_get_output_path_default_rust() {
        let opts = CompilerOptions {
            input_file: PathBuf::from("test.crst"),
            output_file: None,
            emit: EmitMode::Rust,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        assert_eq!(opts.get_output_path(), PathBuf::from("test.rs"));
    }

    #[test]
    fn test_get_output_path_default_binary() {
        let opts = CompilerOptions {
            input_file: PathBuf::from("test.crst"),
            output_file: None,
            emit: EmitMode::Binary,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        assert_eq!(opts.get_output_path(), PathBuf::from("test"));
    }

    #[test]
    fn test_get_output_path_default_ast() {
        let opts = CompilerOptions {
            input_file: PathBuf::from("test.crst"),
            output_file: None,
            emit: EmitMode::Ast,
            absorb: None,
            verbose: false,
            no_compile: false,
        };

        assert_eq!(opts.get_output_path(), PathBuf::from("test.ast"));
    }

    #[test]
    fn test_read_nonexistent_file() {
        let result = read_source_file(&PathBuf::from("nonexistent_file_12345.crst"));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    fn test_write_and_read_file() {
        use std::fs;
        let test_path = PathBuf::from("test_output_12345.tmp");
        let test_content = "test content for file I/O";

        // Write file
        let write_result = write_output_file(&test_path, test_content);
        assert!(write_result.is_ok());

        // Read file back
        let read_result = read_source_file(&test_path);
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), test_content);

        // Clean up
        let _ = fs::remove_file(&test_path);
    }

    #[test]
    fn test_run_compiler_with_valid_crusty_source() {
        use std::fs;

        // Create a simple test source file
        let test_source = r#"
int add(int a, int b) {
    return a + b;
}
"#;
        let input_path = PathBuf::from("test_add_12345.crst");
        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(PathBuf::from("test_add_12345.rs")),
            emit: EmitMode::Rust,
            absorb: None,
            verbose: false,
            no_compile: true,
        };

        let result = run_compiler(&options);

        // Clean up
        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file("test_add_12345.rs");

        assert!(result.is_ok());
    }

    #[test]
    fn test_run_compiler_with_nonexistent_file() {
        let options = CompilerOptions {
            input_file: PathBuf::from("nonexistent_file_99999.crst"),
            output_file: None,
            emit: EmitMode::Auto,
            absorb: None,
            verbose: false,
            no_compile: true,
        };

        let result = run_compiler(&options);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_compiler_ast_mode() {
        use std::fs;

        let test_source = r#"
int main() {
    return 0;
}
"#;
        let input_path = PathBuf::from("test_ast_12345.crst");
        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(PathBuf::from("test_ast_12345.ast")),
            emit: EmitMode::Ast,
            absorb: None,
            verbose: false,
            no_compile: true,
        };

        let result = run_compiler(&options);

        // Clean up
        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file("test_ast_12345.ast");

        assert!(result.is_ok());
    }

    #[test]
    fn test_run_compiler_rust_source_not_implemented() {
        use std::fs;

        let test_source = "fn main() {}";
        let input_path = PathBuf::from("test_rust_12345.rs");
        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: None,
            emit: EmitMode::Auto,
            absorb: None, // Will auto-detect as Rust from .rs extension
            verbose: false,
            no_compile: true,
        };

        let result = run_compiler(&options);

        // Clean up
        let _ = fs::remove_file(&input_path);

        // Should fail because Rust parsing is not yet implemented
        assert!(result.is_err());
    }

    #[test]
    fn test_auto_mode_with_crst_file() {
        use std::fs;

        let test_source = r#"
int square(int x) {
    return x * x;
}
"#;
        let input_path = PathBuf::from("test_auto_12345.crst");
        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: Some(PathBuf::from("test_auto_12345.rs")),
            emit: EmitMode::Auto, // Should auto-detect Rust from .rs output
            absorb: None,         // Should auto-detect Crusty from .crst input
            verbose: false,
            no_compile: true,
        };

        let result = run_compiler(&options);

        // Clean up
        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file("test_auto_12345.rs");

        assert!(result.is_ok());
    }
}
