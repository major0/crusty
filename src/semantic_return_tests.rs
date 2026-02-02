// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Tests for semantic analyzer return statement coverage

#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::error::SemanticErrorKind;
    use crate::semantic::SemanticAnalyzer;

    fn create_file_with_items(items: Vec<Item>) -> File {
        File {
            items,
            doc_comments: vec![],
        }
    }

    #[test]
    fn test_return_with_value() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: Some(Type::Primitive(PrimitiveType::Int)),
            body: Block::new(vec![Statement::Return(Some(Expression::Literal(
                Literal::Int(42),
            )))]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_return_without_value() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Return(None)]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_return_type_mismatch() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: Some(Type::Primitive(PrimitiveType::Int)),
            body: Block::new(vec![Statement::Return(Some(Expression::Literal(
                Literal::Bool(true),
            )))]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].kind, SemanticErrorKind::TypeMismatch);
    }

    #[test]
    fn test_return_void_when_expecting_value() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: Some(Type::Primitive(PrimitiveType::Int)),
            body: Block::new(vec![Statement::Return(None)]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].kind, SemanticErrorKind::TypeMismatch);
    }

    #[test]
    fn test_macro_definition_valid() {
        let mut analyzer = SemanticAnalyzer::new();

        let macro_def = Item::MacroDefinition(MacroDefinition {
            name: Ident::new("__MY_MACRO__"),
            params: vec![Ident::new("x")],
            body: vec![],
            delimiter: MacroDelimiter::Parens,
        });

        let file = create_file_with_items(vec![macro_def]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_macro_definition_invalid_name() {
        let mut analyzer = SemanticAnalyzer::new();

        let macro_def = Item::MacroDefinition(MacroDefinition {
            name: Ident::new("MY_MACRO"),
            params: vec![],
            body: vec![],
            delimiter: MacroDelimiter::None,
        });

        let file = create_file_with_items(vec![macro_def]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].kind, SemanticErrorKind::InvalidOperation);
    }

    #[test]
    fn test_function_with_invalid_name() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("__test__"),
            params: vec![],
            return_type: None,
            body: Block::empty(),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].kind, SemanticErrorKind::InvalidOperation);
    }

    #[test]
    fn test_undefined_variable() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Expr(Expression::Ident(Ident::new(
                "undefined_var",
            )))]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].kind, SemanticErrorKind::UndefinedVariable);
    }

    #[test]
    fn test_binary_op_type_mismatch() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Let {
                name: Ident::new("result"),
                ty: None,
                init: Some(Expression::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expression::Literal(Literal::Int(1))),
                    right: Box::new(Expression::Literal(Literal::Bool(true))),
                }),
                mutable: false,
            }]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].kind, SemanticErrorKind::TypeMismatch);
    }

    #[test]
    fn test_let_without_init() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Let {
                name: Ident::new("x"),
                ty: Some(Type::Primitive(PrimitiveType::Int)),
                init: None,
                mutable: false,
            }]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_var_without_init() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Var {
                name: Ident::new("x"),
                ty: Some(Type::Primitive(PrimitiveType::Int)),
                init: None,
            }]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_duplicate_var() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![
                Statement::Var {
                    name: Ident::new("x"),
                    ty: Some(Type::Primitive(PrimitiveType::Int)),
                    init: Some(Expression::Literal(Literal::Int(1))),
                },
                Statement::Var {
                    name: Ident::new("x"),
                    ty: Some(Type::Primitive(PrimitiveType::Int)),
                    init: Some(Expression::Literal(Literal::Int(2))),
                },
            ]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].kind, SemanticErrorKind::DuplicateDefinition);
    }

    #[test]
    fn test_duplicate_let() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![
                Statement::Let {
                    name: Ident::new("x"),
                    ty: Some(Type::Primitive(PrimitiveType::Int)),
                    init: Some(Expression::Literal(Literal::Int(1))),
                    mutable: false,
                },
                Statement::Let {
                    name: Ident::new("x"),
                    ty: Some(Type::Primitive(PrimitiveType::Int)),
                    init: Some(Expression::Literal(Literal::Int(2))),
                    mutable: false,
                },
            ]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].kind, SemanticErrorKind::DuplicateDefinition);
    }

    #[test]
    fn test_let_type_inference() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Let {
                name: Ident::new("x"),
                ty: None,
                init: Some(Expression::Literal(Literal::Int(42))),
                mutable: false,
            }]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_var_type_inference() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Var {
                name: Ident::new("x"),
                ty: None,
                init: Some(Expression::Literal(Literal::Int(42))),
            }]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_let_with_mutable() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Let {
                name: Ident::new("x"),
                ty: Some(Type::Primitive(PrimitiveType::Int)),
                init: Some(Expression::Literal(Literal::Int(42))),
                mutable: true,
            }]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_nested_function_with_invalid_name() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::NestedFunction {
                name: Ident::new("__nested__"),
                params: vec![],
                return_type: None,
                body: Block::empty(),
            }]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].kind, SemanticErrorKind::UnsupportedFeature);
    }
}
