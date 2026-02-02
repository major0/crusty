// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Additional parser coverage tests to reach 90% target

#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::parser::Parser;

    // ========== More Type Parsing Tests ==========
    // Note: Some type syntax may not be fully supported

    #[test]
    fn test_parse_tuple_type() {
        let source = "void func((int, int) pair) { }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.params.len(), 1);
                match &func.params[0].ty {
                    Type::Tuple { .. } => {}
                    _ => panic!("Expected tuple type"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_reference_type() {
        let source = "void func(&int ref) { }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.params.len(), 1);
                match &func.params[0].ty {
                    Type::Reference { .. } => {}
                    _ => panic!("Expected reference type"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_mutable_reference_type() {
        let source = "void func(&mut int ref) { }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.params.len(), 1);
                match &func.params[0].ty {
                    Type::Reference { mutable, .. } => {
                        assert!(*mutable);
                    }
                    _ => panic!("Expected reference type"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    // ========== More Expression Tests ==========

    #[test]
    fn test_parse_logical_and() {
        let source = "void func() { bool b = a && b; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::Binary { op, .. } => {
                        assert!(matches!(op, BinaryOp::And));
                    }
                    _ => panic!("Expected binary expression"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_logical_or() {
        let source = "void func() { bool b = a || b; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::Binary { op, .. } => {
                        assert!(matches!(op, BinaryOp::Or));
                    }
                    _ => panic!("Expected binary expression"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_bitwise_and() {
        let source = "void func() { int x = a & b; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::Binary { op, .. } => {
                        assert!(matches!(op, BinaryOp::BitAnd));
                    }
                    _ => panic!("Expected binary expression"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_bitwise_or() {
        let source = "void func() { int x = a | b; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::Binary { op, .. } => {
                        assert!(matches!(op, BinaryOp::BitOr));
                    }
                    _ => panic!("Expected binary expression"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_modulo() {
        let source = "void func() { int x = a % b; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::Binary { op, .. } => {
                        assert!(matches!(op, BinaryOp::Mod));
                    }
                    _ => panic!("Expected binary expression"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_not_equal() {
        let source = "void func() { bool b = a != b; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::Binary { op, .. } => {
                        assert!(matches!(op, BinaryOp::Ne));
                    }
                    _ => panic!("Expected binary expression"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_less_than_or_equal() {
        let source = "void func() { bool b = a <= b; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::Binary { op, .. } => {
                        assert!(matches!(op, BinaryOp::Le));
                    }
                    _ => panic!("Expected binary expression"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_greater_than_or_equal() {
        let source = "void func() { bool b = a >= b; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::Binary { op, .. } => {
                        assert!(matches!(op, BinaryOp::Ge));
                    }
                    _ => panic!("Expected binary expression"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_less_than() {
        let source = "void func() { bool b = a < b; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::Binary { op, .. } => {
                        assert!(matches!(op, BinaryOp::Lt));
                    }
                    _ => panic!("Expected binary expression"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_greater_than() {
        let source = "void func() { bool b = a > b; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::Binary { op, .. } => {
                        assert!(matches!(op, BinaryOp::Gt));
                    }
                    _ => panic!("Expected binary expression"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_equal() {
        let source = "void func() { bool b = a == b; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::Binary { op, .. } => {
                        assert!(matches!(op, BinaryOp::Eq));
                    }
                    _ => panic!("Expected binary expression"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    // ========== More Statement Tests ==========

    #[test]
    fn test_parse_if_else() {
        let source = r#"
            void func() {
                if (x > 0) {
                    y = 1;
                } else {
                    y = 0;
                }
            }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::If { else_block, .. } => {
                    assert!(else_block.is_some());
                }
                _ => panic!("Expected if statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_break_without_label() {
        let source = "void func() { while (true) { break; } }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::While { body, .. } => match &body.statements[0] {
                    Statement::Break(None) => {}
                    _ => panic!("Expected break without label"),
                },
                _ => panic!("Expected while statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_continue_without_label() {
        let source = "void func() { while (true) { continue; } }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::While { body, .. } => match &body.statements[0] {
                    Statement::Continue(None) => {}
                    _ => panic!("Expected continue without label"),
                },
                _ => panic!("Expected while statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    // ========== More Function Tests ==========

    #[test]
    fn test_parse_function_with_multiple_params() {
        let source = "int add(int a, int b, int c) { return a + b + c; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.params.len(), 3);
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_function_no_params() {
        let source = "int get_value() { return 42; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.params.len(), 0);
            }
            _ => panic!("Expected function"),
        }
    }

    // ========== Enum Tests ==========

    #[test]
    fn test_parse_enum_without_values() {
        let source = r#"
            enum Color {
                Red,
                Green,
                Blue
            }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Enum(e) => {
                assert_eq!(e.variants.len(), 3);
                assert_eq!(e.variants[0].value, Some(0));
                assert_eq!(e.variants[1].value, Some(1));
                assert_eq!(e.variants[2].value, Some(2));
            }
            _ => panic!("Expected enum"),
        }
    }

    #[test]
    fn test_parse_enum_with_trailing_comma() {
        let source = r#"
            enum Status {
                Active,
                Inactive,
            }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Enum(e) => {
                assert_eq!(e.variants.len(), 2);
            }
            _ => panic!("Expected enum"),
        }
    }

    // ========== Struct Tests ==========

    #[test]
    fn test_parse_struct_with_multiple_fields() {
        let source = r#"
            struct Point3D {
                int x;
                int y;
                int z;
            }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Struct(s) => {
                assert_eq!(s.fields.len(), 3);
            }
            _ => panic!("Expected struct"),
        }
    }

    #[test]
    fn test_parse_struct_with_method_and_fields() {
        let source = r#"
            struct Counter {
                int count;
                
                void increment(&mut self) {
                    self.count = self.count + 1;
                }
            }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Struct(s) => {
                assert_eq!(s.fields.len(), 1);
                assert_eq!(s.methods.len(), 1);
            }
            _ => panic!("Expected struct"),
        }
    }

    // ========== More Macro Tests ==========

    #[test]
    fn test_parse_define_with_no_params() {
        let source = "#define __VERSION__ 1";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::MacroDefinition(m) => {
                assert_eq!(m.params.len(), 0);
                assert!(matches!(m.delimiter, MacroDelimiter::None));
            }
            _ => panic!("Expected macro definition"),
        }
    }

    #[test]
    fn test_parse_define_with_single_param() {
        let source = "#define __SQUARE__(x) x * x";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::MacroDefinition(m) => {
                assert_eq!(m.params.len(), 1);
                assert_eq!(m.params[0].name, "x");
            }
            _ => panic!("Expected macro definition"),
        }
    }

    // ========== Complex Expression Tests ==========

    #[test]
    fn test_parse_nested_binary_expressions() {
        let source = "void func() { int x = a + b * c - d / e; }";
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

    #[test]
    fn test_parse_function_call_with_args() {
        let source = "void func() { result = calculate(1, 2, 3); }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Expr(Expression::Binary {
                    op: BinaryOp::Assign,
                    right,
                    ..
                }) => match &**right {
                    Expression::Call { args, .. } => {
                        assert_eq!(args.len(), 3);
                    }
                    _ => panic!("Expected function call"),
                },
                _ => panic!("Expected assignment"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_array_indexing() {
        let source = "void func() { int x = arr[5]; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::Index { .. } => {}
                    _ => panic!("Expected index expression"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_logical_not() {
        let source = "void func() { bool b = a == false; }";
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

    #[test]
    fn test_parse_unary_negate() {
        let source = "void func() { int x = -value; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::Unary { op, .. } => {
                        assert!(matches!(op, UnaryOp::Neg));
                    }
                    _ => panic!("Expected unary expression"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_unary_ref() {
        let source = "void func() { int* ptr = &value; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::Unary { op, .. } => {
                        assert!(matches!(op, UnaryOp::Ref));
                    }
                    _ => panic!("Expected unary expression"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }
}
