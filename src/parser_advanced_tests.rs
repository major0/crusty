// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Unit tests for advanced parsing features (task 14)

#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::parser::Parser;

    #[test]
    fn test_parse_struct_with_method() {
        let source = r#"
            struct Point {
                int x;
                int y;
                
                int get_x(&self) {
                    return self.x;
                }
            }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        assert_eq!(file.items.len(), 1);
        match &file.items[0] {
            Item::Struct(s) => {
                assert_eq!(s.name.name, "Point");
                assert_eq!(s.fields.len(), 2);
                assert_eq!(s.methods.len(), 1);
                assert_eq!(s.methods[0].name.name, "get_x");
            }
            _ => panic!("Expected struct"),
        }
    }

    #[test]
    fn test_parse_struct_with_static_method() {
        let source = r#"
            struct Point {
                static int new(int x, int y) {
                    return 0;
                }
            }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Struct(s) => {
                assert_eq!(s.methods.len(), 1);
                assert_eq!(s.methods[0].visibility, Visibility::Private);
            }
            _ => panic!("Expected struct"),
        }
    }

    #[test]
    fn test_parse_type_scoped_call() {
        let source = "int main() { return @Vec.new(); }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Return(Some(expr)) => {
                    assert!(matches!(expr, Expression::TypeScopedCall { .. }));
                }
                _ => panic!("Expected return statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_explicit_generic_parameters() {
        let source = "int main() { return @Vec(i32).new(); }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Return(Some(expr)) => match expr {
                    Expression::ExplicitGenericCall {
                        ty,
                        generics,
                        method,
                        ..
                    } => {
                        assert_eq!(method.name, "new");
                        assert_eq!(generics.len(), 1);
                    }
                    _ => panic!("Expected ExplicitGenericCall"),
                },
                _ => panic!("Expected return statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_nested_generic_parameters() {
        let source = "int main() { return @Option(Result[String, Error]).None; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => {
                match &func.body.statements[0] {
                    Statement::Return(Some(expr)) => {
                        match expr {
                            Expression::ExplicitGenericCall { generics, .. } => {
                                assert_eq!(generics.len(), 1);
                                // First generic should be Result[String, Error]
                                match &generics[0] {
                                    Type::Generic { args, .. } => {
                                        assert_eq!(args.len(), 2);
                                    }
                                    _ => panic!("Expected generic type"),
                                }
                            }
                            _ => panic!("Expected ExplicitGenericCall"),
                        }
                    }
                    _ => panic!("Expected return statement"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_attribute() {
        let source = r#"
            #[derive(Debug)]
            struct Point {
                int x;
            }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Struct(s) => {
                assert_eq!(s.attributes.len(), 1);
                assert_eq!(s.attributes[0].name.name, "derive");
                assert_eq!(s.attributes[0].args.len(), 1);
            }
            _ => panic!("Expected struct"),
        }
    }

    #[test]
    fn test_parse_multiple_attributes() {
        let source = r#"
            #[derive(Debug)]
            #[test]
            int test_func() {}
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.attributes.len(), 2);
                assert_eq!(func.attributes[0].name.name, "derive");
                assert_eq!(func.attributes[1].name.name, "test");
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_macro_call() {
        let source = "int main() { println!(\"hello\"); }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Expr(expr) => match expr {
                    Expression::MacroCall { name, .. } => {
                        assert_eq!(name.name, "println");
                    }
                    _ => panic!("Expected macro call"),
                },
                _ => panic!("Expected expression statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_array_literal() {
        let source = "int main() { let arr = [1, 2, 3]; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::ArrayLit { elements } => {
                        assert_eq!(elements.len(), 3);
                    }
                    _ => panic!("Expected array literal"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_tuple_literal() {
        let source = "int main() { let t = (1, 2, 3); }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::TupleLit { elements } => {
                        assert_eq!(elements.len(), 3);
                    }
                    _ => panic!("Expected tuple literal"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_tuple_indexing() {
        let source = "int main() { return t.0; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Return(Some(expr)) => match expr {
                    Expression::FieldAccess { field, .. } => {
                        assert_eq!(field.name, "0");
                    }
                    _ => panic!("Expected field access"),
                },
                _ => panic!("Expected return statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_range_expression() {
        let source = "int main() { let r = arr[0..5]; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::Index { index, .. } => match &**index {
                        Expression::Range {
                            start,
                            end,
                            inclusive,
                        } => {
                            assert!(start.is_some());
                            assert!(end.is_some());
                            assert!(!inclusive);
                        }
                        _ => panic!("Expected range expression"),
                    },
                    _ => panic!("Expected index expression"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_inclusive_range() {
        let source = "int main() { let r = arr[0..=5]; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    init: Some(expr), ..
                } => match expr {
                    Expression::Index { index, .. } => match &**index {
                        Expression::Range { inclusive, .. } => {
                            assert!(*inclusive);
                        }
                        _ => panic!("Expected range expression"),
                    },
                    _ => panic!("Expected index expression"),
                },
                _ => panic!("Expected let statement"),
            },
            _ => panic!("Expected function"),
        }
    }
}
