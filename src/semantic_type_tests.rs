// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Tests for semantic analyzer type checking coverage

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
    fn test_cast_pointer_types() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![
                Statement::Let {
                    name: Ident::new("ptr1"),
                    ty: Some(Type::Pointer {
                        ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                        mutable: false,
                    }),
                    init: None,
                    mutable: false,
                },
                Statement::Let {
                    name: Ident::new("ptr2"),
                    ty: None,
                    init: Some(Expression::Cast {
                        expr: Box::new(Expression::Ident(Ident::new("ptr1"))),
                        ty: Type::Pointer {
                            ty: Box::new(Type::Primitive(PrimitiveType::Char)),
                            mutable: false,
                        },
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
    fn test_cast_primitive_to_pointer() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![
                Statement::Let {
                    name: Ident::new("addr"),
                    ty: Some(Type::Primitive(PrimitiveType::U64)),
                    init: None,
                    mutable: false,
                },
                Statement::Let {
                    name: Ident::new("ptr"),
                    ty: None,
                    init: Some(Expression::Cast {
                        expr: Box::new(Expression::Ident(Ident::new("addr"))),
                        ty: Type::Pointer {
                            ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                            mutable: false,
                        },
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
    fn test_cast_pointer_to_primitive() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![
                Statement::Let {
                    name: Ident::new("ptr"),
                    ty: Some(Type::Pointer {
                        ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                        mutable: false,
                    }),
                    init: None,
                    mutable: false,
                },
                Statement::Let {
                    name: Ident::new("addr"),
                    ty: None,
                    init: Some(Expression::Cast {
                        expr: Box::new(Expression::Ident(Ident::new("ptr"))),
                        ty: Type::Primitive(PrimitiveType::U64),
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
    fn test_cast_invalid() {
        let mut analyzer = SemanticAnalyzer::new();

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![
                Statement::Let {
                    name: Ident::new("s"),
                    ty: Some(Type::Ident(Ident::new("String"))),
                    init: None,
                    mutable: false,
                },
                Statement::Let {
                    name: Ident::new("x"),
                    ty: None,
                    init: Some(Expression::Cast {
                        expr: Box::new(Expression::Ident(Ident::new("s"))),
                        ty: Type::Primitive(PrimitiveType::Int),
                    }),
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
        assert_eq!(errors[0].kind, SemanticErrorKind::InvalidOperation);
    }

    #[test]
    fn test_field_access_on_reference() {
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

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![
                Statement::Let {
                    name: Ident::new("p_ref"),
                    ty: Some(Type::Reference {
                        ty: Box::new(Type::Ident(Ident::new("Point"))),
                        mutable: false,
                    }),
                    init: None,
                    mutable: false,
                },
                Statement::Let {
                    name: Ident::new("x_val"),
                    ty: None,
                    init: Some(Expression::FieldAccess {
                        expr: Box::new(Expression::Ident(Ident::new("p_ref"))),
                        field: Ident::new("x"),
                    }),
                    mutable: false,
                },
            ]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Struct(point_struct), Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_field_access_on_pointer() {
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

        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::new(vec![
                Statement::Let {
                    name: Ident::new("p_ptr"),
                    ty: Some(Type::Pointer {
                        ty: Box::new(Type::Ident(Ident::new("Point"))),
                        mutable: false,
                    }),
                    init: None,
                    mutable: false,
                },
                Statement::Let {
                    name: Ident::new("x_val"),
                    ty: None,
                    init: Some(Expression::FieldAccess {
                        expr: Box::new(Expression::Ident(Ident::new("p_ptr"))),
                        field: Ident::new("x"),
                    }),
                    mutable: false,
                },
            ]),
            doc_comments: vec![],
            attributes: vec![],
        };

        let file = create_file_with_items(vec![Item::Struct(point_struct), Item::Function(func)]);
        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_binary_op_logical_and() {
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
                    op: BinaryOp::And,
                    left: Box::new(Expression::Literal(Literal::Bool(true))),
                    right: Box::new(Expression::Literal(Literal::Bool(false))),
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
    fn test_binary_op_logical_or() {
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
                    op: BinaryOp::Or,
                    left: Box::new(Expression::Literal(Literal::Bool(true))),
                    right: Box::new(Expression::Literal(Literal::Bool(false))),
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
}
