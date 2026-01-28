// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Command-line interface module for crustyc compiler.

use clap::{Parser, ValueEnum};
use std::path::{Path, PathBuf};

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

    /// Output directory for generated files (for multi-file compilation)
    #[arg(long = "out-dir")]
    pub out_dir: Option<PathBuf>,

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
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)
}

/// Create output directory if it doesn't exist
pub fn ensure_output_dir(dir: &PathBuf) -> Result<(), std::io::Error> {
    if !dir.exists() {
        std::fs::create_dir_all(dir)?;
    }
    Ok(())
}

/// Compute output path for a source file when using --out-dir
/// Preserves the directory structure relative to the base directory
pub fn compute_output_path(
    source_file: &Path,
    base_dir: &Path,
    out_dir: &Path,
    extension: &str,
) -> Result<PathBuf, std::io::Error> {
    // Get the relative path from base_dir to source_file
    let relative_path = if source_file.starts_with(base_dir) {
        source_file.strip_prefix(base_dir).unwrap()
    } else {
        // If source is not under base_dir, just use the file name
        source_file.file_name().map(std::path::Path::new).unwrap()
    };

    // Change the extension
    let output_file = relative_path.with_extension(extension);

    // Combine with out_dir
    Ok(out_dir.join(output_file))
}

/// Run the compiler with the given options
pub fn run_compiler(options: &CompilerOptions) -> crate::error::Result<()> {
    let source_lang = options.get_source_language();
    let emit_mode = options.get_emit_mode();

    if options.verbose {
        println!("Compiling: {:?}", options.input_file);
        println!(
            "Source language: {:?} ({})",
            source_lang,
            if options.absorb.is_some() {
                "explicit"
            } else {
                "auto-detected"
            }
        );
        println!(
            "Emit mode: {:?} ({})",
            emit_mode,
            if options.emit == EmitMode::Auto {
                "auto-detected"
            } else {
                "explicit"
            }
        );
        if let Some(ref out_dir) = options.out_dir {
            println!("Output directory: {:?}", out_dir);
        }
    }

    // Check if input is a directory (batch mode) or a single file
    if options.input_file.is_dir() {
        // Batch transpilation mode
        return run_batch_compilation(options);
    }

    // Single file compilation
    run_single_file_compilation(options)
}

/// Run compilation for a single source file
fn run_single_file_compilation(options: &CompilerOptions) -> crate::error::Result<()> {
    // For single file mode, use the file's parent directory as base
    let base_dir = options
        .input_file
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();
    run_single_file_compilation_with_base(options, &base_dir)
}

/// Run compilation for a single source file with a specified base directory
/// The base_dir is used to preserve directory structure when using --out-dir
fn run_single_file_compilation_with_base(
    options: &CompilerOptions,
    base_dir: &Path,
) -> crate::error::Result<()> {
    use crate::ast::File;
    use crate::codegen::{CodeGenerator, TargetLanguage};
    use crate::error::CompilerError;
    use crate::parser::Parser;
    use crate::semantic::SemanticAnalyzer;

    let source_lang = options.get_source_language();
    let emit_mode = options.get_emit_mode();

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
    let output_path = if let Some(ref out_dir) = options.out_dir {
        // Using --out-dir: compute output path preserving directory structure
        ensure_output_dir(out_dir)?;
        let extension = "rs"; // Always emit Rust for now
        compute_output_path(&options.input_file, base_dir, out_dir, extension)?
    } else {
        options.get_output_path()
    };

    let rust_output_path = if emit_mode == EmitMode::Binary {
        // For binary mode, write to a temporary .rs file
        PathBuf::from(format!(
            "{}.rs",
            output_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output")
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

        use crate::rustc;
        let rustc_result = rustc::invoke_rustc(&rust_output_path, &output_path, options.verbose)
            .map_err(CompilerError::RustcInvocation)?;

        if !rustc_result.is_success() {
            return Err(CompilerError::RustcInvocation(
                rustc_result
                    .error_message()
                    .unwrap_or_else(|| "Unknown rustc error".to_string()),
            ));
        }

        if options.verbose {
            println!("Compilation successful: {:?}", output_path);
        }
    }

    Ok(())
}

/// Run batch compilation for multiple source files in a directory
fn run_batch_compilation(options: &CompilerOptions) -> crate::error::Result<()> {
    use crate::error::CompilerError;

    if options.verbose {
        println!("Batch compilation mode: discovering source files...");
    }

    // Determine the file extension to look for based on source language
    let source_lang = options.get_source_language();
    let extension = match source_lang {
        SourceLanguage::Crusty => "crst",
        SourceLanguage::Rust => "rs",
    };

    // Discover all source files recursively
    let source_files = discover_source_files(&options.input_file, extension)?;

    if options.verbose {
        println!("Found {} source files", source_files.len());
    }

    if source_files.is_empty() {
        return Err(CompilerError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "No .{} files found in directory: {:?}",
                extension, options.input_file
            ),
        )));
    }

    // Ensure output directory exists
    let out_dir = options.out_dir.as_ref().ok_or_else(|| {
        CompilerError::CodeGen(crate::error::CodeGenError::new(
            "--out-dir is required for batch compilation",
        ))
    })?;
    ensure_output_dir(out_dir)?;

    // Compile each file
    let mut errors = Vec::new();
    let mut success_count = 0;

    // Store the base directory for preserving structure
    let base_dir = options.input_file.clone();

    for source_file in &source_files {
        if options.verbose {
            println!("Compiling: {:?}", source_file);
        }

        // Create a modified options struct for this file
        let file_options = CompilerOptions {
            input_file: source_file.clone(),
            output_file: None,
            out_dir: options.out_dir.clone(),
            emit: options.emit,
            absorb: options.absorb,
            verbose: false, // Suppress per-file verbose output
            no_compile: options.no_compile,
        };

        match run_single_file_compilation_with_base(&file_options, &base_dir) {
            Ok(()) => {
                success_count += 1;
                if options.verbose {
                    println!("  ✓ Success");
                }
            }
            Err(e) => {
                errors.push((source_file.clone(), e));
                if options.verbose {
                    println!("  ✗ Error: {}", errors.last().unwrap().1);
                }
            }
        }
    }

    // Report results
    if options.verbose {
        println!("\nBatch compilation complete:");
        println!("  Success: {}/{}", success_count, source_files.len());
        println!("  Errors: {}", errors.len());
    }

    if !errors.is_empty() {
        // Report all errors
        eprintln!("\nErrors encountered during batch compilation:");
        for (file, error) in &errors {
            eprintln!("  {:?}: {}", file, error);
        }
        return Err(CompilerError::CodeGen(crate::error::CodeGenError::new(
            format!("Batch compilation failed with {} errors", errors.len()),
        )));
    }

    Ok(())
}

/// Discover all source files with the given extension in a directory (recursively)
fn discover_source_files(dir: &PathBuf, extension: &str) -> Result<Vec<PathBuf>, std::io::Error> {
    use std::fs;

    let mut files = Vec::new();

    fn visit_dir(
        dir: &PathBuf,
        extension: &str,
        files: &mut Vec<PathBuf>,
    ) -> Result<(), std::io::Error> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dir(&path, extension, files)?;
                } else if let Some(ext) = path.extension() {
                    if ext == extension {
                        files.push(path);
                    }
                }
            }
        }
        Ok(())
    }

    visit_dir(dir, extension, &mut files)?;
    files.sort(); // Sort for deterministic order
    Ok(files)
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
            out_dir: None,
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
            out_dir: None,
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
            out_dir: None,
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
            out_dir: None,
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
            out_dir: None,
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
            out_dir: None,
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
            out_dir: None,
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
            out_dir: None,
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
            out_dir: None,
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
            out_dir: None,
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
            out_dir: None,
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
            out_dir: None,
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
            out_dir: None,
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
            out_dir: None,
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
            out_dir: None,
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
            out_dir: None,
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

    #[test]
    fn test_out_dir_option() {
        use std::fs;

        let test_source = r#"
int add(int a, int b) {
    return a + b;
}
"#;
        let input_path = PathBuf::from("test_outdir_12345.crst");
        let out_dir = PathBuf::from("test_output_dir_12345");

        fs::write(&input_path, test_source).unwrap();

        let options = CompilerOptions {
            input_file: input_path.clone(),
            output_file: None,
            out_dir: Some(out_dir.clone()),
            emit: EmitMode::Rust,
            absorb: None,
            verbose: false,
            no_compile: true,
        };

        let result = run_compiler(&options);

        // Check that output directory was created
        assert!(out_dir.exists());

        // Check that output file exists in the output directory
        let expected_output = out_dir.join("test_outdir_12345.rs");
        assert!(expected_output.exists());

        // Clean up
        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_dir_all(&out_dir);

        assert!(result.is_ok());
    }

    #[test]
    fn test_compute_output_path() {
        let source = PathBuf::from("src/module/file.crst");
        let base = PathBuf::from("src");
        let out_dir = PathBuf::from("target/generated");

        let result = compute_output_path(&source, &base, &out_dir, "rs").unwrap();
        assert_eq!(result, PathBuf::from("target/generated/module/file.rs"));
    }

    #[test]
    fn test_compute_output_path_no_subdirs() {
        let source = PathBuf::from("file.crst");
        let base = PathBuf::from(".");
        let out_dir = PathBuf::from("output");

        let result = compute_output_path(&source, &base, &out_dir, "rs").unwrap();
        assert_eq!(result, PathBuf::from("output/file.rs"));
    }

    #[test]
    fn test_discover_source_files() {
        use std::fs;

        // Create a temporary directory structure
        let test_dir = PathBuf::from("test_discover_12345");
        fs::create_dir_all(test_dir.join("subdir")).unwrap();

        // Create some test files
        fs::write(test_dir.join("file1.crst"), "").unwrap();
        fs::write(test_dir.join("file2.crst"), "").unwrap();
        fs::write(test_dir.join("subdir/file3.crst"), "").unwrap();
        fs::write(test_dir.join("other.txt"), "").unwrap();

        let files = discover_source_files(&test_dir, "crst").unwrap();

        // Should find 3 .crst files
        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|f| f.ends_with("file1.crst")));
        assert!(files.iter().any(|f| f.ends_with("file2.crst")));
        assert!(files.iter().any(|f| f.ends_with("file3.crst")));
        assert!(!files.iter().any(|f| f.ends_with("other.txt")));

        // Clean up
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_batch_compilation() {
        use std::fs;

        // Create a temporary directory with multiple source files
        let test_dir = PathBuf::from("test_batch_12345");
        let out_dir = PathBuf::from("test_batch_output_12345");

        fs::create_dir_all(&test_dir).unwrap();

        let source1 = r#"
int add(int a, int b) {
    return a + b;
}
"#;
        let source2 = r#"
int multiply(int a, int b) {
    return a * b;
}
"#;

        fs::write(test_dir.join("file1.crst"), source1).unwrap();
        fs::write(test_dir.join("file2.crst"), source2).unwrap();

        let options = CompilerOptions {
            input_file: test_dir.clone(),
            output_file: None,
            out_dir: Some(out_dir.clone()),
            emit: EmitMode::Rust,
            absorb: None,
            verbose: false,
            no_compile: true,
        };

        let result = run_compiler(&options);

        // Check that output files were created
        assert!(out_dir.join("file1.rs").exists());
        assert!(out_dir.join("file2.rs").exists());

        // Clean up
        let _ = fs::remove_dir_all(&test_dir);
        let _ = fs::remove_dir_all(&out_dir);

        assert!(result.is_ok());
    }

    #[test]
    fn test_batch_compilation_requires_out_dir() {
        use std::fs;

        let test_dir = PathBuf::from("test_batch_nodir_12345");
        fs::create_dir_all(&test_dir).unwrap();
        fs::write(test_dir.join("file.crst"), "int main() { return 0; }").unwrap();

        let options = CompilerOptions {
            input_file: test_dir.clone(),
            output_file: None,
            out_dir: None, // Missing --out-dir
            emit: EmitMode::Rust,
            absorb: None,
            verbose: false,
            no_compile: true,
        };

        let result = run_compiler(&options);

        // Clean up
        let _ = fs::remove_dir_all(&test_dir);

        // Should fail because --out-dir is required for batch mode
        assert!(result.is_err());
    }
}
