// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Tests for semantic analyzer statement coverage

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
    fn test_const_statement_in_function() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Const {
                name: Ident::new("X"),
                ty: Type::Primitive(PrimitiveType::Int),
                value: Expression::Literal(Literal::Int(42)),
            }]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_const_statement_type_mismatch() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Const {
                name: Ident::new("X"),
                ty: Type::Primitive(PrimitiveType::Int),
                value: Expression::Literal(Literal::Bool(true)),
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
    fn test_duplicate_const_in_function() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![
                Statement::Const {
                    name: Ident::new("X"),
                    ty: Type::Primitive(PrimitiveType::Int),
                    value: Expression::Literal(Literal::Int(42)),
                },
                Statement::Const {
                    name: Ident::new("X"),
                    ty: Type::Primitive(PrimitiveType::Int),
                    value: Expression::Literal(Literal::Int(100)),
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
    fn test_if_with_else_block() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::If {
                condition: Expression::Literal(Literal::Bool(true)),
                then_block: Block::new(vec![Statement::Let {
                    name: Ident::new("x"),
                    ty: Some(Type::Primitive(PrimitiveType::Int)),
                    init: Some(Expression::Literal(Literal::Int(1))),
                    mutable: false,
                }]),
                else_block: Some(Block::new(vec![Statement::Let {
                    name: Ident::new("y"),
                    ty: Some(Type::Primitive(PrimitiveType::Int)),
                    init: Some(Expression::Literal(Literal::Int(2))),
                    mutable: false,
                }])),
            }]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_if_condition_not_boolean() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::If {
                condition: Expression::Literal(Literal::Int(42)),
                then_block: Block::empty(),
                else_block: None,
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
    fn test_while_condition_not_boolean() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::While {
                label: None,
                condition: Expression::Literal(Literal::Int(1)),
                body: Block::empty(),
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
    fn test_for_loop_condition_not_boolean() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::For {
                label: None,
                init: Box::new(Statement::Let {
                    name: Ident::new("i"),
                    ty: Some(Type::Primitive(PrimitiveType::Int)),
                    init: Some(Expression::Literal(Literal::Int(0))),
                    mutable: true,
                }),
                condition: Expression::Literal(Literal::Int(10)),
                increment: Expression::Binary {
                    op: BinaryOp::AddAssign,
                    left: Box::new(Expression::Ident(Ident::new("i"))),
                    right: Box::new(Expression::Literal(Literal::Int(1))),
                },
                body: Block::empty(),
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
    fn test_for_in_loop_with_duplicate_var() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![
                Statement::Let {
                    name: Ident::new("i"),
                    ty: Some(Type::Primitive(PrimitiveType::Int)),
                    init: Some(Expression::Literal(Literal::Int(0))),
                    mutable: false,
                },
                Statement::ForIn {
                    label: None,
                    var: Ident::new("i"),
                    iter: Expression::Range {
                        start: Some(Box::new(Expression::Literal(Literal::Int(0)))),
                        end: Some(Box::new(Expression::Literal(Literal::Int(10)))),
                        inclusive: false,
                    },
                    body: Block::empty(),
                },
            ]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        // This should succeed because for-in creates a new scope
        assert!(result.is_ok());
    }

    #[test]
    fn test_switch_with_default() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Switch {
                expr: Expression::Literal(Literal::Int(1)),
                cases: vec![SwitchCase {
                    values: vec![Expression::Literal(Literal::Int(1))],
                    body: Block::empty(),
                }],
                default: Some(Block::empty()),
            }]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_switch_case_type_mismatch() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Switch {
                expr: Expression::Literal(Literal::Int(1)),
                cases: vec![SwitchCase {
                    values: vec![Expression::Literal(Literal::Bool(true))],
                    body: Block::empty(),
                }],
                default: None,
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
    fn test_break_statement() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::While {
                label: None,
                condition: Expression::Literal(Literal::Bool(true)),
                body: Block::new(vec![Statement::Break(None)]),
            }]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_continue_statement() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::While {
                label: None,
                condition: Expression::Literal(Literal::Bool(true)),
                body: Block::new(vec![Statement::Continue(None)]),
            }]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_duplicate_enum() {
        let mut analyzer = SemanticAnalyzer::new();

        let enum1 = Item::Enum(Enum {
            visibility: Visibility::Public,
            name: Ident::new("Status"),
            variants: vec![EnumVariant {
                name: Ident::new("Ok"),
                value: None,
            }],
            doc_comments: vec![],
            attributes: vec![],
        });

        let enum2 = Item::Enum(Enum {
            visibility: Visibility::Public,
            name: Ident::new("Status"),
            variants: vec![EnumVariant {
                name: Ident::new("Error"),
                value: None,
            }],
            doc_comments: vec![],
            attributes: vec![],
        });

        let file = create_file_with_items(vec![enum1, enum2]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].kind, SemanticErrorKind::DuplicateDefinition);
    }

    #[test]
    fn test_duplicate_struct() {
        let mut analyzer = SemanticAnalyzer::new();

        let struct1 = Item::Struct(Struct {
            visibility: Visibility::Public,
            name: Ident::new("Point"),
            fields: vec![],
            methods: vec![],
            doc_comments: vec![],
            attributes: vec![],
        });

        let struct2 = Item::Struct(Struct {
            visibility: Visibility::Public,
            name: Ident::new("Point"),
            fields: vec![],
            methods: vec![],
            doc_comments: vec![],
            attributes: vec![],
        });

        let file = create_file_with_items(vec![struct1, struct2]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].kind, SemanticErrorKind::DuplicateDefinition);
    }

    #[test]
    fn test_duplicate_typedef() {
        let mut analyzer = SemanticAnalyzer::new();

        let typedef1 = Item::Typedef(Typedef {
            visibility: Visibility::Public,
            name: Ident::new("MyInt"),
            target: Type::Primitive(PrimitiveType::Int),
            doc_comments: vec![],
        });

        let typedef2 = Item::Typedef(Typedef {
            visibility: Visibility::Public,
            name: Ident::new("MyInt"),
            target: Type::Primitive(PrimitiveType::I32),
            doc_comments: vec![],
        });

        let file = create_file_with_items(vec![typedef1, typedef2]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].kind, SemanticErrorKind::DuplicateDefinition);
    }

    #[test]
    fn test_duplicate_const() {
        let mut analyzer = SemanticAnalyzer::new();

        let const1 = Item::Const(Const {
            visibility: Visibility::Public,
            name: Ident::new("MAX"),
            ty: Type::Primitive(PrimitiveType::Int),
            value: Expression::Literal(Literal::Int(100)),
            doc_comments: vec![],
        });

        let const2 = Item::Const(Const {
            visibility: Visibility::Public,
            name: Ident::new("MAX"),
            ty: Type::Primitive(PrimitiveType::Int),
            value: Expression::Literal(Literal::Int(200)),
            doc_comments: vec![],
        });

        let file = create_file_with_items(vec![const1, const2]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].kind, SemanticErrorKind::DuplicateDefinition);
    }

    #[test]
    fn test_duplicate_static() {
        let mut analyzer = SemanticAnalyzer::new();

        let static1 = Item::Static(Static {
            visibility: Visibility::Public,
            name: Ident::new("COUNTER"),
            ty: Type::Primitive(PrimitiveType::Int),
            value: Expression::Literal(Literal::Int(0)),
            mutable: true,
            doc_comments: vec![],
        });

        let static2 = Item::Static(Static {
            visibility: Visibility::Public,
            name: Ident::new("COUNTER"),
            ty: Type::Primitive(PrimitiveType::Int),
            value: Expression::Literal(Literal::Int(1)),
            mutable: true,
            doc_comments: vec![],
        });

        let file = create_file_with_items(vec![static1, static2]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].kind, SemanticErrorKind::DuplicateDefinition);
    }

    #[test]
    fn test_while_loop_with_label() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::While {
                label: Some(Ident::new("outer")),
                condition: Expression::Literal(Literal::Bool(true)),
                body: Block::new(vec![Statement::Break(Some(Ident::new("outer")))]),
            }]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_for_loop_with_label() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::For {
                label: Some(Ident::new("loop1")),
                init: Box::new(Statement::Let {
                    name: Ident::new("i"),
                    ty: Some(Type::Primitive(PrimitiveType::Int)),
                    init: Some(Expression::Literal(Literal::Int(0))),
                    mutable: true,
                }),
                condition: Expression::Binary {
                    op: BinaryOp::Lt,
                    left: Box::new(Expression::Ident(Ident::new("i"))),
                    right: Box::new(Expression::Literal(Literal::Int(10))),
                },
                increment: Expression::Binary {
                    op: BinaryOp::AddAssign,
                    left: Box::new(Expression::Ident(Ident::new("i"))),
                    right: Box::new(Expression::Literal(Literal::Int(1))),
                },
                body: Block::new(vec![Statement::Continue(Some(Ident::new("loop1")))]),
            }]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_for_in_loop_with_label() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::ForIn {
                label: Some(Ident::new("iter")),
                var: Ident::new("i"),
                iter: Expression::Range {
                    start: Some(Box::new(Expression::Literal(Literal::Int(0)))),
                    end: Some(Box::new(Expression::Literal(Literal::Int(10)))),
                    inclusive: false,
                },
                body: Block::new(vec![Statement::Break(Some(Ident::new("iter")))]),
            }]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_switch_multiple_cases() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Switch {
                expr: Expression::Literal(Literal::Int(1)),
                cases: vec![
                    SwitchCase {
                        values: vec![Expression::Literal(Literal::Int(1))],
                        body: Block::empty(),
                    },
                    SwitchCase {
                        values: vec![Expression::Literal(Literal::Int(2))],
                        body: Block::empty(),
                    },
                    SwitchCase {
                        values: vec![Expression::Literal(Literal::Int(3))],
                        body: Block::empty(),
                    },
                ],
                default: Some(Block::empty()),
            }]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_var_statement() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Var {
                name: Ident::new("x"),
                ty: Some(Type::Primitive(PrimitiveType::Int)),
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
    fn test_var_statement_type_mismatch() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Var {
                name: Ident::new("x"),
                ty: Some(Type::Primitive(PrimitiveType::Int)),
                init: Some(Expression::Literal(Literal::Bool(true))),
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
    fn test_for_loop_with_var_init() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::For {
                label: None,
                init: Box::new(Statement::Var {
                    name: Ident::new("i"),
                    ty: Some(Type::Primitive(PrimitiveType::Int)),
                    init: Some(Expression::Literal(Literal::Int(0))),
                }),
                condition: Expression::Binary {
                    op: BinaryOp::Lt,
                    left: Box::new(Expression::Ident(Ident::new("i"))),
                    right: Box::new(Expression::Literal(Literal::Int(10))),
                },
                increment: Expression::Binary {
                    op: BinaryOp::AddAssign,
                    left: Box::new(Expression::Ident(Ident::new("i"))),
                    right: Box::new(Expression::Literal(Literal::Int(1))),
                },
                body: Block::empty(),
            }]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_for_in_loop_basic() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::ForIn {
                label: None,
                var: Ident::new("i"),
                iter: Expression::Range {
                    start: Some(Box::new(Expression::Literal(Literal::Int(0)))),
                    end: Some(Box::new(Expression::Literal(Literal::Int(10)))),
                    inclusive: false,
                },
                body: Block::new(vec![Statement::Expr(Expression::Ident(Ident::new("i")))]),
            }]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_switch_without_default() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Switch {
                expr: Expression::Literal(Literal::Int(1)),
                cases: vec![SwitchCase {
                    values: vec![Expression::Literal(Literal::Int(1))],
                    body: Block::empty(),
                }],
                default: None,
            }]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }
}
