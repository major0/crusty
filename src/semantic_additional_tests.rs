// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Additional tests for semantic analyzer coverage

#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::semantic::SemanticAnalyzer;

    fn create_file_with_items(items: Vec<Item>) -> File {
        File {
            items,
            doc_comments: vec![],
        }
    }

    #[test]
    fn test_empty_array_literal() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Let {
                name: Ident::new("arr"),
                ty: None,
                init: Some(Expression::ArrayLit { elements: vec![] }),
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
    fn test_struct_init_expression() {
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

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Let {
                name: Ident::new("p"),
                ty: None,
                init: Some(Expression::StructInit {
                    ty: Type::Ident(Ident::new("Point")),
                    fields: vec![
                        (Ident::new("x"), Expression::Literal(Literal::Int(10))),
                        (Ident::new("y"), Expression::Literal(Literal::Int(20))),
                    ],
                }),
                mutable: false,
            }]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Struct(point_struct), Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_range_with_start_only() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Let {
                name: Ident::new("range"),
                ty: None,
                init: Some(Expression::Range {
                    start: Some(Box::new(Expression::Literal(Literal::Int(5)))),
                    end: None,
                    inclusive: false,
                }),
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
    fn test_range_with_end_only() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Let {
                name: Ident::new("range"),
                ty: None,
                init: Some(Expression::Range {
                    start: None,
                    end: Some(Box::new(Expression::Literal(Literal::Int(10)))),
                    inclusive: false,
                }),
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
    fn test_range_inclusive() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Let {
                name: Ident::new("range"),
                ty: None,
                init: Some(Expression::Range {
                    start: Some(Box::new(Expression::Literal(Literal::Int(0)))),
                    end: Some(Box::new(Expression::Literal(Literal::Int(10)))),
                    inclusive: true,
                }),
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
    fn test_macro_call_expression() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Expr(Expression::MacroCall {
                name: Ident::new("println"),
                args: vec![],
            })]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rust_block_expression() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Let {
                name: Ident::new("x"),
                ty: None,
                init: Some(Expression::RustBlock { tokens: vec![] }),
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
    fn test_binary_op_shift_left() {
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
                    op: BinaryOp::Shl,
                    left: Box::new(Expression::Literal(Literal::Int(1))),
                    right: Box::new(Expression::Literal(Literal::Int(3))),
                }),
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
    fn test_binary_op_shift_right() {
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
                    op: BinaryOp::Shr,
                    left: Box::new(Expression::Literal(Literal::Int(8))),
                    right: Box::new(Expression::Literal(Literal::Int(2))),
                }),
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
    fn test_binary_op_bitwise_xor() {
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
                    op: BinaryOp::BitXor,
                    left: Box::new(Expression::Literal(Literal::Int(5))),
                    right: Box::new(Expression::Literal(Literal::Int(3))),
                }),
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
    fn test_binary_op_bitwise_or() {
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
                    op: BinaryOp::BitOr,
                    left: Box::new(Expression::Literal(Literal::Int(5))),
                    right: Box::new(Expression::Literal(Literal::Int(3))),
                }),
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
    fn test_binary_op_modulo() {
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
                    op: BinaryOp::Mod,
                    left: Box::new(Expression::Literal(Literal::Int(10))),
                    right: Box::new(Expression::Literal(Literal::Int(3))),
                }),
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
    fn test_binary_op_comparison_operators() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![
                Statement::Let {
                    name: Ident::new("eq"),
                    ty: None,
                    init: Some(Expression::Binary {
                        op: BinaryOp::Eq,
                        left: Box::new(Expression::Literal(Literal::Int(1))),
                        right: Box::new(Expression::Literal(Literal::Int(1))),
                    }),
                    mutable: false,
                },
                Statement::Let {
                    name: Ident::new("ne"),
                    ty: None,
                    init: Some(Expression::Binary {
                        op: BinaryOp::Ne,
                        left: Box::new(Expression::Literal(Literal::Int(1))),
                        right: Box::new(Expression::Literal(Literal::Int(2))),
                    }),
                    mutable: false,
                },
                Statement::Let {
                    name: Ident::new("gt"),
                    ty: None,
                    init: Some(Expression::Binary {
                        op: BinaryOp::Gt,
                        left: Box::new(Expression::Literal(Literal::Int(2))),
                        right: Box::new(Expression::Literal(Literal::Int(1))),
                    }),
                    mutable: false,
                },
                Statement::Let {
                    name: Ident::new("le"),
                    ty: None,
                    init: Some(Expression::Binary {
                        op: BinaryOp::Le,
                        left: Box::new(Expression::Literal(Literal::Int(1))),
                        right: Box::new(Expression::Literal(Literal::Int(2))),
                    }),
                    mutable: false,
                },
                Statement::Let {
                    name: Ident::new("ge"),
                    ty: None,
                    init: Some(Expression::Binary {
                        op: BinaryOp::Ge,
                        left: Box::new(Expression::Literal(Literal::Int(2))),
                        right: Box::new(Expression::Literal(Literal::Int(1))),
                    }),
                    mutable: false,
                },
            ]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_binary_op_assignment_operators() {
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
                    init: Some(Expression::Literal(Literal::Int(10))),
                },
                Statement::Expr(Expression::Binary {
                    op: BinaryOp::SubAssign,
                    left: Box::new(Expression::Ident(Ident::new("x"))),
                    right: Box::new(Expression::Literal(Literal::Int(5))),
                }),
                Statement::Expr(Expression::Binary {
                    op: BinaryOp::MulAssign,
                    left: Box::new(Expression::Ident(Ident::new("x"))),
                    right: Box::new(Expression::Literal(Literal::Int(2))),
                }),
                Statement::Expr(Expression::Binary {
                    op: BinaryOp::DivAssign,
                    left: Box::new(Expression::Ident(Ident::new("x"))),
                    right: Box::new(Expression::Literal(Literal::Int(2))),
                }),
            ]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unary_op_postdec() {
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
                    init: Some(Expression::Literal(Literal::Int(10))),
                },
                Statement::Expr(Expression::Unary {
                    op: UnaryOp::PostDec,
                    expr: Box::new(Expression::Ident(Ident::new("x"))),
                }),
            ]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unary_op_postinc() {
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
                    init: Some(Expression::Literal(Literal::Int(0))),
                },
                Statement::Expr(Expression::Unary {
                    op: UnaryOp::PostInc,
                    expr: Box::new(Expression::Ident(Ident::new("x"))),
                }),
            ]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unary_op_predec() {
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
                    init: Some(Expression::Literal(Literal::Int(10))),
                },
                Statement::Expr(Expression::Unary {
                    op: UnaryOp::PreDec,
                    expr: Box::new(Expression::Ident(Ident::new("x"))),
                }),
            ]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_literal_types() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![
                Statement::Let {
                    name: Ident::new("int_val"),
                    ty: None,
                    init: Some(Expression::Literal(Literal::Int(42))),
                    mutable: false,
                },
                Statement::Let {
                    name: Ident::new("float_val"),
                    ty: None,
                    init: Some(Expression::Literal(Literal::Float(2.5))),
                    mutable: false,
                },
                Statement::Let {
                    name: Ident::new("string_val"),
                    ty: None,
                    init: Some(Expression::Literal(Literal::String("hello".to_string()))),
                    mutable: false,
                },
                Statement::Let {
                    name: Ident::new("char_val"),
                    ty: None,
                    init: Some(Expression::Literal(Literal::Char('a'))),
                    mutable: false,
                },
                Statement::Let {
                    name: Ident::new("bool_val"),
                    ty: None,
                    init: Some(Expression::Literal(Literal::Bool(true))),
                    mutable: false,
                },
            ]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_binary_op_assign() {
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
                    init: Some(Expression::Literal(Literal::Int(10))),
                },
                Statement::Expr(Expression::Binary {
                    op: BinaryOp::Assign,
                    left: Box::new(Expression::Ident(Ident::new("x"))),
                    right: Box::new(Expression::Literal(Literal::Int(20))),
                }),
            ]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }
}
