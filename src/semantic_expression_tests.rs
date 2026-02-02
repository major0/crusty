// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Tests for semantic analyzer expression coverage

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

    fn create_test_function(body: Vec<Statement>) -> Function {
        Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(body),
            doc_comments: vec![],
            attributes: vec![],
        }
    }

    #[test]
    fn test_binary_op_arithmetic() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = create_test_function(vec![Statement::Let {
            name: Ident::new("result"),
            ty: Some(Type::Primitive(PrimitiveType::Int)),
            init: Some(Expression::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expression::Literal(Literal::Int(1))),
                right: Box::new(Expression::Literal(Literal::Int(2))),
            }),
            mutable: false,
        }]);

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_binary_op_comparison() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = create_test_function(vec![Statement::Let {
            name: Ident::new("result"),
            ty: Some(Type::Primitive(PrimitiveType::Bool)),
            init: Some(Expression::Binary {
                op: BinaryOp::Lt,
                left: Box::new(Expression::Literal(Literal::Int(1))),
                right: Box::new(Expression::Literal(Literal::Int(2))),
            }),
            mutable: false,
        }]);

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_binary_op_bitwise() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = create_test_function(vec![Statement::Let {
            name: Ident::new("result"),
            ty: Some(Type::Primitive(PrimitiveType::Int)),
            init: Some(Expression::Binary {
                op: BinaryOp::BitAnd,
                left: Box::new(Expression::Literal(Literal::Int(5))),
                right: Box::new(Expression::Literal(Literal::Int(3))),
            }),
            mutable: false,
        }]);

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unary_op_not() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = create_test_function(vec![Statement::Let {
            name: Ident::new("result"),
            ty: Some(Type::Primitive(PrimitiveType::Bool)),
            init: Some(Expression::Unary {
                op: UnaryOp::Not,
                expr: Box::new(Expression::Literal(Literal::Bool(true))),
            }),
            mutable: false,
        }]);

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unary_op_neg() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = create_test_function(vec![Statement::Let {
            name: Ident::new("result"),
            ty: Some(Type::Primitive(PrimitiveType::Int)),
            init: Some(Expression::Unary {
                op: UnaryOp::Neg,
                expr: Box::new(Expression::Literal(Literal::Int(42))),
            }),
            mutable: false,
        }]);

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unary_op_ref() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = create_test_function(vec![
            Statement::Let {
                name: Ident::new("x"),
                ty: Some(Type::Primitive(PrimitiveType::Int)),
                init: Some(Expression::Literal(Literal::Int(42))),
                mutable: false,
            },
            Statement::Let {
                name: Ident::new("ptr"),
                ty: None,
                init: Some(Expression::Unary {
                    op: UnaryOp::Ref,
                    expr: Box::new(Expression::Ident(Ident::new("x"))),
                }),
                mutable: false,
            },
        ]);

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unary_op_deref() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = create_test_function(vec![
            Statement::Let {
                name: Ident::new("ptr"),
                ty: Some(Type::Reference {
                    ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                    mutable: false,
                }),
                init: None,
                mutable: false,
            },
            Statement::Let {
                name: Ident::new("value"),
                ty: None,
                init: Some(Expression::Unary {
                    op: UnaryOp::Deref,
                    expr: Box::new(Expression::Ident(Ident::new("ptr"))),
                }),
                mutable: false,
            },
        ]);

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unary_op_deref_invalid() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = create_test_function(vec![
            Statement::Let {
                name: Ident::new("x"),
                ty: Some(Type::Primitive(PrimitiveType::Int)),
                init: Some(Expression::Literal(Literal::Int(42))),
                mutable: false,
            },
            Statement::Let {
                name: Ident::new("value"),
                ty: None,
                init: Some(Expression::Unary {
                    op: UnaryOp::Deref,
                    expr: Box::new(Expression::Ident(Ident::new("x"))),
                }),
                mutable: false,
            },
        ]);

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].kind, SemanticErrorKind::InvalidOperation);
    }

    #[test]
    fn test_unary_op_preinc() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = create_test_function(vec![
            Statement::Var {
                name: Ident::new("x"),
                ty: Some(Type::Primitive(PrimitiveType::Int)),
                init: Some(Expression::Literal(Literal::Int(0))),
            },
            Statement::Expr(Expression::Unary {
                op: UnaryOp::PreInc,
                expr: Box::new(Expression::Ident(Ident::new("x"))),
            }),
        ]);

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_expression() {
        let mut analyzer = SemanticAnalyzer::new();

        let add_func = Function {
            visibility: Visibility::Public,
            name: Ident::new("add"),
            params: vec![
                Param {
                    name: Ident::new("a"),
                    ty: Type::Primitive(PrimitiveType::Int),
                },
                Param {
                    name: Ident::new("b"),
                    ty: Type::Primitive(PrimitiveType::Int),
                },
            ],
            return_type: Some(Type::Primitive(PrimitiveType::Int)),
            body: Block::empty(),
            doc_comments: vec![],
            attributes: vec![],
        };

        let test_func = create_test_function(vec![Statement::Let {
            name: Ident::new("result"),
            ty: None,
            init: Some(Expression::Call {
                func: Box::new(Expression::Ident(Ident::new("add"))),
                args: vec![
                    Expression::Literal(Literal::Int(1)),
                    Expression::Literal(Literal::Int(2)),
                ],
            }),
            mutable: false,
        }]);

        let file =
            create_file_with_items(vec![Item::Function(add_func), Item::Function(test_func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_call_expression_wrong_arg_count() {
        let mut analyzer = SemanticAnalyzer::new();

        let add_func = Function {
            visibility: Visibility::Public,
            name: Ident::new("add"),
            params: vec![
                Param {
                    name: Ident::new("a"),
                    ty: Type::Primitive(PrimitiveType::Int),
                },
                Param {
                    name: Ident::new("b"),
                    ty: Type::Primitive(PrimitiveType::Int),
                },
            ],
            return_type: Some(Type::Primitive(PrimitiveType::Int)),
            body: Block::empty(),
            doc_comments: vec![],
            attributes: vec![],
        };

        let test_func = create_test_function(vec![Statement::Expr(Expression::Call {
            func: Box::new(Expression::Ident(Ident::new("add"))),
            args: vec![Expression::Literal(Literal::Int(1))],
        })]);

        let file =
            create_file_with_items(vec![Item::Function(add_func), Item::Function(test_func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].kind, SemanticErrorKind::TypeMismatch);
    }

    #[test]
    fn test_call_expression_wrong_arg_type() {
        let mut analyzer = SemanticAnalyzer::new();

        let add_func = Function {
            visibility: Visibility::Public,
            name: Ident::new("add"),
            params: vec![
                Param {
                    name: Ident::new("a"),
                    ty: Type::Primitive(PrimitiveType::Int),
                },
                Param {
                    name: Ident::new("b"),
                    ty: Type::Primitive(PrimitiveType::Int),
                },
            ],
            return_type: Some(Type::Primitive(PrimitiveType::Int)),
            body: Block::empty(),
            doc_comments: vec![],
            attributes: vec![],
        };

        let test_func = create_test_function(vec![Statement::Expr(Expression::Call {
            func: Box::new(Expression::Ident(Ident::new("add"))),
            args: vec![
                Expression::Literal(Literal::Int(1)),
                Expression::Literal(Literal::Bool(true)),
            ],
        })]);

        let file =
            create_file_with_items(vec![Item::Function(add_func), Item::Function(test_func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].kind, SemanticErrorKind::TypeMismatch);
    }

    #[test]
    fn test_call_non_function() {
        let mut analyzer = SemanticAnalyzer::new();

        let test_func = create_test_function(vec![
            Statement::Let {
                name: Ident::new("x"),
                ty: Some(Type::Primitive(PrimitiveType::Int)),
                init: Some(Expression::Literal(Literal::Int(42))),
                mutable: false,
            },
            Statement::Expr(Expression::Call {
                func: Box::new(Expression::Ident(Ident::new("x"))),
                args: vec![],
            }),
        ]);

        let file = create_file_with_items(vec![Item::Function(test_func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].kind, SemanticErrorKind::InvalidOperation);
    }

    #[test]
    fn test_field_access() {
        let mut analyzer = SemanticAnalyzer::new();

        let point_struct = Struct {
            visibility: Visibility::Public,
            name: Ident::new("Point"),
            fields: vec![
                Field {
                    visibility: Visibility::Public,
                    name: Ident::new("x"),
                    ty: Type::Primitive(PrimitiveType::Int),
                    doc_comments: vec![],
                    attributes: vec![],
                },
                Field {
                    visibility: Visibility::Public,
                    name: Ident::new("y"),
                    ty: Type::Primitive(PrimitiveType::Int),
                    doc_comments: vec![],
                    attributes: vec![],
                },
            ],
            methods: vec![],
            doc_comments: vec![],
            attributes: vec![],
        };

        let test_func = create_test_function(vec![
            Statement::Let {
                name: Ident::new("p"),
                ty: Some(Type::Ident(Ident::new("Point"))),
                init: Some(Expression::StructInit {
                    ty: Type::Ident(Ident::new("Point")),
                    fields: vec![
                        (Ident::new("x"), Expression::Literal(Literal::Int(1))),
                        (Ident::new("y"), Expression::Literal(Literal::Int(2))),
                    ],
                }),
                mutable: false,
            },
            Statement::Let {
                name: Ident::new("x_val"),
                ty: None,
                init: Some(Expression::FieldAccess {
                    expr: Box::new(Expression::Ident(Ident::new("p"))),
                    field: Ident::new("x"),
                }),
                mutable: false,
            },
        ]);

        let file =
            create_file_with_items(vec![Item::Struct(point_struct), Item::Function(test_func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_field_access_invalid_field() {
        let mut analyzer = SemanticAnalyzer::new();

        let point_struct = Struct {
            visibility: Visibility::Public,
            name: Ident::new("Point"),
            fields: vec![Field {
                visibility: Visibility::Public,
                name: Ident::new("x"),
                ty: Type::Primitive(PrimitiveType::Int),
                doc_comments: vec![],
                attributes: vec![],
            }],
            methods: vec![],
            doc_comments: vec![],
            attributes: vec![],
        };

        let test_func = create_test_function(vec![
            Statement::Let {
                name: Ident::new("p"),
                ty: Some(Type::Ident(Ident::new("Point"))),
                init: Some(Expression::StructInit {
                    ty: Type::Ident(Ident::new("Point")),
                    fields: vec![(Ident::new("x"), Expression::Literal(Literal::Int(1)))],
                }),
                mutable: false,
            },
            Statement::Expr(Expression::FieldAccess {
                expr: Box::new(Expression::Ident(Ident::new("p"))),
                field: Ident::new("z"),
            }),
        ]);

        let file =
            create_file_with_items(vec![Item::Struct(point_struct), Item::Function(test_func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].kind, SemanticErrorKind::InvalidOperation);
    }

    #[test]
    fn test_field_access_non_struct() {
        let mut analyzer = SemanticAnalyzer::new();

        let test_func = create_test_function(vec![
            Statement::Let {
                name: Ident::new("x"),
                ty: Some(Type::Primitive(PrimitiveType::Int)),
                init: Some(Expression::Literal(Literal::Int(42))),
                mutable: false,
            },
            Statement::Expr(Expression::FieldAccess {
                expr: Box::new(Expression::Ident(Ident::new("x"))),
                field: Ident::new("value"),
            }),
        ]);

        let file = create_file_with_items(vec![Item::Function(test_func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].kind, SemanticErrorKind::InvalidOperation);
    }

    #[test]
    fn test_index_expression() {
        let mut analyzer = SemanticAnalyzer::new();

        let test_func = create_test_function(vec![
            Statement::Let {
                name: Ident::new("arr"),
                ty: Some(Type::Slice {
                    ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                }),
                init: None,
                mutable: false,
            },
            Statement::Let {
                name: Ident::new("val"),
                ty: None,
                init: Some(Expression::Index {
                    expr: Box::new(Expression::Ident(Ident::new("arr"))),
                    index: Box::new(Expression::Literal(Literal::Int(0))),
                }),
                mutable: false,
            },
        ]);

        let file = create_file_with_items(vec![Item::Function(test_func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_index_expression_invalid_index_type() {
        let mut analyzer = SemanticAnalyzer::new();

        let test_func = create_test_function(vec![
            Statement::Let {
                name: Ident::new("arr"),
                ty: Some(Type::Array {
                    ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                    size: Some(5),
                }),
                init: Some(Expression::ArrayLit {
                    elements: vec![Expression::Literal(Literal::Int(1))],
                }),
                mutable: false,
            },
            Statement::Expr(Expression::Index {
                expr: Box::new(Expression::Ident(Ident::new("arr"))),
                index: Box::new(Expression::Literal(Literal::Bool(true))),
            }),
        ]);

        let file = create_file_with_items(vec![Item::Function(test_func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].kind, SemanticErrorKind::TypeMismatch);
    }

    #[test]
    fn test_index_expression_non_array() {
        let mut analyzer = SemanticAnalyzer::new();

        let test_func = create_test_function(vec![
            Statement::Let {
                name: Ident::new("x"),
                ty: Some(Type::Primitive(PrimitiveType::Int)),
                init: Some(Expression::Literal(Literal::Int(42))),
                mutable: false,
            },
            Statement::Expr(Expression::Index {
                expr: Box::new(Expression::Ident(Ident::new("x"))),
                index: Box::new(Expression::Literal(Literal::Int(0))),
            }),
        ]);

        let file = create_file_with_items(vec![Item::Function(test_func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].kind, SemanticErrorKind::InvalidOperation);
    }

    #[test]
    fn test_cast_expression() {
        let mut analyzer = SemanticAnalyzer::new();

        let test_func = create_test_function(vec![Statement::Let {
            name: Ident::new("x"),
            ty: None,
            init: Some(Expression::Cast {
                expr: Box::new(Expression::Literal(Literal::Int(42))),
                ty: Type::Primitive(PrimitiveType::F64),
            }),
            mutable: false,
        }]);

        let file = create_file_with_items(vec![Item::Function(test_func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sizeof_expression() {
        let mut analyzer = SemanticAnalyzer::new();

        let test_func = create_test_function(vec![Statement::Let {
            name: Ident::new("size"),
            ty: None,
            init: Some(Expression::Sizeof {
                ty: Type::Primitive(PrimitiveType::Int),
            }),
            mutable: false,
        }]);

        let file = create_file_with_items(vec![Item::Function(test_func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ternary_expression() {
        let mut analyzer = SemanticAnalyzer::new();

        let test_func = create_test_function(vec![Statement::Let {
            name: Ident::new("result"),
            ty: None,
            init: Some(Expression::Ternary {
                condition: Box::new(Expression::Literal(Literal::Bool(true))),
                then_expr: Box::new(Expression::Literal(Literal::Int(1))),
                else_expr: Box::new(Expression::Literal(Literal::Int(2))),
            }),
            mutable: false,
        }]);

        let file = create_file_with_items(vec![Item::Function(test_func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ternary_condition_not_bool() {
        let mut analyzer = SemanticAnalyzer::new();

        let test_func = create_test_function(vec![Statement::Let {
            name: Ident::new("result"),
            ty: None,
            init: Some(Expression::Ternary {
                condition: Box::new(Expression::Literal(Literal::Int(1))),
                then_expr: Box::new(Expression::Literal(Literal::Int(1))),
                else_expr: Box::new(Expression::Literal(Literal::Int(2))),
            }),
            mutable: false,
        }]);

        let file = create_file_with_items(vec![Item::Function(test_func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].kind, SemanticErrorKind::TypeMismatch);
    }

    #[test]
    fn test_ternary_branch_type_mismatch() {
        let mut analyzer = SemanticAnalyzer::new();

        let test_func = create_test_function(vec![Statement::Let {
            name: Ident::new("result"),
            ty: None,
            init: Some(Expression::Ternary {
                condition: Box::new(Expression::Literal(Literal::Bool(true))),
                then_expr: Box::new(Expression::Literal(Literal::Int(1))),
                else_expr: Box::new(Expression::Literal(Literal::Bool(false))),
            }),
            mutable: false,
        }]);

        let file = create_file_with_items(vec![Item::Function(test_func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].kind, SemanticErrorKind::TypeMismatch);
    }

    #[test]
    fn test_array_literal() {
        let mut analyzer = SemanticAnalyzer::new();

        let test_func = create_test_function(vec![Statement::Let {
            name: Ident::new("arr"),
            ty: None,
            init: Some(Expression::ArrayLit {
                elements: vec![
                    Expression::Literal(Literal::Int(1)),
                    Expression::Literal(Literal::Int(2)),
                    Expression::Literal(Literal::Int(3)),
                ],
            }),
            mutable: false,
        }]);

        let file = create_file_with_items(vec![Item::Function(test_func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_array_literal_type_mismatch() {
        let mut analyzer = SemanticAnalyzer::new();

        let test_func = create_test_function(vec![Statement::Let {
            name: Ident::new("arr"),
            ty: None,
            init: Some(Expression::ArrayLit {
                elements: vec![
                    Expression::Literal(Literal::Int(1)),
                    Expression::Literal(Literal::Bool(true)),
                ],
            }),
            mutable: false,
        }]);

        let file = create_file_with_items(vec![Item::Function(test_func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].kind, SemanticErrorKind::TypeMismatch);
    }

    #[test]
    fn test_tuple_literal() {
        let mut analyzer = SemanticAnalyzer::new();

        let test_func = create_test_function(vec![Statement::Let {
            name: Ident::new("tup"),
            ty: None,
            init: Some(Expression::TupleLit {
                elements: vec![
                    Expression::Literal(Literal::Int(1)),
                    Expression::Literal(Literal::Bool(true)),
                ],
            }),
            mutable: false,
        }]);

        let file = create_file_with_items(vec![Item::Function(test_func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_range_expression() {
        let mut analyzer = SemanticAnalyzer::new();

        let test_func = create_test_function(vec![Statement::Let {
            name: Ident::new("range"),
            ty: None,
            init: Some(Expression::Range {
                start: Some(Box::new(Expression::Literal(Literal::Int(0)))),
                end: Some(Box::new(Expression::Literal(Literal::Int(10)))),
                inclusive: false,
            }),
            mutable: false,
        }]);

        let file = create_file_with_items(vec![Item::Function(test_func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_prop_expression() {
        let mut analyzer = SemanticAnalyzer::new();

        let test_func = create_test_function(vec![
            Statement::Let {
                name: Ident::new("result"),
                ty: Some(Type::Fallible {
                    ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                }),
                init: None,
                mutable: false,
            },
            Statement::Let {
                name: Ident::new("val"),
                ty: None,
                init: Some(Expression::ErrorProp {
                    expr: Box::new(Expression::Ident(Ident::new("result"))),
                }),
                mutable: false,
            },
        ]);

        let file = create_file_with_items(vec![Item::Function(test_func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_prop_invalid() {
        let mut analyzer = SemanticAnalyzer::new();

        let test_func = create_test_function(vec![
            Statement::Let {
                name: Ident::new("x"),
                ty: Some(Type::Primitive(PrimitiveType::Int)),
                init: Some(Expression::Literal(Literal::Int(42))),
                mutable: false,
            },
            Statement::Expr(Expression::ErrorProp {
                expr: Box::new(Expression::Ident(Ident::new("x"))),
            }),
        ]);

        let file = create_file_with_items(vec![Item::Function(test_func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].kind, SemanticErrorKind::InvalidOperation);
    }

    #[test]
    fn test_method_call_expression() {
        let mut analyzer = SemanticAnalyzer::new();

        let test_func = create_test_function(vec![
            Statement::Let {
                name: Ident::new("s"),
                ty: Some(Type::Ident(Ident::new("String"))),
                init: None,
                mutable: false,
            },
            Statement::Expr(Expression::MethodCall {
                receiver: Box::new(Expression::Ident(Ident::new("s"))),
                method: Ident::new("len"),
                args: vec![],
            }),
        ]);

        let file = create_file_with_items(vec![Item::Function(test_func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_scoped_call() {
        let mut analyzer = SemanticAnalyzer::new();

        let test_func = create_test_function(vec![Statement::Expr(Expression::TypeScopedCall {
            ty: Type::Ident(Ident::new("String")),
            method: Ident::new("new"),
            args: vec![],
        })]);

        let file = create_file_with_items(vec![Item::Function(test_func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_explicit_generic_call() {
        let mut analyzer = SemanticAnalyzer::new();

        let test_func =
            create_test_function(vec![Statement::Expr(Expression::ExplicitGenericCall {
                ty: Type::Ident(Ident::new("Vec")),
                generics: vec![Type::Primitive(PrimitiveType::Int)],
                method: Ident::new("new"),
                args: vec![],
            })]);

        let file = create_file_with_items(vec![Item::Function(test_func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_literal_null() {
        let mut analyzer = SemanticAnalyzer::new();

        let test_func = create_test_function(vec![Statement::Let {
            name: Ident::new("ptr"),
            ty: None,
            init: Some(Expression::Literal(Literal::Null)),
            mutable: false,
        }]);

        let file = create_file_with_items(vec![Item::Function(test_func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }
}
