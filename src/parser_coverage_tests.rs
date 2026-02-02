// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Comprehensive parser coverage tests to reach 90% coverage target

#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::parser::Parser;

    // ========== Switch Statement Tests ==========
    // Note: Switch statements may not be fully implemented in the parser

    // ========== C-Style For Loop Tests ==========

    #[test]
    fn test_parse_c_style_for_loop() {
        let source = "void func() { for (int i = 0; i < 10; i++) { } }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 1);
                match &func.body.statements[0] {
                    Statement::For {
                        init,
                        condition,
                        increment: _,
                        ..
                    } => {
                        // init is a Box<Statement>, not Option
                        // condition is Expression, not Option
                        // Check that they exist (not None for optional fields)
                        let _ = init;
                        let _ = condition;
                    }
                    _ => panic!("Expected for statement"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_for_in_loop() {
        let source = "void func() { for (item in collection) { } }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::ForIn { .. } => {}
                _ => panic!("Expected for-in statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    // ========== Labeled Loop Tests ==========

    #[test]
    fn test_parse_labeled_while_loop() {
        let source = "void func() { .outer: while (true) { } }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::While { label, .. } => {
                    assert!(label.is_some());
                    assert_eq!(label.as_ref().unwrap().name, "outer");
                }
                _ => panic!("Expected while statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    // ========== Expression Tests ==========
    // Note: Some advanced expression features may not be fully implemented

    #[test]
    fn test_parse_parenthesized_expression() {
        let source = "int func() { return (1 + 2); }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Return(Some(_)) => {}
                _ => panic!("Expected return statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_cast_expression() {
        let source = "int func() { return (int)x; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Return(Some(expr)) => match expr {
                    Expression::Cast { .. } => {}
                    _ => panic!("Expected cast expression"),
                },
                _ => panic!("Expected return statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    // ========== Type Parsing Tests ==========
    // Note: Some advanced type features may not be fully implemented

    #[test]
    fn test_parse_pointer_type() {
        let source = "void func(int* ptr) { }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.params.len(), 1);
                match &func.params[0].ty {
                    Type::Pointer { .. } => {}
                    _ => panic!("Expected pointer type"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    // ========== Attribute Tests ==========

    #[test]
    fn test_parse_attribute_with_string_arg() {
        let source = r#"
            #[doc("This is a test")]
            int func() { return 0; }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.attributes.len(), 1);
                assert_eq!(func.attributes[0].name.name, "doc");
                assert_eq!(func.attributes[0].args.len(), 1);
                match &func.attributes[0].args[0] {
                    AttributeArg::Literal(Literal::String(_)) => {}
                    _ => panic!("Expected string literal argument"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_attribute_with_int_arg() {
        let source = r#"
            #[align(8)]
            struct Data { int x; }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Struct(s) => {
                assert_eq!(s.attributes.len(), 1);
                assert_eq!(s.attributes[0].args.len(), 1);
                match &s.attributes[0].args[0] {
                    AttributeArg::Literal(Literal::Int(_)) => {}
                    _ => panic!("Expected int literal argument"),
                }
            }
            _ => panic!("Expected struct"),
        }
    }

    #[test]
    fn test_parse_attribute_with_name_value() {
        let source = r#"
            #[cfg(target = "linux")]
            int func() { return 0; }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.attributes.len(), 1);
                assert_eq!(func.attributes[0].args.len(), 1);
                match &func.attributes[0].args[0] {
                    AttributeArg::NameValue { name, .. } => {
                        assert_eq!(name.name, "target");
                    }
                    _ => panic!("Expected name-value argument"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_attribute_on_struct_field() {
        let source = r#"
            struct Data {
                #[serde(skip)]
                int internal;
            }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Struct(s) => {
                assert_eq!(s.fields.len(), 1);
                assert_eq!(s.fields[0].attributes.len(), 1);
            }
            _ => panic!("Expected struct"),
        }
    }

    // ========== Macro Tests ==========

    #[test]
    fn test_parse_define_with_brackets() {
        let source = "#define __MAX__[a, b] a > b ? a : b";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::MacroDefinition(m) => {
                assert_eq!(m.name.name, "__MAX__");
                assert_eq!(m.params.len(), 2);
                assert!(matches!(m.delimiter, MacroDelimiter::Brackets));
            }
            _ => panic!("Expected macro definition"),
        }
    }

    #[test]
    fn test_parse_define_with_braces() {
        let source = "#define __MAX__{a, b} a > b ? a : b";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::MacroDefinition(m) => {
                assert_eq!(m.name.name, "__MAX__");
                assert_eq!(m.params.len(), 2);
                assert!(matches!(m.delimiter, MacroDelimiter::Braces));
            }
            _ => panic!("Expected macro definition"),
        }
    }

    #[test]
    fn test_parse_define_constant() {
        let source = "#define __PI__ 3.14159";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::MacroDefinition(m) => {
                assert_eq!(m.name.name, "__PI__");
                assert_eq!(m.params.len(), 0);
                assert!(matches!(m.delimiter, MacroDelimiter::None));
            }
            _ => panic!("Expected macro definition"),
        }
    }

    #[test]
    fn test_parse_macro_call_with_brackets() {
        let source = "void func() { __MAX__[1, 2]; }";
        let mut parser = Parser::new(source).unwrap();
        // First parse the macro definition to register it
        let define_source = "#define __MAX__[a, b] a > b ? a : b";
        let mut define_parser = Parser::new(define_source).unwrap();
        let _ = define_parser.parse_file();

        // Now parse the function with macro call
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Expr(Expression::MacroCall { .. }) => {}
                _ => panic!("Expected macro call"),
            },
            _ => panic!("Expected function"),
        }
    }

    // ========== Struct Initializer Tests ==========
    // Note: Struct initializers may require specific syntax

    // ========== Unary Operator Tests ==========

    #[test]
    fn test_parse_pre_increment() {
        let source = "void func() { ++x; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Expr(Expression::Unary { op, .. }) => {
                    assert!(matches!(op, UnaryOp::PreInc));
                }
                _ => panic!("Expected unary expression"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_pre_decrement() {
        let source = "void func() { --x; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Expr(Expression::Unary { op, .. }) => {
                    assert!(matches!(op, UnaryOp::PreDec));
                }
                _ => panic!("Expected unary expression"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_post_increment() {
        let source = "void func() { x++; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Expr(Expression::Unary { op, .. }) => {
                    assert!(matches!(op, UnaryOp::PostInc));
                }
                _ => panic!("Expected unary expression"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_post_decrement() {
        let source = "void func() { x--; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Expr(Expression::Unary { op, .. }) => {
                    assert!(matches!(op, UnaryOp::PostDec));
                }
                _ => panic!("Expected unary expression"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_dereference() {
        let source = "void func() { int x = *ptr; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::Unary { op, .. } => {
                        assert!(matches!(op, UnaryOp::Deref));
                    }
                    _ => panic!("Expected unary expression"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    // ========== Binary Operator Tests ==========

    #[test]
    fn test_parse_bitwise_shift_left() {
        let source = "void func() { int x = a << 2; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::Binary { op, .. } => {
                        assert!(matches!(op, BinaryOp::Shl));
                    }
                    _ => panic!("Expected binary expression"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_bitwise_shift_right() {
        let source = "void func() { int x = a >> 2; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::Binary { op, .. } => {
                        assert!(matches!(op, BinaryOp::Shr));
                    }
                    _ => panic!("Expected binary expression"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_bitwise_xor() {
        let source = "void func() { int x = a ^ b; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::Binary { op, .. } => {
                        assert!(matches!(op, BinaryOp::BitXor));
                    }
                    _ => panic!("Expected binary expression"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    // ========== Range Expression Tests ==========
    // Note: Range expressions may not be fully implemented as standalone expressions

    // ========== Method Call Tests ==========

    #[test]
    fn test_parse_field_access_chain() {
        let source = "void func() { int x = obj.field1.field2; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let { init: Some(_), .. } => {}
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    // ========== Static Declaration Tests ==========

    #[test]
    fn test_parse_static_typedef() {
        let source = "static typedef int MyInt;";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Typedef(t) => {
                assert_eq!(t.visibility, Visibility::Private);
            }
            _ => panic!("Expected typedef"),
        }
    }

    // ========== Nested Function Tests ==========

    #[test]
    fn test_parse_nested_function_with_params() {
        let source = r#"
            void outer() {
                int inner(int x) {
                    return x * 2;
                }
            }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 1);
                match &func.body.statements[0] {
                    Statement::NestedFunction { name, params, .. } => {
                        assert_eq!(name.name, "inner");
                        assert_eq!(params.len(), 1);
                    }
                    _ => panic!("Expected nested function"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_nested_function_void_return() {
        let source = r#"
            void outer() {
                void inner() {
                    return;
                }
            }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::NestedFunction { return_type, .. } => {
                    assert!(return_type.is_none());
                }
                _ => panic!("Expected nested function"),
            },
            _ => panic!("Expected function"),
        }
    }

    // ========== Assignment Operator Tests ==========
    // Note: Compound assignment operators like +=, -=, etc. are parsed as
    // separate tokens and handled during semantic analysis, not in the parser

    // ========== Literal Tests ==========

    #[test]
    fn test_parse_int_literal() {
        let source = "void func() { int x = 42; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::Literal(Literal::Int(_)) => {}
                    _ => panic!("Expected int literal"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_float_literal() {
        let source = "void func() { float f = 3.14; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::Literal(Literal::Float(_)) => {}
                    _ => panic!("Expected float literal"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_string_literal() {
        let source = r#"void func() { let s = "hello"; }"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::Literal(Literal::String(_)) => {}
                    _ => panic!("Expected string literal"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_bool_true_literal() {
        let source = "void func() { bool b = true; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::Literal(Literal::Bool(true)) => {}
                    _ => panic!("Expected bool true literal"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_bool_false_literal() {
        let source = "void func() { bool b = false; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::Literal(Literal::Bool(false)) => {}
                    _ => panic!("Expected bool false literal"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    // ========== Edge Case Tests ==========

    #[test]
    fn test_parse_empty_array_literal() {
        let source = "void func() { let arr = []; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::ArrayLit { elements } => {
                        assert_eq!(elements.len(), 0);
                    }
                    _ => panic!("Expected array literal"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }
}
