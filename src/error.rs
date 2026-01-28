// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Error handling types and utilities.

use std::fmt;

/// Source code position for error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Source code span for error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

/// Lexical analysis error
#[derive(Debug, Clone)]
pub struct LexError {
    pub span: Span,
    pub message: String,
}

impl LexError {
    pub fn new(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
        }
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Lexical error at {}: {}", self.span, self.message)
    }
}

impl std::error::Error for LexError {}

/// Parse error with expected tokens
#[derive(Debug, Clone)]
pub struct ParseError {
    pub span: Span,
    pub message: String,
    pub expected: Vec<String>,
    pub found: String,
}

impl ParseError {
    pub fn new(
        span: Span,
        message: impl Into<String>,
        expected: Vec<String>,
        found: impl Into<String>,
    ) -> Self {
        Self {
            span,
            message: message.into(),
            expected,
            found: found.into(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error at {}: {}", self.span, self.message)?;
        if !self.expected.is_empty() {
            write!(f, " (expected: {})", self.expected.join(", "))?;
        }
        write!(f, " (found: {})", self.found)
    }
}

impl std::error::Error for ParseError {}

/// Semantic analysis error kinds
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticErrorKind {
    UndefinedVariable,
    TypeMismatch,
    DuplicateDefinition,
    InvalidOperation,
    UnsupportedFeature,
}

impl fmt::Display for SemanticErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SemanticErrorKind::UndefinedVariable => write!(f, "undefined variable"),
            SemanticErrorKind::TypeMismatch => write!(f, "type mismatch"),
            SemanticErrorKind::DuplicateDefinition => write!(f, "duplicate definition"),
            SemanticErrorKind::InvalidOperation => write!(f, "invalid operation"),
            SemanticErrorKind::UnsupportedFeature => write!(f, "unsupported feature"),
        }
    }
}

/// Semantic analysis error
#[derive(Debug, Clone)]
pub struct SemanticError {
    pub span: Span,
    pub kind: SemanticErrorKind,
    pub message: String,
}

impl SemanticError {
    pub fn new(span: Span, kind: SemanticErrorKind, message: impl Into<String>) -> Self {
        Self {
            span,
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Semantic error at {} ({}): {}",
            self.span, self.kind, self.message
        )
    }
}

impl std::error::Error for SemanticError {}

/// Code generation error
#[derive(Debug, Clone)]
pub struct CodeGenError {
    pub message: String,
}

impl CodeGenError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for CodeGenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Code generation error: {}", self.message)
    }
}

impl std::error::Error for CodeGenError {}

/// Top-level compiler error
#[derive(Debug)]
pub enum CompilerError {
    Lex(LexError),
    Parse(ParseError),
    Semantic(Vec<SemanticError>),
    CodeGen(CodeGenError),
    Io(std::io::Error),
    RustcInvocation(String),
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompilerError::Lex(e) => write!(f, "{}", e),
            CompilerError::Parse(e) => write!(f, "{}", e),
            CompilerError::Semantic(errors) => {
                writeln!(f, "Semantic errors:")?;
                for error in errors {
                    writeln!(f, "  {}", error)?;
                }
                Ok(())
            }
            CompilerError::CodeGen(e) => write!(f, "{}", e),
            CompilerError::Io(e) => write!(f, "I/O error: {}", e),
            CompilerError::RustcInvocation(msg) => write!(f, "rustc invocation error: {}", msg),
        }
    }
}

impl std::error::Error for CompilerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CompilerError::Lex(e) => Some(e),
            CompilerError::Parse(e) => Some(e),
            CompilerError::Semantic(errors) => errors.first().map(|e| e as &dyn std::error::Error),
            CompilerError::CodeGen(e) => Some(e),
            CompilerError::Io(e) => Some(e),
            CompilerError::RustcInvocation(_) => None,
        }
    }
}

impl From<LexError> for CompilerError {
    fn from(e: LexError) -> Self {
        CompilerError::Lex(e)
    }
}

impl From<ParseError> for CompilerError {
    fn from(e: ParseError) -> Self {
        CompilerError::Parse(e)
    }
}

impl From<Vec<SemanticError>> for CompilerError {
    fn from(e: Vec<SemanticError>) -> Self {
        CompilerError::Semantic(e)
    }
}

impl From<CodeGenError> for CompilerError {
    fn from(e: CodeGenError) -> Self {
        CompilerError::CodeGen(e)
    }
}

impl From<std::io::Error> for CompilerError {
    fn from(e: std::io::Error) -> Self {
        CompilerError::Io(e)
    }
}

/// Result type for compiler operations
pub type Result<T> = std::result::Result<T, CompilerError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_display() {
        let pos = Position::new(10, 5);
        assert_eq!(format!("{}", pos), "10:5");
    }

    #[test]
    fn test_span_display() {
        let span = Span::new(Position::new(1, 1), Position::new(1, 10));
        assert_eq!(format!("{}", span), "1:1-1:10");
    }

    #[test]
    fn test_lex_error() {
        let span = Span::new(Position::new(1, 1), Position::new(1, 5));
        let error = LexError::new(span, "unexpected character");
        assert!(format!("{}", error).contains("Lexical error"));
        assert!(format!("{}", error).contains("unexpected character"));
    }

    #[test]
    fn test_parse_error() {
        let span = Span::new(Position::new(2, 3), Position::new(2, 8));
        let error = ParseError::new(
            span,
            "unexpected token",
            vec!["identifier".to_string(), "keyword".to_string()],
            "number",
        );
        let display = format!("{}", error);
        assert!(display.contains("Parse error"));
        assert!(display.contains("identifier"));
        assert!(display.contains("number"));
    }

    #[test]
    fn test_semantic_error() {
        let span = Span::new(Position::new(5, 10), Position::new(5, 15));
        let error = SemanticError::new(
            span,
            SemanticErrorKind::UndefinedVariable,
            "variable 'x' not found",
        );
        let display = format!("{}", error);
        assert!(display.contains("Semantic error"));
        assert!(display.contains("undefined variable"));
        assert!(display.contains("variable 'x' not found"));
    }

    #[test]
    fn test_compiler_error_conversion() {
        let lex_error = LexError::new(
            Span::new(Position::new(1, 1), Position::new(1, 1)),
            "test",
        );
        let compiler_error: CompilerError = lex_error.into();
        assert!(matches!(compiler_error, CompilerError::Lex(_)));
    }

    #[test]
    fn test_semantic_error_kinds() {
        assert_eq!(
            format!("{}", SemanticErrorKind::UndefinedVariable),
            "undefined variable"
        );
        assert_eq!(
            format!("{}", SemanticErrorKind::TypeMismatch),
            "type mismatch"
        );
        assert_eq!(
            format!("{}", SemanticErrorKind::DuplicateDefinition),
            "duplicate definition"
        );
    }
}
