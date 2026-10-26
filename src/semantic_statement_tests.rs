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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::Const {
                name: Ident::new("X"),
                ty: Type::Primitive(PrimitiveType::Int),
                value: Expression::Literal(Literal::Int(42)),
            }])),
            is_variadic: false,
            is_unsafe: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::Const {
                name: Ident::new("X"),
                ty: Type::Primitive(PrimitiveType::Int),
                value: Expression::Literal(Literal::Bool(true)),
            }])),
            is_variadic: false,
            is_unsafe: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![
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
            ])),
            is_variadic: false,
            is_unsafe: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::If {
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
            }])),
            is_variadic: false,
            is_unsafe: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::If {
                condition: Expression::Literal(Literal::Int(42)),
                then_block: Block::empty(),
                else_block: None,
            }])),
            is_variadic: false,
            is_unsafe: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::While {
                label: None,
                condition: Expression::Literal(Literal::Int(1)),
                body: Block::empty(),
            }])),
            is_variadic: false,
            is_unsafe: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::For {
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
            }])),
            is_variadic: false,
            is_unsafe: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![
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
            ])),
            is_variadic: false,
            is_unsafe: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::Switch {
                expr: Expression::Literal(Literal::Int(1)),
                cases: vec![SwitchCase {
                    values: vec![Expression::Literal(Literal::Int(1))],
                    body: Block::empty(),
                }],
                default: Some(Block::empty()),
            }])),
            is_variadic: false,
            is_unsafe: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::Switch {
                expr: Expression::Literal(Literal::Int(1)),
                cases: vec![SwitchCase {
                    values: vec![Expression::Literal(Literal::Bool(true))],
                    body: Block::empty(),
                }],
                default: None,
            }])),
            is_variadic: false,
            is_unsafe: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::While {
                label: None,
                condition: Expression::Literal(Literal::Bool(true)),
                body: Block::new(vec![Statement::Break { label: None, value: None }]),
            }])),
            is_variadic: false,
            is_unsafe: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::While {
                label: None,
                condition: Expression::Literal(Literal::Bool(true)),
                body: Block::new(vec![Statement::Continue(None)]),
            }])),
            is_variadic: false,
            is_unsafe: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("Status"),
            variants: vec![EnumVariant {
                name: Ident::new("Ok"),
                body: VariantBody::Unit,
            }],
            doc_comments: vec![],
            attributes: vec![],
        });

        let enum2 = Item::Enum(Enum {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("Status"),
            variants: vec![EnumVariant {
                name: Ident::new("Error"),
                body: VariantBody::Unit,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("Point"),
            body: StructBody::Named(vec![]),
            methods: vec![],
            doc_comments: vec![],
            attributes: vec![],
        });

        let struct2 = Item::Struct(Struct {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("Point"),
            body: StructBody::Named(vec![]),
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::While {
                label: Some(Ident::new("outer")),
                condition: Expression::Literal(Literal::Bool(true)),
                body: Block::new(vec![Statement::Break { label: Some(Ident::new("outer")), value: None }]),
            }])),
            is_variadic: false,
            is_unsafe: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::For {
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
            }])),
            is_variadic: false,
            is_unsafe: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::ForIn {
                label: Some(Ident::new("iter")),
                var: Ident::new("i"),
                iter: Expression::Range {
                    start: Some(Box::new(Expression::Literal(Literal::Int(0)))),
                    end: Some(Box::new(Expression::Literal(Literal::Int(10)))),
                    inclusive: false,
                },
                body: Block::new(vec![Statement::Break { label: Some(Ident::new("iter")), value: None }]),
            }])),
            is_variadic: false,
            is_unsafe: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::Switch {
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
            }])),
            is_variadic: false,
            is_unsafe: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::Var {
                name: Ident::new("x"),
                ty: Some(Type::Primitive(PrimitiveType::Int)),
                init: Some(Expression::Literal(Literal::Int(42))),
            }])),
            is_variadic: false,
            is_unsafe: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::Var {
                name: Ident::new("x"),
                ty: Some(Type::Primitive(PrimitiveType::Int)),
                init: Some(Expression::Literal(Literal::Bool(true))),
            }])),
            is_variadic: false,
            is_unsafe: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::For {
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
            }])),
            is_variadic: false,
            is_unsafe: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::ForIn {
                label: None,
                var: Ident::new("i"),
                iter: Expression::Range {
                    start: Some(Box::new(Expression::Literal(Literal::Int(0)))),
                    end: Some(Box::new(Expression::Literal(Literal::Int(10)))),
                    inclusive: false,
                },
                body: Block::new(vec![Statement::Expr(Expression::Ident(Ident::new("i")))]),
            }])),
            is_variadic: false,
            is_unsafe: false,
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_for_in_loop_binds_slice_element_type() {
        // Regression: iterating a slice must bind the loop variable to the
        // ELEMENT type, not the slice type. Here the loop sums `acc + x` where
        // `acc` is an int and `x` comes from iterating an `int[]`; if `x` were
        // bound to the slice type this would be a type mismatch.
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("sum"),
            params: vec![Param {
                name: Ident::new("xs"),
                ty: Type::Slice {
                    ty: Box::new(Type::Primitive(PrimitiveType::I32)),
                },
            }],
            return_type: Some(Type::Primitive(PrimitiveType::I32)),
            body: Some(Block::new(vec![
                Statement::Let {
                    name: Ident::new("acc"),
                    ty: Some(Type::Primitive(PrimitiveType::I32)),
                    init: Some(Expression::Literal(Literal::Int(0))),
                    mutable: true,
                },
                Statement::ForIn {
                    label: None,
                    var: Ident::new("x"),
                    iter: Expression::Ident(Ident::new("xs")),
                    body: Block::new(vec![Statement::Expr(Expression::Binary {
                        op: BinaryOp::Assign,
                        left: Box::new(Expression::Ident(Ident::new("acc"))),
                        right: Box::new(Expression::Binary {
                            op: BinaryOp::Add,
                            left: Box::new(Expression::Ident(Ident::new("acc"))),
                            right: Box::new(Expression::Ident(Ident::new("x"))),
                        }),
                    })]),
                },
                Statement::Return(Some(Expression::Ident(Ident::new("acc")))),
            ])),
            is_variadic: false,
            is_unsafe: false,
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(
            result.is_ok(),
            "slice element should bind as the element type, not the slice: {:?}",
            result
        );
    }

    #[test]
    fn test_for_in_loop_binds_vec_element_type() {
        // Iterating a `Vec<int>` binds the loop variable to `int` (the single
        // type argument), so `acc + x` type-checks — not the collection type.
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("sum"),
            params: vec![Param {
                name: Ident::new("xs"),
                ty: Type::Generic {
                    base: Box::new(Type::Ident(Ident::new("Vec"))),
                    args: vec![Type::Primitive(PrimitiveType::I32)],
                },
            }],
            return_type: Some(Type::Primitive(PrimitiveType::I32)),
            body: Some(Block::new(vec![
                Statement::Let {
                    name: Ident::new("acc"),
                    ty: Some(Type::Primitive(PrimitiveType::I32)),
                    init: Some(Expression::Literal(Literal::Int(0))),
                    mutable: true,
                },
                Statement::ForIn {
                    label: None,
                    var: Ident::new("x"),
                    iter: Expression::Ident(Ident::new("xs")),
                    body: Block::new(vec![Statement::Expr(Expression::Binary {
                        op: BinaryOp::Assign,
                        left: Box::new(Expression::Ident(Ident::new("acc"))),
                        right: Box::new(Expression::Binary {
                            op: BinaryOp::Add,
                            left: Box::new(Expression::Ident(Ident::new("acc"))),
                            right: Box::new(Expression::Ident(Ident::new("x"))),
                        }),
                    })]),
                },
                Statement::Return(Some(Expression::Ident(Ident::new("acc")))),
            ])),
            is_variadic: false,
            is_unsafe: false,
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(
            result.is_ok(),
            "Vec<int> element should bind as int: {:?}",
            result
        );
    }

    #[test]
    fn test_for_in_loop_binds_map_pair_type() {
        // Iterating a `HashMap<String, int>` binds the loop variable to the
        // `(String, int)` pair, matching Rust's `for (k, v) in map`. The loop
        // just references the bound variable, which must analyze cleanly.
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("walk"),
            params: vec![Param {
                name: Ident::new("m"),
                ty: Type::Generic {
                    base: Box::new(Type::Ident(Ident::new("HashMap"))),
                    args: vec![
                        Type::Ident(Ident::new("String")),
                        Type::Primitive(PrimitiveType::I32),
                    ],
                },
            }],
            return_type: None,
            body: Some(Block::new(vec![Statement::ForIn {
                label: None,
                var: Ident::new("entry"),
                iter: Expression::Ident(Ident::new("m")),
                body: Block::new(vec![Statement::Expr(Expression::Ident(Ident::new(
                    "entry",
                )))]),
            }])),
            is_variadic: false,
            is_unsafe: false,
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(
            result.is_ok(),
            "HashMap<String, int> iteration should bind the (String, int) pair: {:?}",
            result
        );
    }

    #[test]
    fn test_switch_without_default() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::Switch {
                expr: Expression::Literal(Literal::Int(1)),
                cases: vec![SwitchCase {
                    values: vec![Expression::Literal(Literal::Int(1))],
                    body: Block::empty(),
                }],
                default: None,
            }])),
            is_variadic: false,
            is_unsafe: false,
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }
}
