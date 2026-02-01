// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Unit tests for C-style variable declarations

#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::parser::Parser;

    // Task 2.1 Tests: let with explicit type

    #[test]
    fn test_let_with_int_type() {
        let source = "int main() { let int x = 42; return x; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 2);
                match &func.body.statements[0] {
                    Statement::Let {
                        name,
                        ty,
                        init,
                        mutable,
                    } => {
                        assert_eq!(name.name, "x");
                        assert!(matches!(ty, Some(Type::Primitive(PrimitiveType::Int))));
                        assert!(init.is_some());
                        assert!(!mutable);
                    }
                    _ => panic!("Expected Let statement"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_let_with_typedef_type() {
        let source = r#"
            typedef int MyInt;
            int main() { 
                let MyInt x = 32; 
                return x; 
            }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[1] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    name,
                    ty,
                    init,
                    mutable,
                } => {
                    assert_eq!(name.name, "x");
                    assert!(matches!(ty, Some(Type::Ident(_))));
                    assert!(init.is_some());
                    assert!(!mutable);
                }
                _ => panic!("Expected Let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_let_with_type_inference_still_works() {
        let source = "int main() { let x = 42; return x; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => {
                match &func.body.statements[0] {
                    Statement::Let {
                        name,
                        ty,
                        init,
                        mutable,
                    } => {
                        assert_eq!(name.name, "x");
                        assert!(ty.is_none()); // Type inference
                        assert!(init.is_some());
                        assert!(!mutable);
                    }
                    _ => panic!("Expected Let statement"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    // Task 2.2 Tests: var with explicit type

    #[test]
    fn test_var_with_int_type() {
        let source = "int main() { var int x = 42; x = 43; return x; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Var { name, ty, init } => {
                    assert_eq!(name.name, "x");
                    assert!(matches!(ty, Some(Type::Primitive(PrimitiveType::Int))));
                    assert!(init.is_some());
                }
                _ => panic!("Expected Var statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_var_with_typedef_type() {
        let source = r#"
            typedef int MyInt;
            int main() { 
                var MyInt x = 32; 
                x = 33;
                return x; 
            }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[1] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Var { name, ty, init } => {
                    assert_eq!(name.name, "x");
                    assert!(matches!(ty, Some(Type::Ident(_))));
                    assert!(init.is_some());
                }
                _ => panic!("Expected Var statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_var_with_type_inference_still_works() {
        let source = "int main() { var x = 42; x = 43; return x; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => {
                match &func.body.statements[0] {
                    Statement::Var { name, ty, init } => {
                        assert_eq!(name.name, "x");
                        assert!(ty.is_none()); // Type inference
                        assert!(init.is_some());
                    }
                    _ => panic!("Expected Var statement"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    // Task 2.3 Tests: const with explicit type

    #[test]
    fn test_const_with_int_type() {
        let source = "int main() { const int MAX = 100; return MAX; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Const { name, ty, value: _ } => {
                    assert_eq!(name.name, "MAX");
                    assert!(matches!(ty, Type::Primitive(PrimitiveType::Int)));
                }
                _ => panic!("Expected Const statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_const_with_typedef_type() {
        let source = r#"
            typedef int MyInt;
            int main() { 
                const MyInt MAX = 100; 
                return MAX; 
            }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[1] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Const { name, ty, value: _ } => {
                    assert_eq!(name.name, "MAX");
                    assert!(matches!(ty, Type::Ident(_)));
                }
                _ => panic!("Expected Const statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_const_with_type_inference_still_works() {
        let source = "int main() { const MAX = 100; return MAX; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => {
                match &func.body.statements[0] {
                    Statement::Const {
                        name,
                        ty: _,
                        value: _,
                    } => {
                        assert_eq!(name.name, "MAX");
                        // Type is inferred (will be Int by default)
                    }
                    _ => panic!("Expected Const statement"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    // Task 2.4 Tests: implicit let (C-style)

    #[test]
    fn test_implicit_let_with_int() {
        let source = "int main() { int x = 42; return x; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => {
                match &func.body.statements[0] {
                    Statement::Let {
                        name,
                        ty,
                        init,
                        mutable,
                    } => {
                        assert_eq!(name.name, "x");
                        assert!(matches!(ty, Some(Type::Primitive(PrimitiveType::Int))));
                        assert!(init.is_some());
                        assert!(!mutable); // Implicit let is immutable
                    }
                    _ => panic!("Expected Let statement, got {:?}", func.body.statements[0]),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_implicit_let_with_typedef() {
        let source = r#"
            typedef int MyInt;
            int main() { 
                MyInt x = 32; 
                return x; 
            }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[1] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    name,
                    ty,
                    init,
                    mutable,
                } => {
                    assert_eq!(name.name, "x");
                    assert!(matches!(ty, Some(Type::Ident(_))));
                    assert!(init.is_some());
                    assert!(!mutable);
                }
                _ => panic!("Expected Let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_implicit_let_with_pointer_type() {
        let source = "int main() { int* ptr = 0; return 0; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => match &func.body.statements[0] {
                Statement::Let {
                    name,
                    ty,
                    init,
                    mutable,
                } => {
                    assert_eq!(name.name, "ptr");
                    assert!(matches!(ty, Some(Type::Pointer { .. })));
                    assert!(init.is_some());
                    assert!(!mutable);
                }
                _ => panic!("Expected Let statement"),
            },
            _ => panic!("Expected function"),
        }
    }

    // Task 2.5 & 2.6 Tests: lookahead and routing

    #[test]
    fn test_cast_expression_not_declaration() {
        // (int)x should be parsed as a cast, not a declaration
        // Using proper Crusty cast syntax: (Type)expr
        let source = "int main() { int x = 5; int y = (int)x; return y; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 3);
                // First statement should be a declaration
                assert!(matches!(func.body.statements[0], Statement::Let { .. }));
                // Second statement should also be a declaration with cast in init
                match &func.body.statements[1] {
                    Statement::Let { name, init, .. } => {
                        assert_eq!(name.name, "y");
                        // The init should contain a cast expression
                        assert!(matches!(init, Some(Expression::Cast { .. })));
                    }
                    _ => panic!("Expected Let statement"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_assignment_not_declaration() {
        // x = 42 should be parsed as assignment, not declaration
        let source = "int main() { int x = 0; x = 42; return x; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 3);
                // First statement is declaration
                assert!(matches!(func.body.statements[0], Statement::Let { .. }));
                // Second statement is assignment (expression statement)
                assert!(matches!(func.body.statements[1], Statement::Expr(_)));
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_multiple_declarations_in_function() {
        let source = r#"
            int main() {
                int x = 10;
                let int y = 20;
                var int z = 30;
                const int MAX = 100;
                return x + y + z + MAX;
            }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 5);
                // All should parse successfully
                assert!(matches!(func.body.statements[0], Statement::Let { .. }));
                assert!(matches!(func.body.statements[1], Statement::Let { .. }));
                assert!(matches!(func.body.statements[2], Statement::Var { .. }));
                assert!(matches!(func.body.statements[3], Statement::Const { .. }));
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_nested_function_still_works() {
        let source = r#"
            int main() {
                int helper(int x) {
                    return x + 1;
                }
                int result = helper(5);
                return result;
            }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 3);
                // First should be nested function
                assert!(matches!(
                    func.body.statements[0],
                    Statement::NestedFunction { .. }
                ));
                // Second should be declaration
                assert!(matches!(func.body.statements[1], Statement::Let { .. }));
            }
            _ => panic!("Expected function"),
        }
    }
}
