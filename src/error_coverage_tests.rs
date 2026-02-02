// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Error module coverage tests

#[cfg(test)]
mod tests {
    use crate::error::*;
    use std::error::Error;

    #[test]
    fn test_position_creation() {
        let pos = Position::new(10, 20);
        assert_eq!(pos.line, 10);
        assert_eq!(pos.column, 20);
    }

    #[test]
    fn test_position_equality() {
        let pos1 = Position::new(5, 10);
        let pos2 = Position::new(5, 10);
        let pos3 = Position::new(5, 11);
        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);
    }

    #[test]
    fn test_span_creation() {
        let start = Position::new(1, 1);
        let end = Position::new(1, 10);
        let span = Span::new(start, end);
        assert_eq!(span.start, start);
        assert_eq!(span.end, end);
    }

    #[test]
    fn test_span_equality() {
        let span1 = Span::new(Position::new(1, 1), Position::new(1, 10));
        let span2 = Span::new(Position::new(1, 1), Position::new(1, 10));
        let span3 = Span::new(Position::new(1, 1), Position::new(1, 11));
        assert_eq!(span1, span2);
        assert_ne!(span1, span3);
    }

    #[test]
    fn test_lex_error_creation() {
        let span = Span::new(Position::new(1, 1), Position::new(1, 5));
        let error = LexError::new(span, "unexpected character");
        assert_eq!(error.span, span);
        assert_eq!(error.message, "unexpected character");
    }

    #[test]
    fn test_lex_error_display() {
        let span = Span::new(Position::new(2, 3), Position::new(2, 8));
        let error = LexError::new(span, "invalid token");
        let display = format!("{}", error);
        assert!(display.contains("Lexical error"));
        assert!(display.contains("2:3"));
        assert!(display.contains("invalid token"));
    }

    #[test]
    fn test_lex_error_as_std_error() {
        let span = Span::new(Position::new(1, 1), Position::new(1, 1));
        let error = LexError::new(span, "test");
        let _: &dyn std::error::Error = &error;
    }

    #[test]
    fn test_parse_error_creation() {
        let span = Span::new(Position::new(3, 5), Position::new(3, 10));
        let error = ParseError::new(
            span,
            "unexpected token",
            vec!["identifier".to_string()],
            "number",
        );
        assert_eq!(error.span, span);
        assert_eq!(error.message, "unexpected token");
        assert_eq!(error.expected, vec!["identifier".to_string()]);
        assert_eq!(error.found, "number");
    }

    #[test]
    fn test_parse_error_display_with_expected() {
        let span = Span::new(Position::new(1, 1), Position::new(1, 5));
        let error = ParseError::new(
            span,
            "syntax error",
            vec!["identifier".to_string(), "keyword".to_string()],
            "symbol",
        );
        let display = format!("{}", error);
        assert!(display.contains("Parse error"));
        assert!(display.contains("syntax error"));
        assert!(display.contains("identifier"));
        assert!(display.contains("keyword"));
        assert!(display.contains("symbol"));
    }

    #[test]
    fn test_parse_error_display_without_expected() {
        let span = Span::new(Position::new(1, 1), Position::new(1, 5));
        let error = ParseError::new(span, "syntax error", vec![], "symbol");
        let display = format!("{}", error);
        assert!(display.contains("Parse error"));
        assert!(display.contains("syntax error"));
        assert!(display.contains("symbol"));
    }

    #[test]
    fn test_parse_error_as_std_error() {
        let span = Span::new(Position::new(1, 1), Position::new(1, 1));
        let error = ParseError::new(span, "test", vec![], "test");
        let _: &dyn std::error::Error = &error;
    }

    #[test]
    fn test_semantic_error_kind_display() {
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
        assert_eq!(
            format!("{}", SemanticErrorKind::InvalidOperation),
            "invalid operation"
        );
        assert_eq!(
            format!("{}", SemanticErrorKind::UnsupportedFeature),
            "unsupported feature"
        );
    }

    #[test]
    fn test_semantic_error_kind_equality() {
        assert_eq!(
            SemanticErrorKind::UndefinedVariable,
            SemanticErrorKind::UndefinedVariable
        );
        assert_ne!(
            SemanticErrorKind::UndefinedVariable,
            SemanticErrorKind::TypeMismatch
        );
    }

    #[test]
    fn test_semantic_error_creation() {
        let span = Span::new(Position::new(5, 10), Position::new(5, 15));
        let error = SemanticError::new(
            span,
            SemanticErrorKind::UndefinedVariable,
            "variable 'x' not found",
        );
        assert_eq!(error.span, span);
        assert_eq!(error.kind, SemanticErrorKind::UndefinedVariable);
        assert_eq!(error.message, "variable 'x' not found");
    }

    #[test]
    fn test_semantic_error_display() {
        let span = Span::new(Position::new(10, 5), Position::new(10, 10));
        let error = SemanticError::new(
            span,
            SemanticErrorKind::TypeMismatch,
            "expected int, found string",
        );
        let display = format!("{}", error);
        assert!(display.contains("Semantic error"));
        assert!(display.contains("10:5"));
        assert!(display.contains("type mismatch"));
        assert!(display.contains("expected int, found string"));
    }

    #[test]
    fn test_semantic_error_as_std_error() {
        let span = Span::new(Position::new(1, 1), Position::new(1, 1));
        let error = SemanticError::new(span, SemanticErrorKind::TypeMismatch, "test");
        let _: &dyn std::error::Error = &error;
    }

    #[test]
    fn test_codegen_error_creation() {
        let error = CodeGenError::new("code generation failed");
        assert_eq!(error.message, "code generation failed");
    }

    #[test]
    fn test_codegen_error_display() {
        let error = CodeGenError::new("invalid syntax");
        let display = format!("{}", error);
        assert!(display.contains("Code generation error"));
        assert!(display.contains("invalid syntax"));
    }

    #[test]
    fn test_codegen_error_as_std_error() {
        let error = CodeGenError::new("test");
        let _: &dyn std::error::Error = &error;
    }

    #[test]
    fn test_compiler_error_lex_variant() {
        let lex_error = LexError::new(Span::new(Position::new(1, 1), Position::new(1, 1)), "test");
        let compiler_error = CompilerError::Lex(lex_error);
        let display = format!("{}", compiler_error);
        assert!(display.contains("Lexical error"));
    }

    #[test]
    fn test_compiler_error_parse_variant() {
        let parse_error = ParseError::new(
            Span::new(Position::new(1, 1), Position::new(1, 1)),
            "test",
            vec![],
            "test",
        );
        let compiler_error = CompilerError::Parse(parse_error);
        let display = format!("{}", compiler_error);
        assert!(display.contains("Parse error"));
    }

    #[test]
    fn test_compiler_error_semantic_variant() {
        let semantic_error = SemanticError::new(
            Span::new(Position::new(1, 1), Position::new(1, 1)),
            SemanticErrorKind::TypeMismatch,
            "test",
        );
        let compiler_error = CompilerError::Semantic(vec![semantic_error]);
        let display = format!("{}", compiler_error);
        assert!(display.contains("Semantic errors"));
    }

    #[test]
    fn test_compiler_error_semantic_multiple() {
        let error1 = SemanticError::new(
            Span::new(Position::new(1, 1), Position::new(1, 1)),
            SemanticErrorKind::TypeMismatch,
            "error 1",
        );
        let error2 = SemanticError::new(
            Span::new(Position::new(2, 2), Position::new(2, 2)),
            SemanticErrorKind::UndefinedVariable,
            "error 2",
        );
        let compiler_error = CompilerError::Semantic(vec![error1, error2]);
        let display = format!("{}", compiler_error);
        assert!(display.contains("Semantic errors"));
        assert!(display.contains("error 1"));
        assert!(display.contains("error 2"));
    }

    #[test]
    fn test_compiler_error_codegen_variant() {
        let codegen_error = CodeGenError::new("test");
        let compiler_error = CompilerError::CodeGen(codegen_error);
        let display = format!("{}", compiler_error);
        assert!(display.contains("Code generation error"));
    }

    #[test]
    fn test_compiler_error_io_variant() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let compiler_error = CompilerError::Io(io_error);
        let display = format!("{}", compiler_error);
        assert!(display.contains("I/O error"));
        assert!(display.contains("file not found"));
    }

    #[test]
    fn test_compiler_error_rustc_invocation_variant() {
        let compiler_error = CompilerError::RustcInvocation("rustc failed".to_string());
        let display = format!("{}", compiler_error);
        assert!(display.contains("rustc invocation error"));
        assert!(display.contains("rustc failed"));
    }

    #[test]
    fn test_compiler_error_source_lex() {
        let lex_error = LexError::new(Span::new(Position::new(1, 1), Position::new(1, 1)), "test");
        let compiler_error = CompilerError::Lex(lex_error);
        assert!(compiler_error.source().is_some());
    }

    #[test]
    fn test_compiler_error_source_parse() {
        let parse_error = ParseError::new(
            Span::new(Position::new(1, 1), Position::new(1, 1)),
            "test",
            vec![],
            "test",
        );
        let compiler_error = CompilerError::Parse(parse_error);
        assert!(compiler_error.source().is_some());
    }

    #[test]
    fn test_compiler_error_source_semantic() {
        let semantic_error = SemanticError::new(
            Span::new(Position::new(1, 1), Position::new(1, 1)),
            SemanticErrorKind::TypeMismatch,
            "test",
        );
        let compiler_error = CompilerError::Semantic(vec![semantic_error]);
        assert!(compiler_error.source().is_some());
    }

    #[test]
    fn test_compiler_error_source_semantic_empty() {
        let compiler_error = CompilerError::Semantic(vec![]);
        assert!(compiler_error.source().is_none());
    }

    #[test]
    fn test_compiler_error_source_codegen() {
        let codegen_error = CodeGenError::new("test");
        let compiler_error = CompilerError::CodeGen(codegen_error);
        assert!(compiler_error.source().is_some());
    }

    #[test]
    fn test_compiler_error_source_io() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let compiler_error = CompilerError::Io(io_error);
        assert!(compiler_error.source().is_some());
    }

    #[test]
    fn test_compiler_error_source_rustc() {
        let compiler_error = CompilerError::RustcInvocation("test".to_string());
        assert!(compiler_error.source().is_none());
    }

    #[test]
    fn test_compiler_error_from_lex_error() {
        let lex_error = LexError::new(Span::new(Position::new(1, 1), Position::new(1, 1)), "test");
        let compiler_error: CompilerError = lex_error.into();
        assert!(matches!(compiler_error, CompilerError::Lex(_)));
    }

    #[test]
    fn test_compiler_error_from_parse_error() {
        let parse_error = ParseError::new(
            Span::new(Position::new(1, 1), Position::new(1, 1)),
            "test",
            vec![],
            "test",
        );
        let compiler_error: CompilerError = parse_error.into();
        assert!(matches!(compiler_error, CompilerError::Parse(_)));
    }

    #[test]
    fn test_compiler_error_from_semantic_errors() {
        let semantic_error = SemanticError::new(
            Span::new(Position::new(1, 1), Position::new(1, 1)),
            SemanticErrorKind::TypeMismatch,
            "test",
        );
        let compiler_error: CompilerError = vec![semantic_error].into();
        assert!(matches!(compiler_error, CompilerError::Semantic(_)));
    }

    #[test]
    fn test_compiler_error_from_codegen_error() {
        let codegen_error = CodeGenError::new("test");
        let compiler_error: CompilerError = codegen_error.into();
        assert!(matches!(compiler_error, CompilerError::CodeGen(_)));
    }

    #[test]
    fn test_compiler_error_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let compiler_error: CompilerError = io_error.into();
        assert!(matches!(compiler_error, CompilerError::Io(_)));
    }
}
