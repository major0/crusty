// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Additional tests to improve semantic analyzer coverage to 90%

#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::error::{SemanticError, SemanticErrorKind};
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
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
            }])),
            is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::While {
                label: None,
                condition: Expression::Literal(Literal::Bool(true)),
                body: Block::new(vec![Statement::Break { label: None, value: None }]),
            }])),
            is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::While {
                label: None,
                condition: Expression::Literal(Literal::Bool(true)),
                body: Block::new(vec![Statement::Continue(None)]),
            }])),
            is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::Switch {
                expr: Expression::Literal(Literal::Int(1)),
                cases: vec![SwitchCase {
                    values: vec![Expression::Literal(Literal::Int(1))],
                    body: Block::new(vec![]),
                }],
                default: None,
            }])),
            is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![
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
            ])),
            is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![
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
            ])),
            is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![
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
            ])),
            is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![
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
            ])),
            is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![
                Statement::Expr(Expression::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(Expression::Literal(Literal::Bool(true))),
                }),
                Statement::Expr(Expression::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(Expression::Literal(Literal::Int(42))),
                }),
            ])),
            is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::Expr(Expression::Cast {
                expr: Box::new(Expression::Literal(Literal::Int(42))),
                ty: Type::Primitive(PrimitiveType::F64),
            })])),
            is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::Expr(Expression::Sizeof {
                ty: Type::Primitive(PrimitiveType::Int),
            })])),
            is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::Expr(Expression::Ternary {
                condition: Box::new(Expression::Literal(Literal::Bool(true))),
                then_expr: Box::new(Expression::Literal(Literal::Int(1))),
                else_expr: Box::new(Expression::Literal(Literal::Int(2))),
            })])),
            is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::Expr(Expression::Range {
                start: Some(Box::new(Expression::Literal(Literal::Int(0)))),
                end: Some(Box::new(Expression::Literal(Literal::Int(10)))),
                inclusive: false,
            })])),
            is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![
                Statement::Expr(Expression::Literal(Literal::Null)),
                Statement::Expr(Expression::Literal(Literal::String("hello".to_string()))),
                Statement::Expr(Expression::Literal(Literal::Char('a'))),
                Statement::Expr(Expression::Literal(Literal::Float(2.5))),
            ])),
            is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
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
                generic_params: vec![],
                where_clause: None,
                visibility: Visibility::Public,
                name: Ident::new("new"),
                params: vec![],
                return_type: Some(Type::Ident(Ident::new("Point"))),
                body: Some(Block::new(vec![])),
                is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
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

    // ------------------------------------------------------------------
    // SEM-05: StructInit field-name / field-type / completeness checking
    // ------------------------------------------------------------------

    // Build a `struct Point { int x; int y; }` plus a `void main()` whose body
    // is `let p = Point { <fields> };`, so the struct is registered and the
    // initializer is analysed.
    fn point_program(init_fields: Vec<(Ident, Expression)>) -> File {
        let point = Item::Struct(Struct {
            generic_params: vec![],
            where_clause: None,
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
        });

        let main = Item::Function(Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("main"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::Let {
                name: Ident::new("p"),
                ty: None,
                init: Some(Expression::StructInit {
                    ty: Type::Ident(Ident::new("Point")),
                    fields: init_fields,
                    base: None,
                }),
                mutable: false,
            }])),
            is_variadic: false,
            doc_comments: vec![],
            attributes: vec![],
        });

        create_file_with_items(vec![point, main])
    }

    #[test]
    fn test_struct_init_all_fields_valid() {
        let mut analyzer = SemanticAnalyzer::new();
        let file = point_program(vec![
            (Ident::new("x"), Expression::Literal(Literal::Int(1))),
            (Ident::new("y"), Expression::Literal(Literal::Int(2))),
        ]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok(), "fully-initialised struct must pass: {:?}", result);
    }

    #[test]
    fn test_struct_init_unknown_field() {
        let mut analyzer = SemanticAnalyzer::new();
        let file = point_program(vec![
            (Ident::new("x"), Expression::Literal(Literal::Int(1))),
            (Ident::new("y"), Expression::Literal(Literal::Int(2))),
            (Ident::new("z"), Expression::Literal(Literal::Int(3))),
        ]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err(), "unknown field must be rejected");
        assert!(result
            .unwrap_err()
            .iter()
            .any(|e| e.kind == SemanticErrorKind::InvalidOperation
                && e.message.contains("unknown field 'z'")));
    }

    #[test]
    fn test_struct_init_missing_field() {
        let mut analyzer = SemanticAnalyzer::new();
        let file = point_program(vec![(
            Ident::new("x"),
            Expression::Literal(Literal::Int(1)),
        )]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err(), "missing field must be rejected");
        assert!(result
            .unwrap_err()
            .iter()
            .any(|e| e.kind == SemanticErrorKind::InvalidOperation
                && e.message.contains("missing field 'y'")));
    }

    #[test]
    fn test_struct_init_field_type_mismatch() {
        let mut analyzer = SemanticAnalyzer::new();
        let file = point_program(vec![
            (Ident::new("x"), Expression::Literal(Literal::Int(1))),
            (
                Ident::new("y"),
                Expression::Literal(Literal::String("nope".to_string())),
            ),
        ]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err(), "field type mismatch must be rejected");
        assert!(result
            .unwrap_err()
            .iter()
            .any(|e| e.kind == SemanticErrorKind::TypeMismatch
                && e.message.contains("struct field 'y'")));
    }

    // ------------------------------------------------------------------
    // SEM-06: field access on an unknown (Auto) receiver does not error
    // ------------------------------------------------------------------
    #[test]
    fn test_field_access_on_unknown_receiver_ok() {
        let mut analyzer = SemanticAnalyzer::new();
        // `foo().field` — a method/‑free call result types to Auto, so the
        // field access must pass through rather than erroring.
        let expr = Expression::FieldAccess {
            expr: Box::new(Expression::MethodCall {
                receiver: Box::new(Expression::Ident(Ident::new("obj"))),
                method: Ident::new("make"),
                args: vec![],
            }),
            field: Ident::new("port"),
        };
        analyzer.analyze_expression_test(&expr);
        assert!(
            analyzer
                .errors()
                .iter()
                .all(|e| e.kind != SemanticErrorKind::InvalidOperation),
            "field access on an unknown receiver must not error: {:?}",
            analyzer.errors()
        );
    }

    // ------------------------------------------------------------------
    // SEM-07: usize/isize indexing, and Vec<T> indexing
    // ------------------------------------------------------------------
    #[test]
    fn test_index_with_usize_and_vec() {
        let mut analyzer = SemanticAnalyzer::new();
        // Index an array by a usize-typed value: `arr[n as usize]`.
        let arr = Expression::Ident(Ident::new("arr"));
        // Fake the array's type by indexing a literal array; element i32.
        let array_expr = Expression::ArrayLit {
            elements: vec![
                Expression::Literal(Literal::Int(1)),
                Expression::Literal(Literal::Int(2)),
            ],
        };
        let idx = Expression::Cast {
            expr: Box::new(Expression::Literal(Literal::Int(0))),
            ty: Type::Primitive(PrimitiveType::Usize),
        };
        let e = Expression::Index {
            expr: Box::new(array_expr),
            index: Box::new(idx),
        };
        analyzer.analyze_expression_test(&e);
        assert!(
            analyzer.errors().is_empty(),
            "usize index on an array must be accepted: {:?}",
            analyzer.errors()
        );
        let _ = arr;

        // Index a Vec<i32> by an int: element type comes out as i32, no error.
        let mut analyzer = SemanticAnalyzer::new();
        let vec_val = Expression::Cast {
            // A cast to Vec<i32> is a convenient way to get that static type
            // onto an expression for the index check.
            expr: Box::new(Expression::ArrayLit { elements: vec![] }),
            ty: Type::Generic {
                base: Box::new(Type::Ident(Ident::new("Vec"))),
                args: vec![Type::Primitive(PrimitiveType::I32)],
            },
        };
        let e = Expression::Index {
            expr: Box::new(vec_val),
            index: Box::new(Expression::Literal(Literal::Int(0))),
        };
        let ty = analyzer.analyze_expression_test(&e);
        assert!(
            !analyzer
                .errors()
                .iter()
                .any(|er| er.message.contains("cannot index")),
            "Vec<T> indexing must not error: {:?}",
            analyzer.errors()
        );
        assert_eq!(ty, Type::Primitive(PrimitiveType::I32));
    }

    // ------------------------------------------------------------------
    // SEM-08: casting an enum to a primitive integer is allowed
    // ------------------------------------------------------------------
    #[test]
    fn test_enum_to_int_cast_ok() {
        let mut analyzer = SemanticAnalyzer::new();
        let enum_item = Item::Enum(Enum {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("Color"),
            variants: vec![
                EnumVariant { name: Ident::new("Red"), value: Some(0) },
                EnumVariant { name: Ident::new("Green"), value: Some(1) },
            ],
            doc_comments: vec![],
            attributes: vec![],
        });
        // void main() { let n = (Color as i32); }  — Color is a known enum.
        let main = Item::Function(Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("main"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![Statement::Let {
                name: Ident::new("n"),
                ty: None,
                init: Some(Expression::Cast {
                    expr: Box::new(Expression::Ident(Ident::new("Color"))),
                    ty: Type::Primitive(PrimitiveType::I32),
                }),
                mutable: false,
            }])),
            is_variadic: false,
            doc_comments: vec![],
            attributes: vec![],
        });
        let file = create_file_with_items(vec![enum_item, main]);
        let result = analyzer.analyze(&file);
        assert!(
            result.is_ok()
                || !result
                    .as_ref()
                    .unwrap_err()
                    .iter()
                    .any(|e| e.message.contains("invalid cast")),
            "enum -> int cast must not be an invalid-cast error: {:?}",
            result
        );
    }

    // ------------------------------------------------------------------
    // SEM-00 (ADR-0010): method-signature typing for MethodCall / TypeScopedCall
    // ------------------------------------------------------------------

    /// A `Point` struct with an instance method `scaled(self, int) -> Point`
    /// and an associated method `make() -> Point`.
    fn point_with_methods() -> Item {
        Item::Struct(Struct {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("Point"),
            fields: vec![Field {
                visibility: Visibility::Public,
                name: Ident::new("x"),
                ty: Type::Primitive(PrimitiveType::Int),
                doc_comments: vec![],
                attributes: vec![],
            }],
            methods: vec![
                Function {
                    generic_params: vec![],
                    where_clause: None,
                    visibility: Visibility::Public,
                    name: Ident::new("scaled"),
                    params: vec![
                        Param {
                            name: Ident::new("self"),
                            ty: Type::Ident(Ident::new("Self")),
                        },
                        Param {
                            name: Ident::new("factor"),
                            ty: Type::Primitive(PrimitiveType::Int),
                        },
                    ],
                    return_type: Some(Type::Ident(Ident::new("Point"))),
                    body: Some(Block::new(vec![])),
                    is_variadic: false,
                    doc_comments: vec![],
                    attributes: vec![],
                },
                Function {
                    generic_params: vec![],
                    where_clause: None,
                    visibility: Visibility::Public,
                    name: Ident::new("make"),
                    params: vec![],
                    return_type: Some(Type::Ident(Ident::new("Point"))),
                    body: Some(Block::new(vec![])),
                    is_variadic: false,
                    doc_comments: vec![],
                    attributes: vec![],
                },
                Function {
                    generic_params: vec![],
                    where_clause: None,
                    visibility: Visibility::Public,
                    name: Ident::new("origin_x"),
                    params: vec![],
                    return_type: Some(Type::Primitive(PrimitiveType::Int)),
                    body: Some(Block::new(vec![])),
                    is_variadic: false,
                    doc_comments: vec![],
                    attributes: vec![],
                },
            ],
            doc_comments: vec![],
            attributes: vec![],
        })
    }

    // A StructInit of Point types to Type::Ident("Point"), giving a known
    // struct receiver without any symbol-table setup.
    fn point_value() -> Expression {
        Expression::StructInit {
            ty: Type::Ident(Ident::new("Point")),
            fields: vec![(Ident::new("x"), Expression::Literal(Literal::Int(0)))],
            base: None,
        }
    }

    #[test]
    fn test_method_call_returns_declared_return_type() {
        let mut analyzer = SemanticAnalyzer::new();
        // Register the struct (and its method signatures) via a file analysis.
        let _ = analyzer.analyze(&create_file_with_items(vec![point_with_methods()]));

        // `Point{..}.scaled(2)` resolves the instance method's return type.
        let call = Expression::MethodCall {
            receiver: Box::new(point_value()),
            method: Ident::new("scaled"),
            args: vec![Expression::Literal(Literal::Int(2))],
        };
        let ty = analyzer.analyze_expression_test(&call);
        assert_eq!(ty, Type::Ident(Ident::new("Point")));
    }

    #[test]
    fn test_method_call_unknown_method_on_known_struct_errors() {
        let mut analyzer = SemanticAnalyzer::new();
        let _ = analyzer.analyze(&create_file_with_items(vec![point_with_methods()]));
        let before = analyzer.errors().len();

        let call = Expression::MethodCall {
            receiver: Box::new(point_value()),
            method: Ident::new("bogus"),
            args: vec![],
        };
        let ty = analyzer.analyze_expression_test(&call);
        assert_eq!(ty, Type::Auto);
        let new_errors: Vec<_> = analyzer.errors()[before..]
            .iter()
            .filter(|e| e.message.contains("no method 'bogus' on type 'Point'"))
            .collect();
        assert_eq!(new_errors.len(), 1, "expected exactly one missing-method error");
    }

    #[test]
    fn test_method_call_on_non_struct_receiver_is_auto() {
        let mut analyzer = SemanticAnalyzer::new();
        let _ = analyzer.analyze(&create_file_with_items(vec![point_with_methods()]));
        let before = analyzer.errors().len();

        // Receiver is an integer literal — not a struct. Pass through as Auto,
        // no error (rustc arbitrates methods on built-in types).
        let call = Expression::MethodCall {
            receiver: Box::new(Expression::Literal(Literal::Int(1))),
            method: Ident::new("scaled"),
            args: vec![],
        };
        let ty = analyzer.analyze_expression_test(&call);
        assert_eq!(ty, Type::Auto);
        assert_eq!(analyzer.errors().len(), before, "must not push an error");
    }

    #[test]
    fn test_type_scoped_call_returns_declared_return_type() {
        let mut analyzer = SemanticAnalyzer::new();
        let _ = analyzer.analyze(&create_file_with_items(vec![point_with_methods()]));

        // `Point::origin_x()` returns the associated method's DECLARED return
        // type (int) — proving SEM-02 returns the signature's return rather
        // than echoing the scope type (Point), which the old code did.
        let call = Expression::TypeScopedCall {
            ty: Type::Ident(Ident::new("Point")),
            method: Ident::new("origin_x"),
            args: vec![],
        };
        let ty = analyzer.analyze_expression_test(&call);
        assert_eq!(ty, Type::Primitive(PrimitiveType::Int));
    }

    #[test]
    fn test_type_scoped_call_unknown_type_is_auto() {
        let mut analyzer = SemanticAnalyzer::new();
        let before = analyzer.errors().len();
        // Unknown type: must return Auto (NOT echo the scope type) and not error.
        let call = Expression::TypeScopedCall {
            ty: Type::Ident(Ident::new("Nonexistent")),
            method: Ident::new("make"),
            args: vec![],
        };
        let ty = analyzer.analyze_expression_test(&call);
        assert_eq!(ty, Type::Auto);
        assert_eq!(analyzer.errors().len(), before, "must not push an error");
    }

    // ------------------------------------------------------------------
    // SEM-03 (ADR-0011): explicit-generic / turbofish return substitution
    // ------------------------------------------------------------------

    /// Parse Crusty source and analyze it, returning the analyzer so callers
    /// can inspect errors.
    fn analyze_source(src: &str) -> (SemanticAnalyzer, Result<(), Vec<SemanticError>>) {
        use crate::parser::Parser;
        let mut parser = Parser::new(src).expect("parser init");
        let file = parser.parse_file().expect("parse");
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&file);
        (analyzer, result)
    }

    /// A struct `Box` with a static method `wrap<U>(U) -> U` (returns its own
    /// generic parameter) plus `first(&self) -> int` (concrete return).
    fn box_with_generic_method() -> Item {
        Item::Struct(Struct {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("Boxy"),
            fields: vec![Field {
                visibility: Visibility::Public,
                name: Ident::new("v"),
                ty: Type::Primitive(PrimitiveType::Int),
                doc_comments: vec![],
                attributes: vec![],
            }],
            methods: vec![
                Function {
                    generic_params: vec![GenericParam::Type {
                        name: Ident::new("U"),
                        bounds: vec![],
                    }],
                    where_clause: None,
                    visibility: Visibility::Public,
                    name: Ident::new("wrap"),
                    params: vec![Param {
                        name: Ident::new("x"),
                        ty: Type::Ident(Ident::new("U")),
                    }],
                    return_type: Some(Type::Ident(Ident::new("U"))),
                    body: Some(Block::new(vec![])),
                    is_variadic: false,
                    doc_comments: vec![],
                    attributes: vec![],
                },
                Function {
                    generic_params: vec![],
                    where_clause: None,
                    visibility: Visibility::Public,
                    name: Ident::new("make"),
                    params: vec![],
                    return_type: Some(Type::Primitive(PrimitiveType::Int)),
                    body: Some(Block::new(vec![])),
                    is_variadic: false,
                    doc_comments: vec![],
                    attributes: vec![],
                },
            ],
            doc_comments: vec![],
            attributes: vec![],
        })
    }

    #[test]
    fn test_explicit_generic_call_substitutes_type_param_return() {
        let mut analyzer = SemanticAnalyzer::new();
        let _ = analyzer.analyze(&create_file_with_items(vec![box_with_generic_method()]));

        // `@Boxy(i32).wrap(0)` — wrap returns its own param `U`, so the result
        // is the supplied type argument i32.
        let call = Expression::ExplicitGenericCall {
            ty: Type::Ident(Ident::new("Boxy")),
            generics: vec![Type::Primitive(PrimitiveType::I32)],
            method: Ident::new("wrap"),
            args: vec![Expression::Literal(Literal::Int(0))],
        };
        let ty = analyzer.analyze_expression_test(&call);
        assert_eq!(ty, Type::Primitive(PrimitiveType::I32));
    }

    #[test]
    fn test_turbofish_call_substitutes_type_param_return() {
        let mut analyzer = SemanticAnalyzer::new();
        let _ = analyzer.analyze(&create_file_with_items(vec![box_with_generic_method()]));

        // `Boxy::wrap::<bool>(true)` — same substitution through TurbofishCall.
        let call = Expression::TurbofishCall {
            ty: Type::Ident(Ident::new("Boxy")),
            method: Ident::new("wrap"),
            generics: vec![Type::Primitive(PrimitiveType::Bool)],
            args: vec![Expression::Literal(Literal::Bool(true))],
        };
        let ty = analyzer.analyze_expression_test(&call);
        assert_eq!(ty, Type::Primitive(PrimitiveType::Bool));
    }

    #[test]
    fn test_explicit_generic_call_concrete_return_is_auto() {
        let mut analyzer = SemanticAnalyzer::new();
        let _ = analyzer.analyze(&create_file_with_items(vec![box_with_generic_method()]));

        // `make` returns a concrete `int`, not a generic param, so an
        // explicit-generic call on it is undecidable here -> Auto.
        let call = Expression::ExplicitGenericCall {
            ty: Type::Ident(Ident::new("Boxy")),
            generics: vec![Type::Primitive(PrimitiveType::I32)],
            method: Ident::new("make"),
            args: vec![],
        };
        let ty = analyzer.analyze_expression_test(&call);
        assert_eq!(ty, Type::Auto);
    }

    // ------------------------------------------------------------------
    // SEM-04 (ADR-0011): generic-param argument compatibility, generic
    // return construction, and array-literal <-> Vec<T>.
    // ------------------------------------------------------------------

    #[test]
    fn test_generic_function_accepts_concrete_args() {
        // A generic free function called with concrete arguments must type-
        // check: its `T` parameters erase to Auto, so `i32` is accepted.
        let (_, result) = analyze_source(
            "T echo<T>(T x) { return x; } void main() { let n = echo(7); }",
        );
        assert!(result.is_ok(), "generic fn call with concrete args: {:?}", result);
    }

    #[test]
    fn test_generic_struct_monomorphic_init_returns() {
        // A static constructor returning `Stack<T>` from a `Stack { .. }`
        // initializer must type-check (SEM-04 generic-return construction).
        let (_, result) = analyze_source(
            "struct Stack<T> { Vec<T> items; \
             static Stack<T> new() { return Stack { items: @Vec.new() }; } }",
        );
        assert!(result.is_ok(), "generic struct ctor: {:?}", result);
    }

    #[test]
    fn test_array_literal_initialises_declared_vec() {
        use crate::semantic::TypeEnvironment;
        // SEM-04 part 1: a declared `Vec<i32>` accepts an `[i32; n]` literal.
        let env = TypeEnvironment::new();
        let vec_i32 = Type::Generic {
            base: Box::new(Type::Ident(Ident::new("Vec"))),
            args: vec![Type::Primitive(PrimitiveType::I32)],
        };
        let arr_i32 = Type::Array {
            ty: Box::new(Type::Primitive(PrimitiveType::I32)),
            size: Some(3),
        };
        assert!(env.is_compatible(&vec_i32, &arr_i32));
        // Element-type mismatch is still rejected.
        let arr_bool = Type::Array {
            ty: Box::new(Type::Primitive(PrimitiveType::Bool)),
            size: Some(1),
        };
        assert!(!env.is_compatible(&vec_i32, &arr_bool));
    }

    // ------------------------------------------------------------------
    // PS-06 Phase 3 (ADR-0012): match binding scope and arm typing.
    // ------------------------------------------------------------------

    #[test]
    fn test_match_binding_in_scope_in_arm_body() {
        // A `Pattern::Binding` is visible in its arm body and guard.
        let (_, result) = analyze_source(
            "void main() { let n = 7; let d = match n { 0 => 0, x if x > 1 => x + x, y => y }; }",
        );
        assert!(result.is_ok(), "binding should be in scope: {:?}", result);
    }

    #[test]
    fn test_match_arm_body_undefined_variable_errors() {
        // A non-raw arm body referencing an unbound name still errors — the
        // binding scope does not suppress real diagnostics.
        let (_, result) = analyze_source(
            "void main() { let n = 7; let d = match n { _ => bogus }; }",
        );
        assert!(result.is_err(), "undefined var in arm body should error");
    }

    #[test]
    fn test_match_binding_does_not_leak_out_of_arm() {
        // A name bound in one arm is not visible after the match.
        let (_, result) = analyze_source(
            "void main() { let n = 7; let d = match n { z => z }; let e = z; }",
        );
        assert!(result.is_err(), "arm binding must not leak to outer scope");
    }

    #[test]
    fn test_match_raw_pattern_body_not_strictly_analysed() {
        // A `Raw` pattern may bind names Crusty cannot parse (`Some(v)`), so
        // its arm body is deferred to rustc rather than false-flagged.
        let (_, result) = analyze_source(
            "void main() { let n = 7; let d = match n { Some(v) => v, _ => 0 }; }",
        );
        assert!(result.is_ok(), "raw-pattern arm body must not error: {:?}", result);
    }
}
