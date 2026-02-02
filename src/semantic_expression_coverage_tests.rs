// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Additional tests to improve semantic analyzer expression coverage to 90%

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

    #[test]
    fn test_unary_not_expression() {
        let mut analyzer = SemanticAnalyzer::new();
        
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: Some(Type::Primitive(PrimitiveType::Bool)),
            body: Block::new(vec![
                Statement::Return(Some(Expression::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(Expression::Literal(Literal::Bool(true))),
                })),
            ]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unary_negate_expression() {
        let mut analyzer = SemanticAnalyzer::new();
        
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: Some(Type::Primitive(PrimitiveType::Int)),
            body: Block::new(vec![
                Statement::Return(Some(Expression::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(Expression::Literal(Literal::Int(42))),
                })),
            ]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unary_deref_expression() {
        let mut analyzer = SemanticAnalyzer::new();
        
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![
                Param {
                    name: Ident::new("ptr"),
                    ty: Type::Pointer {
                        ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                        mutable: false,
                    },
                },
            ],
            return_type: Some(Type::Primitive(PrimitiveType::Int)),
            body: Block::new(vec![
                Statement::Return(Some(Expression::Unary {
                    op: UnaryOp::Deref,
                    expr: Box::new(Expression::Ident(Ident::new("ptr"))),
                })),
            ]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unary_ref_expression() {
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
                    init: Some(Expression::Literal(Literal::Int(42))),
                    mutable: false,
                },
                Statement::Let {
                    name: Ident::new("r"),
                    ty: None,
                    init: Some(Expression::Unary {
                        op: UnaryOp::Ref,
                        expr: Box::new(Expression::Ident(Ident::new("x"))),
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
    fn test_pre_increment_expression() {
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
                    op: UnaryOp::PreInc,
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
    fn test_pre_decrement_expression() {
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
    fn test_post_increment_expression() {
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
    fn test_post_decrement_expression() {
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
    fn test_assignment_expression() {
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
                Statement::Expr(Expression::Binary {
                    op: BinaryOp::Assign,
                    left: Box::new(Expression::Ident(Ident::new("x"))),
                    right: Box::new(Expression::Literal(Literal::Int(42))),
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

    #[test]
    fn test_add_assign_expression() {
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
                Statement::Expr(Expression::Binary {
                    op: BinaryOp::AddAssign,
                    left: Box::new(Expression::Ident(Ident::new("x"))),
                    right: Box::new(Expression::Literal(Literal::Int(5))),
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
    fn test_sub_assign_expression() {
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
                    right: Box::new(Expression::Literal(Literal::Int(3))),
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
    fn test_mul_assign_expression() {
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
                    init: Some(Expression::Literal(Literal::Int(5))),
                },
                Statement::Expr(Expression::Binary {
                    op: BinaryOp::MulAssign,
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
    fn test_div_assign_expression() {
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
    fn test_return_type_mismatch() {
        let mut analyzer = SemanticAnalyzer::new();
        
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: Some(Type::Primitive(PrimitiveType::Int)),
            body: Block::new(vec![
                Statement::Return(Some(Expression::Literal(Literal::Bool(true)))),
            ]),
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
    fn test_return_void_with_value() {
        let mut analyzer = SemanticAnalyzer::new();
        
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: Some(Type::Primitive(PrimitiveType::Void)),
            body: Block::new(vec![
                Statement::Return(Some(Expression::Literal(Literal::Int(42)))),
            ]),
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
    fn test_return_value_from_void_function() {
        let mut analyzer = SemanticAnalyzer::new();
        
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![
                Statement::Return(Some(Expression::Literal(Literal::Int(42)))),
            ]),
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
    fn test_return_without_value_from_non_void() {
        let mut analyzer = SemanticAnalyzer::new();
        
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: Some(Type::Primitive(PrimitiveType::Int)),
            body: Block::new(vec![
                Statement::Return(None),
            ]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors[0].kind, SemanticErrorKind::TypeMismatch);
    }
}
