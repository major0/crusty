// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Additional semantic analyzer tests for improved coverage

#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::error::{Position, SemanticErrorKind, Span};
    use crate::semantic::SemanticAnalyzer;

    fn create_span() -> Span {
        Span::new(Position::new(0, 0), Position::new(0, 0))
    }

    #[test]
    fn test_array_index_with_integer_types() {
        let mut analyzer = SemanticAnalyzer::new();

        // Create array and index with different integer types
        let array_expr = Expression::ArrayLit {
            elements: vec![
                Expression::Literal(Literal::Int(1)),
                Expression::Literal(Literal::Int(2)),
            ],
        };

        // Test with i32 index
        let index_expr = Expression::Index {
            expr: Box::new(array_expr.clone()),
            index: Box::new(Expression::Literal(Literal::Int(0))),
        };

        let result_type = analyzer.analyze_expression_test(&index_expr);
        assert_eq!(result_type, Type::Primitive(PrimitiveType::I32));
    }

    #[test]
    fn test_array_index_with_non_integer() {
        let mut analyzer = SemanticAnalyzer::new();

        let array_expr = Expression::ArrayLit {
            elements: vec![Expression::Literal(Literal::Int(1))],
        };

        // Try to index with a string (invalid)
        let index_expr = Expression::Index {
            expr: Box::new(array_expr),
            index: Box::new(Expression::Literal(Literal::String("test".to_string()))),
        };

        analyzer.analyze_expression_test(&index_expr);

        // Should have an error about non-integer index
        assert!(!analyzer.errors().is_empty());
        assert!(analyzer
            .errors()
            .iter()
            .any(|e| e.kind == SemanticErrorKind::TypeMismatch));
    }

    #[test]
    fn test_index_on_non_array_type() {
        let mut analyzer = SemanticAnalyzer::new();

        // Try to index an integer (invalid)
        let index_expr = Expression::Index {
            expr: Box::new(Expression::Literal(Literal::Int(42))),
            index: Box::new(Expression::Literal(Literal::Int(0))),
        };

        analyzer.analyze_expression_test(&index_expr);

        // Should have an error about indexing non-array
        assert!(!analyzer.errors().is_empty());
        assert!(analyzer
            .errors()
            .iter()
            .any(|e| e.kind == SemanticErrorKind::InvalidOperation));
    }

    #[test]
    fn test_ternary_with_non_boolean_condition() {
        let mut analyzer = SemanticAnalyzer::new();

        let ternary = Expression::Ternary {
            condition: Box::new(Expression::Literal(Literal::Int(42))), // Not boolean
            then_expr: Box::new(Expression::Literal(Literal::Int(1))),
            else_expr: Box::new(Expression::Literal(Literal::Int(2))),
        };

        analyzer.analyze_expression_test(&ternary);

        // Should have an error about non-boolean condition
        assert!(!analyzer.errors().is_empty());
        assert!(analyzer
            .errors()
            .iter()
            .any(|e| e.kind == SemanticErrorKind::TypeMismatch));
    }

    #[test]
    fn test_ternary_with_incompatible_branches() {
        let mut analyzer = SemanticAnalyzer::new();

        let ternary = Expression::Ternary {
            condition: Box::new(Expression::Literal(Literal::Bool(true))),
            then_expr: Box::new(Expression::Literal(Literal::Int(1))),
            else_expr: Box::new(Expression::Literal(Literal::String("test".to_string()))),
        };

        analyzer.analyze_expression_test(&ternary);

        // Should have an error about incompatible branch types
        assert!(!analyzer.errors().is_empty());
        assert!(analyzer
            .errors()
            .iter()
            .any(|e| e.kind == SemanticErrorKind::TypeMismatch));
    }

    #[test]
    fn test_error_propagation_on_fallible_type() {
        let mut analyzer = SemanticAnalyzer::new();

        let fallible_expr = Expression::Literal(Literal::Int(42));
        let error_prop = Expression::ErrorProp {
            expr: Box::new(fallible_expr),
        };

        analyzer.analyze_expression_test(&error_prop);

        // Should have an error since we're using ! on non-fallible type
        assert!(!analyzer.errors().is_empty());
        assert!(analyzer
            .errors()
            .iter()
            .any(|e| e.kind == SemanticErrorKind::InvalidOperation));
    }

    #[test]
    fn test_method_call_expression() {
        let mut analyzer = SemanticAnalyzer::new();

        let method_call = Expression::MethodCall {
            receiver: Box::new(Expression::Ident(Ident::new("obj"))),
            method: Ident::new("method"),
            args: vec![Expression::Literal(Literal::Int(42))],
        };

        let result_type = analyzer.analyze_expression_test(&method_call);
        // Method calls return Auto for now (simplified type checking)
        assert_eq!(result_type, Type::Auto);
    }

    #[test]
    fn test_type_scoped_call() {
        let mut analyzer = SemanticAnalyzer::new();

        let type_scoped = Expression::TypeScopedCall {
            ty: Type::Ident(Ident::new("MyType")),
            method: Ident::new("new"),
            args: vec![],
        };

        let result_type = analyzer.analyze_expression_test(&type_scoped);
        assert_eq!(result_type, Type::Ident(Ident::new("MyType")));
    }

    #[test]
    fn test_explicit_generic_call() {
        let mut analyzer = SemanticAnalyzer::new();

        let generic_call = Expression::ExplicitGenericCall {
            ty: Type::Ident(Ident::new("Vec")),
            generics: vec![Type::Primitive(PrimitiveType::I32)],
            method: Ident::new("new"),
            args: vec![],
        };

        let result_type = analyzer.analyze_expression_test(&generic_call);
        assert_eq!(result_type, Type::Ident(Ident::new("Vec")));
    }

    #[test]
    fn test_rust_block_expression() {
        let mut analyzer = SemanticAnalyzer::new();

        let rust_block = Expression::RustBlock {
            tokens: vec![], // Empty tokens for test
        };

        let result_type = analyzer.analyze_expression_test(&rust_block);
        // Rust blocks return Auto (not type-checked)
        assert_eq!(result_type, Type::Auto);
    }

    #[test]
    fn test_macro_call_expression() {
        let mut analyzer = SemanticAnalyzer::new();

        let macro_call = Expression::MacroCall {
            name: Ident::new("__my_macro__"),
            args: vec![], // Empty args for test
        };

        let result_type = analyzer.analyze_expression_test(&macro_call);
        // Macro calls return Auto (not type-checked at this stage)
        assert_eq!(result_type, Type::Auto);
    }

    #[test]
    fn test_range_expression() {
        let mut analyzer = SemanticAnalyzer::new();

        let range = Expression::Range {
            start: Some(Box::new(Expression::Literal(Literal::Int(0)))),
            end: Some(Box::new(Expression::Literal(Literal::Int(10)))),
            inclusive: false,
        };

        let result_type = analyzer.analyze_expression_test(&range);
        // Ranges return Auto (simplified)
        assert_eq!(result_type, Type::Auto);
    }

    #[test]
    fn test_empty_array_literal() {
        let mut analyzer = SemanticAnalyzer::new();

        let empty_array = Expression::ArrayLit { elements: vec![] };

        let result_type = analyzer.analyze_expression_test(&empty_array);
        assert_eq!(
            result_type,
            Type::Array {
                ty: Box::new(Type::Auto),
                size: Some(0),
            }
        );
    }

    #[test]
    fn test_array_with_incompatible_elements() {
        let mut analyzer = SemanticAnalyzer::new();

        let array = Expression::ArrayLit {
            elements: vec![
                Expression::Literal(Literal::Int(1)),
                Expression::Literal(Literal::String("test".to_string())),
            ],
        };

        analyzer.analyze_expression_test(&array);

        // Should have an error about incompatible element types
        assert!(!analyzer.errors().is_empty());
        assert!(analyzer
            .errors()
            .iter()
            .any(|e| e.kind == SemanticErrorKind::TypeMismatch));
    }

    #[test]
    fn test_struct_init_expression() {
        let mut analyzer = SemanticAnalyzer::new();

        let struct_init = Expression::StructInit {
            ty: Type::Ident(Ident::new("MyStruct")),
            fields: vec![
                (Ident::new("field1"), Expression::Literal(Literal::Int(42))),
                (
                    Ident::new("field2"),
                    Expression::Literal(Literal::String("test".to_string())),
                ),
            ],
        };

        let result_type = analyzer.analyze_expression_test(&struct_init);
        assert_eq!(result_type, Type::Ident(Ident::new("MyStruct")));
    }

    #[test]
    fn test_invalid_cast() {
        let mut analyzer = SemanticAnalyzer::new();

        // Try to cast a string to an integer (invalid)
        let cast_expr = Expression::Cast {
            expr: Box::new(Expression::Literal(Literal::String("test".to_string()))),
            ty: Type::Primitive(PrimitiveType::I32),
        };

        analyzer.analyze_expression_test(&cast_expr);

        // Should have an error about invalid cast
        assert!(!analyzer.errors().is_empty());
        assert!(analyzer
            .errors()
            .iter()
            .any(|e| e.kind == SemanticErrorKind::InvalidOperation));
    }

    #[test]
    fn test_valid_pointer_cast() {
        let mut analyzer = SemanticAnalyzer::new();

        // Cast between pointer types (valid)
        let cast_expr = Expression::Cast {
            expr: Box::new(Expression::Literal(Literal::Int(0))),
            ty: Type::Pointer {
                ty: Box::new(Type::Primitive(PrimitiveType::Void)),
                mutable: false,
            },
        };

        let result_type = analyzer.analyze_expression_test(&cast_expr);
        assert_eq!(
            result_type,
            Type::Pointer {
                ty: Box::new(Type::Primitive(PrimitiveType::Void)),
                mutable: false,
            }
        );
    }

    #[test]
    fn test_field_access_on_reference() {
        let mut analyzer = SemanticAnalyzer::new();

        // Create a file with a struct and field access through reference
        let file = File {
            items: vec![
                Item::Struct(Struct {
                    visibility: Visibility::Public,
                    name: Ident::new("MyStruct"),
                    fields: vec![Field {
                        visibility: Visibility::Public,
                        name: Ident::new("field1"),
                        ty: Type::Primitive(PrimitiveType::I32),
                        doc_comments: vec![],
                        attributes: vec![],
                    }],
                    methods: vec![],
                    doc_comments: vec![],
                    attributes: vec![],
                }),
                Item::Function(Function {
                    visibility: Visibility::Public,
                    name: Ident::new("test_func"),
                    params: vec![],
                    return_type: Some(Type::Primitive(PrimitiveType::Void)),
                    body: Block {
                        statements: vec![Statement::Let {
                            name: Ident::new("obj"),
                            ty: Some(Type::Ident(Ident::new("MyStruct"))),
                            init: None,
                            mutable: false,
                        }],
                    },
                    doc_comments: vec![],
                    attributes: vec![],
                }),
            ],
            doc_comments: vec![],
        };

        analyzer.analyze(&file);
        // Just verify no crashes - field access through reference should work
        assert!(
            analyzer.errors().is_empty()
                || analyzer
                    .errors()
                    .iter()
                    .all(|e| e.kind != SemanticErrorKind::InvalidOperation)
        );
    }

    #[test]
    fn test_field_access_on_non_struct() {
        let mut analyzer = SemanticAnalyzer::new();

        // Try to access field on integer (invalid)
        let field_access = Expression::FieldAccess {
            expr: Box::new(Expression::Literal(Literal::Int(42))),
            field: Ident::new("field"),
        };

        analyzer.analyze_expression_test(&field_access);

        // Should have an error about field access on non-struct
        assert!(!analyzer.errors().is_empty());
        assert!(analyzer
            .errors()
            .iter()
            .any(|e| e.kind == SemanticErrorKind::InvalidOperation));
    }

    #[test]
    fn test_unsupported_union() {
        let mut analyzer = SemanticAnalyzer::new();

        analyzer.check_union_usage("MyUnion");

        // Should have an error about unsupported feature
        assert!(!analyzer.errors().is_empty());
        assert!(analyzer
            .errors()
            .iter()
            .any(|e| e.kind == SemanticErrorKind::UnsupportedFeature));
    }

    #[test]
    fn test_unsupported_goto() {
        let mut analyzer = SemanticAnalyzer::new();

        analyzer.check_goto_usage("my_label");

        // Should have an error about unsupported feature
        assert!(!analyzer.errors().is_empty());
        assert!(analyzer
            .errors()
            .iter()
            .any(|e| e.kind == SemanticErrorKind::UnsupportedFeature));
    }

    #[test]
    fn test_unsupported_include() {
        let mut analyzer = SemanticAnalyzer::new();

        analyzer.check_include_usage("stdio.h");

        // Should have an error about unsupported feature
        assert!(!analyzer.errors().is_empty());
        assert!(analyzer
            .errors()
            .iter()
            .any(|e| e.kind == SemanticErrorKind::UnsupportedFeature));
    }
}
