// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Additional tests to improve semantic analyzer coverage to 90%

#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::error::SemanticErrorKind;
    use crate::semantic::SemanticAnalyzer;

    // Helper to create a simple file with items
    fn create_file_with_items(items: Vec<Item>) -> File {
        File {
            items,
            doc_comments: vec![],
        }
    }

    // Test const declaration with type mismatch
    #[test]
    fn test_const_declaration_type_mismatch() {
        let mut analyzer = SemanticAnalyzer::new();

        let const_item = Item::Const(Const {
            visibility: Visibility::Public,
            name: Ident::new("MY_CONST"),
            ty: Type::Primitive(PrimitiveType::Int),
            value: Expression::Literal(Literal::Bool(true)), // Type mismatch
            doc_comments: vec![],
        });

        let file = create_file_with_items(vec![const_item]);
        let result = analyzer.analyze(&file);

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind, SemanticErrorKind::TypeMismatch);
    }

    // Test const declaration valid
    #[test]
    fn test_const_declaration_valid() {
        let mut analyzer = SemanticAnalyzer::new();

        let const_item = Item::Const(Const {
            visibility: Visibility::Public,
            name: Ident::new("MY_CONST"),
            ty: Type::Primitive(PrimitiveType::Int),
            value: Expression::Literal(Literal::Int(42)),
            doc_comments: vec![],
        });

        let file = create_file_with_items(vec![const_item]);
        let result = analyzer.analyze(&file);

        assert!(result.is_ok());
    }

    // Test static declaration with type mismatch
    #[test]
    fn test_static_declaration_type_mismatch() {
        let mut analyzer = SemanticAnalyzer::new();

        let static_item = Item::Static(Static {
            visibility: Visibility::Public,
            name: Ident::new("MY_STATIC"),
            ty: Type::Primitive(PrimitiveType::Int),
            value: Expression::Literal(Literal::Bool(false)), // Type mismatch
            mutable: false,
            doc_comments: vec![],
        });

        let file = create_file_with_items(vec![static_item]);
        let result = analyzer.analyze(&file);

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind, SemanticErrorKind::TypeMismatch);
    }

    // Test static declaration valid
    #[test]
    fn test_static_declaration_valid() {
        let mut analyzer = SemanticAnalyzer::new();

        let static_item = Item::Static(Static {
            visibility: Visibility::Public,
            name: Ident::new("MY_STATIC"),
            ty: Type::Primitive(PrimitiveType::Int),
            value: Expression::Literal(Literal::Int(100)),
            mutable: true,
            doc_comments: vec![],
        });

        let file = create_file_with_items(vec![static_item]);
        let result = analyzer.analyze(&file);

        assert!(result.is_ok());
    }

    // Test function with various statement types
    #[test]
    fn test_function_with_for_loop() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Item::Function(Function {
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
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
                condition: Expression::Binary {
                    op: BinaryOp::Lt,
                    left: Box::new(Expression::Ident(Ident::new("i"))),
                    right: Box::new(Expression::Literal(Literal::Int(10))),
                },
                increment: Expression::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expression::Ident(Ident::new("i"))),
                    right: Box::new(Expression::Literal(Literal::Int(1))),
                },
                body: Block::new(vec![]),
            }]),
            doc_comments: vec![],
            attributes: vec![],
        });

        let file = create_file_with_items(vec![func]);
        let result = analyzer.analyze(&file);

        assert!(result.is_ok());
    }

    // Test function with break statement
    #[test]
    fn test_function_with_break() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Item::Function(Function {
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::While {
                label: None,
                condition: Expression::Literal(Literal::Bool(true)),
                body: Block::new(vec![Statement::Break(None)]),
            }]),
            doc_comments: vec![],
            attributes: vec![],
        });

        let file = create_file_with_items(vec![func]);
        let result = analyzer.analyze(&file);

        assert!(result.is_ok());
    }

    // Test function with continue statement
    #[test]
    fn test_function_with_continue() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Item::Function(Function {
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::While {
                label: None,
                condition: Expression::Literal(Literal::Bool(true)),
                body: Block::new(vec![Statement::Continue(None)]),
            }]),
            doc_comments: vec![],
            attributes: vec![],
        });

        let file = create_file_with_items(vec![func]);
        let result = analyzer.analyze(&file);

        assert!(result.is_ok());
    }

    // Test function with switch statement
    #[test]
    fn test_function_with_switch() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Item::Function(Function {
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Switch {
                expr: Expression::Literal(Literal::Int(1)),
                cases: vec![SwitchCase {
                    values: vec![Expression::Literal(Literal::Int(1))],
                    body: Block::new(vec![]),
                }],
                default: None,
            }]),
            doc_comments: vec![],
            attributes: vec![],
        });

        let file = create_file_with_items(vec![func]);
        let result = analyzer.analyze(&file);

        assert!(result.is_ok());
    }

    // Test function with various binary operations
    #[test]
    fn test_function_with_arithmetic_ops() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Item::Function(Function {
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![
                Statement::Expr(Expression::Binary {
                    op: BinaryOp::Sub,
                    left: Box::new(Expression::Literal(Literal::Int(10))),
                    right: Box::new(Expression::Literal(Literal::Int(5))),
                }),
                Statement::Expr(Expression::Binary {
                    op: BinaryOp::Mul,
                    left: Box::new(Expression::Literal(Literal::Int(3))),
                    right: Box::new(Expression::Literal(Literal::Int(4))),
                }),
                Statement::Expr(Expression::Binary {
                    op: BinaryOp::Div,
                    left: Box::new(Expression::Literal(Literal::Int(20))),
                    right: Box::new(Expression::Literal(Literal::Int(4))),
                }),
                Statement::Expr(Expression::Binary {
                    op: BinaryOp::Mod,
                    left: Box::new(Expression::Literal(Literal::Int(10))),
                    right: Box::new(Expression::Literal(Literal::Int(3))),
                }),
            ]),
            doc_comments: vec![],
            attributes: vec![],
        });

        let file = create_file_with_items(vec![func]);
        let result = analyzer.analyze(&file);

        assert!(result.is_ok());
    }

    // Test function with comparison operations
    #[test]
    fn test_function_with_comparison_ops() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Item::Function(Function {
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![
                Statement::Expr(Expression::Binary {
                    op: BinaryOp::Gt,
                    left: Box::new(Expression::Literal(Literal::Int(10))),
                    right: Box::new(Expression::Literal(Literal::Int(5))),
                }),
                Statement::Expr(Expression::Binary {
                    op: BinaryOp::Le,
                    left: Box::new(Expression::Literal(Literal::Int(5))),
                    right: Box::new(Expression::Literal(Literal::Int(10))),
                }),
                Statement::Expr(Expression::Binary {
                    op: BinaryOp::Ge,
                    left: Box::new(Expression::Literal(Literal::Int(10))),
                    right: Box::new(Expression::Literal(Literal::Int(5))),
                }),
                Statement::Expr(Expression::Binary {
                    op: BinaryOp::Eq,
                    left: Box::new(Expression::Literal(Literal::Int(5))),
                    right: Box::new(Expression::Literal(Literal::Int(5))),
                }),
                Statement::Expr(Expression::Binary {
                    op: BinaryOp::Ne,
                    left: Box::new(Expression::Literal(Literal::Int(5))),
                    right: Box::new(Expression::Literal(Literal::Int(10))),
                }),
            ]),
            doc_comments: vec![],
            attributes: vec![],
        });

        let file = create_file_with_items(vec![func]);
        let result = analyzer.analyze(&file);

        assert!(result.is_ok());
    }

    // Test function with logical operations
    #[test]
    fn test_function_with_logical_ops() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Item::Function(Function {
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![
                Statement::Expr(Expression::Binary {
                    op: BinaryOp::And,
                    left: Box::new(Expression::Literal(Literal::Bool(true))),
                    right: Box::new(Expression::Literal(Literal::Bool(false))),
                }),
                Statement::Expr(Expression::Binary {
                    op: BinaryOp::Or,
                    left: Box::new(Expression::Literal(Literal::Bool(true))),
                    right: Box::new(Expression::Literal(Literal::Bool(false))),
                }),
            ]),
            doc_comments: vec![],
            attributes: vec![],
        });

        let file = create_file_with_items(vec![func]);
        let result = analyzer.analyze(&file);

        assert!(result.is_ok());
    }

    // Test function with bitwise operations
    #[test]
    fn test_function_with_bitwise_ops() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Item::Function(Function {
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![
                Statement::Expr(Expression::Binary {
                    op: BinaryOp::BitAnd,
                    left: Box::new(Expression::Literal(Literal::Int(15))),
                    right: Box::new(Expression::Literal(Literal::Int(7))),
                }),
                Statement::Expr(Expression::Binary {
                    op: BinaryOp::BitOr,
                    left: Box::new(Expression::Literal(Literal::Int(8))),
                    right: Box::new(Expression::Literal(Literal::Int(4))),
                }),
                Statement::Expr(Expression::Binary {
                    op: BinaryOp::BitXor,
                    left: Box::new(Expression::Literal(Literal::Int(15))),
                    right: Box::new(Expression::Literal(Literal::Int(7))),
                }),
                Statement::Expr(Expression::Binary {
                    op: BinaryOp::Shl,
                    left: Box::new(Expression::Literal(Literal::Int(1))),
                    right: Box::new(Expression::Literal(Literal::Int(3))),
                }),
                Statement::Expr(Expression::Binary {
                    op: BinaryOp::Shr,
                    left: Box::new(Expression::Literal(Literal::Int(8))),
                    right: Box::new(Expression::Literal(Literal::Int(2))),
                }),
            ]),
            doc_comments: vec![],
            attributes: vec![],
        });

        let file = create_file_with_items(vec![func]);
        let result = analyzer.analyze(&file);

        assert!(result.is_ok());
    }

    // Test function with unary operations
    #[test]
    fn test_function_with_unary_ops() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Item::Function(Function {
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![
                Statement::Expr(Expression::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(Expression::Literal(Literal::Bool(true))),
                }),
                Statement::Expr(Expression::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(Expression::Literal(Literal::Int(42))),
                }),
            ]),
            doc_comments: vec![],
            attributes: vec![],
        });

        let file = create_file_with_items(vec![func]);
        let result = analyzer.analyze(&file);

        assert!(result.is_ok());
    }

    // Test function with cast expression
    #[test]
    fn test_function_with_cast() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Item::Function(Function {
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Expr(Expression::Cast {
                expr: Box::new(Expression::Literal(Literal::Int(42))),
                ty: Type::Primitive(PrimitiveType::F64),
            })]),
            doc_comments: vec![],
            attributes: vec![],
        });

        let file = create_file_with_items(vec![func]);
        let result = analyzer.analyze(&file);

        assert!(result.is_ok());
    }

    // Test function with sizeof expression
    #[test]
    fn test_function_with_sizeof() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Item::Function(Function {
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Expr(Expression::Sizeof {
                ty: Type::Primitive(PrimitiveType::Int),
            })]),
            doc_comments: vec![],
            attributes: vec![],
        });

        let file = create_file_with_items(vec![func]);
        let result = analyzer.analyze(&file);

        assert!(result.is_ok());
    }

    // Test function with ternary expression
    #[test]
    fn test_function_with_ternary() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Item::Function(Function {
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Expr(Expression::Ternary {
                condition: Box::new(Expression::Literal(Literal::Bool(true))),
                then_expr: Box::new(Expression::Literal(Literal::Int(1))),
                else_expr: Box::new(Expression::Literal(Literal::Int(2))),
            })]),
            doc_comments: vec![],
            attributes: vec![],
        });

        let file = create_file_with_items(vec![func]);
        let result = analyzer.analyze(&file);

        assert!(result.is_ok());
    }

    // Test function with range expression
    #[test]
    fn test_function_with_range() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Item::Function(Function {
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![Statement::Expr(Expression::Range {
                start: Some(Box::new(Expression::Literal(Literal::Int(0)))),
                end: Some(Box::new(Expression::Literal(Literal::Int(10)))),
                inclusive: false,
            })]),
            doc_comments: vec![],
            attributes: vec![],
        });

        let file = create_file_with_items(vec![func]);
        let result = analyzer.analyze(&file);

        assert!(result.is_ok());
    }

    // Test function with different literal types
    #[test]
    fn test_function_with_various_literals() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Item::Function(Function {
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![
                Statement::Expr(Expression::Literal(Literal::Null)),
                Statement::Expr(Expression::Literal(Literal::String("hello".to_string()))),
                Statement::Expr(Expression::Literal(Literal::Char('a'))),
                Statement::Expr(Expression::Literal(Literal::Float(2.5))),
            ]),
            doc_comments: vec![],
            attributes: vec![],
        });

        let file = create_file_with_items(vec![func]);
        let result = analyzer.analyze(&file);

        assert!(result.is_ok());
    }

    // Test struct with methods
    #[test]
    fn test_struct_with_methods() {
        let mut analyzer = SemanticAnalyzer::new();

        let struct_item = Item::Struct(Struct {
            visibility: Visibility::Public,
            name: Ident::new("Point"),
            fields: vec![Field {
                visibility: Visibility::Public,
                name: Ident::new("x"),
                ty: Type::Primitive(PrimitiveType::Int),
                doc_comments: vec![],
                attributes: vec![],
            }],
            methods: vec![Function {
                visibility: Visibility::Public,
                name: Ident::new("new"),
                params: vec![],
                return_type: Some(Type::Ident(Ident::new("Point"))),
                body: Block::new(vec![]),
                doc_comments: vec![],
                attributes: vec![],
            }],
            doc_comments: vec![],
            attributes: vec![],
        });

        let file = create_file_with_items(vec![struct_item]);
        let result = analyzer.analyze(&file);

        assert!(result.is_ok());
    }

    // Test enum with variants
    #[test]
    fn test_enum_with_explicit_values() {
        let mut analyzer = SemanticAnalyzer::new();

        let enum_item = Item::Enum(Enum {
            visibility: Visibility::Public,
            name: Ident::new("Status"),
            variants: vec![
                EnumVariant {
                    name: Ident::new("Success"),
                    value: Some(0),
                },
                EnumVariant {
                    name: Ident::new("Error"),
                    value: Some(1),
                },
            ],
            doc_comments: vec![],
            attributes: vec![],
        });

        let file = create_file_with_items(vec![enum_item]);
        let result = analyzer.analyze(&file);

        assert!(result.is_ok());
    }

    // Test typedef with complex types
    #[test]
    fn test_typedef_pointer_type() {
        let mut analyzer = SemanticAnalyzer::new();

        let typedef_item = Item::Typedef(Typedef {
            visibility: Visibility::Public,
            name: Ident::new("IntPtr"),
            target: Type::Pointer {
                ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                mutable: true,
            },
            doc_comments: vec![],
        });

        let file = create_file_with_items(vec![typedef_item]);
        let result = analyzer.analyze(&file);

        assert!(result.is_ok());
    }

    // Test typedef with reference type
    #[test]
    fn test_typedef_reference_type() {
        let mut analyzer = SemanticAnalyzer::new();

        let typedef_item = Item::Typedef(Typedef {
            visibility: Visibility::Public,
            name: Ident::new("IntRef"),
            target: Type::Reference {
                ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                mutable: false,
            },
            doc_comments: vec![],
        });

        let file = create_file_with_items(vec![typedef_item]);
        let result = analyzer.analyze(&file);

        assert!(result.is_ok());
    }

    // Test typedef with array type
    #[test]
    fn test_typedef_array_type() {
        let mut analyzer = SemanticAnalyzer::new();

        let typedef_item = Item::Typedef(Typedef {
            visibility: Visibility::Public,
            name: Ident::new("IntArray"),
            target: Type::Array {
                ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                size: Some(10),
            },
            doc_comments: vec![],
        });

        let file = create_file_with_items(vec![typedef_item]);
        let result = analyzer.analyze(&file);

        assert!(result.is_ok());
    }

    // Test typedef with slice type
    #[test]
    fn test_typedef_slice_type() {
        let mut analyzer = SemanticAnalyzer::new();

        let typedef_item = Item::Typedef(Typedef {
            visibility: Visibility::Public,
            name: Ident::new("IntSlice"),
            target: Type::Slice {
                ty: Box::new(Type::Primitive(PrimitiveType::Int)),
            },
            doc_comments: vec![],
        });

        let file = create_file_with_items(vec![typedef_item]);
        let result = analyzer.analyze(&file);

        assert!(result.is_ok());
    }

    // Test typedef with tuple type
    #[test]
    fn test_typedef_tuple_type() {
        let mut analyzer = SemanticAnalyzer::new();

        let typedef_item = Item::Typedef(Typedef {
            visibility: Visibility::Public,
            name: Ident::new("IntPair"),
            target: Type::Tuple {
                types: vec![
                    Type::Primitive(PrimitiveType::Int),
                    Type::Primitive(PrimitiveType::Int),
                ],
            },
            doc_comments: vec![],
        });

        let file = create_file_with_items(vec![typedef_item]);
        let result = analyzer.analyze(&file);

        assert!(result.is_ok());
    }

    // Test typedef with generic type
    #[test]
    fn test_typedef_generic_type() {
        let mut analyzer = SemanticAnalyzer::new();

        let typedef_item = Item::Typedef(Typedef {
            visibility: Visibility::Public,
            name: Ident::new("IntVec"),
            target: Type::Generic {
                base: Box::new(Type::Ident(Ident::new("Vec"))),
                args: vec![Type::Primitive(PrimitiveType::Int)],
            },
            doc_comments: vec![],
        });

        let file = create_file_with_items(vec![typedef_item]);
        let result = analyzer.analyze(&file);

        assert!(result.is_ok());
    }

    // Test typedef with function type
    #[test]
    fn test_typedef_function_type() {
        let mut analyzer = SemanticAnalyzer::new();

        let typedef_item = Item::Typedef(Typedef {
            visibility: Visibility::Public,
            name: Ident::new("BinaryOp"),
            target: Type::Function {
                params: vec![
                    Type::Primitive(PrimitiveType::Int),
                    Type::Primitive(PrimitiveType::Int),
                ],
                return_type: Box::new(Type::Primitive(PrimitiveType::Int)),
            },
            doc_comments: vec![],
        });

        let file = create_file_with_items(vec![typedef_item]);
        let result = analyzer.analyze(&file);

        assert!(result.is_ok());
    }

    // Test typedef with fallible type
    #[test]
    fn test_typedef_fallible_type() {
        let mut analyzer = SemanticAnalyzer::new();

        let typedef_item = Item::Typedef(Typedef {
            visibility: Visibility::Public,
            name: Ident::new("IntResult"),
            target: Type::Fallible {
                ty: Box::new(Type::Primitive(PrimitiveType::Int)),
            },
            doc_comments: vec![],
        });

        let file = create_file_with_items(vec![typedef_item]);
        let result = analyzer.analyze(&file);

        assert!(result.is_ok());
    }
}
