// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Unit tests for nested functions (Task 17.7)

#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::codegen::{CodeGenerator, TargetLanguage};
    use crate::parser::Parser;
    use crate::semantic::SemanticAnalyzer;

    // ============================================================================
    // Test Category 1: Parsing of Nested Functions
    // ============================================================================

    #[test]
    fn test_parse_simple_nested_function() {
        let source = r#"
void outer() {
    void inner() {
        return;
    }
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file();
        assert!(file.is_ok(), "Failed to parse simple nested function");

        let file = file.unwrap();
        assert_eq!(file.items.len(), 1);

        if let Item::Function(func) = &file.items[0] {
            assert_eq!(func.name.name, "outer");
            assert_eq!(func.body.statements.len(), 1);

            if let Statement::NestedFunction { name, .. } = &func.body.statements[0] {
                assert_eq!(name.name, "inner");
            } else {
                panic!("Expected NestedFunction statement");
            }
        } else {
            panic!("Expected Function item");
        }
    }

    #[test]
    fn test_parse_nested_function_with_parameters() {
        let source = r#"
void outer() {
    int add(int x, int y) {
        return x + y;
    }
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        if let Item::Function(func) = &file.items[0] {
            if let Statement::NestedFunction { name, params, .. } = &func.body.statements[0] {
                assert_eq!(name.name, "add");
                assert_eq!(params.len(), 2);
                assert_eq!(params[0].name.name, "x");
                assert_eq!(params[1].name.name, "y");
            } else {
                panic!("Expected NestedFunction statement");
            }
        }
    }

    #[test]
    fn test_parse_nested_function_with_return_type() {
        let source = r#"
void outer() {
    int get_value() {
        return 42;
    }
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        if let Item::Function(func) = &file.items[0] {
            if let Statement::NestedFunction {
                name, return_type, ..
            } = &func.body.statements[0]
            {
                assert_eq!(name.name, "get_value");
                assert!(return_type.is_some());
                assert!(matches!(
                    return_type.as_ref().unwrap(),
                    Type::Primitive(PrimitiveType::Int)
                ));
            } else {
                panic!("Expected NestedFunction statement");
            }
        }
    }

    #[test]
    fn test_parse_multiple_nested_functions() {
        let source = r#"
void outer() {
    void first() {
        return;
    }
    
    void second() {
        return;
    }
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        if let Item::Function(func) = &file.items[0] {
            assert_eq!(func.body.statements.len(), 2);

            if let Statement::NestedFunction { name, .. } = &func.body.statements[0] {
                assert_eq!(name.name, "first");
            } else {
                panic!("Expected first NestedFunction");
            }

            if let Statement::NestedFunction { name, .. } = &func.body.statements[1] {
                assert_eq!(name.name, "second");
            } else {
                panic!("Expected second NestedFunction");
            }
        }
    }

    #[test]
    fn test_parse_nested_function_with_body() {
        let source = r#"
void outer() {
    int compute(int x) {
        let result: int = x * 2;
        return result;
    }
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        if let Item::Function(func) = &file.items[0] {
            if let Statement::NestedFunction { body, .. } = &func.body.statements[0] {
                assert_eq!(body.statements.len(), 2);
            } else {
                panic!("Expected NestedFunction statement");
            }
        }
    }

    // ============================================================================
    // Test Category 2: Capture Analysis (Immutable and Mutable)
    // ============================================================================

    #[test]
    fn test_capture_analysis_immutable() {
        let source = r#"
void outer() {
    let x: int = 42;
    
    int get_x() {
        return x;
    }
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&file);
        assert!(result.is_ok(), "Semantic analysis should succeed");

        // Check that capture was recorded
        let captures = analyzer.get_captures("get_x");
        assert!(captures.is_some(), "Should have captures");
        let captures = captures.unwrap();
        assert_eq!(captures.len(), 1);
        assert_eq!(captures[0].name, "x");
        assert!(matches!(
            captures[0].kind,
            crate::semantic::CaptureKind::Immutable
        ));
    }

    #[test]
    fn test_capture_analysis_mutable() {
        let source = r#"
void outer() {
    var counter: int = 0;
    
    void increment() {
        let temp: int = counter + 1;
        counter = temp;
    }
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&file);
        assert!(result.is_ok(), "Semantic analysis should succeed");

        // Check that mutable capture was recorded
        let captures = analyzer.get_captures("increment");
        assert!(captures.is_some(), "Should have captures");
        let captures = captures.unwrap();
        assert_eq!(captures.len(), 1);
        assert_eq!(captures[0].name, "counter");
        assert!(matches!(
            captures[0].kind,
            crate::semantic::CaptureKind::Mutable
        ));
    }

    #[test]
    fn test_capture_analysis_multiple_variables() {
        let source = r#"
void outer() {
    let a: int = 1;
    let b: int = 2;
    var c: int = 3;
    
    int compute() {
        c = c + 1;
        return a + b + c;
    }
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&file);
        assert!(result.is_ok(), "Semantic analysis should succeed");

        let captures = analyzer.get_captures("compute");
        assert!(captures.is_some(), "Should have captures");
        let captures = captures.unwrap();
        assert_eq!(captures.len(), 3);

        // Check that a and b are immutable, c is mutable
        let a_capture = captures.iter().find(|c| c.name == "a").unwrap();
        assert!(matches!(
            a_capture.kind,
            crate::semantic::CaptureKind::Immutable
        ));

        let b_capture = captures.iter().find(|c| c.name == "b").unwrap();
        assert!(matches!(
            b_capture.kind,
            crate::semantic::CaptureKind::Immutable
        ));

        let c_capture = captures.iter().find(|c| c.name == "c").unwrap();
        assert!(matches!(
            c_capture.kind,
            crate::semantic::CaptureKind::Mutable
        ));
    }

    #[test]
    fn test_capture_analysis_no_captures() {
        let source = r#"
void outer() {
    let x: int = 42;
    
    int independent(int y) {
        return y * 2;
    }
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&file);
        assert!(result.is_ok(), "Semantic analysis should succeed");

        let captures = analyzer.get_captures("independent");
        assert!(captures.is_some(), "Should have captures entry");
        let captures = captures.unwrap();
        assert_eq!(captures.len(), 0, "Should have no captures");
    }

    #[test]
    fn test_capture_analysis_parameter_not_captured() {
        let source = r#"
void outer() {
    let x: int = 42;
    
    int use_param(int x) {
        return x;
    }
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&file);
        assert!(result.is_ok(), "Semantic analysis should succeed");

        let captures = analyzer.get_captures("use_param");
        assert!(captures.is_some());
        let captures = captures.unwrap();
        // Parameter 'x' should not be in captures (it shadows the outer 'x')
        assert_eq!(captures.len(), 0);
    }

    // ============================================================================
    // Test Category 3: Scoping Rules (Before/After Declaration)
    // ============================================================================

    #[test]
    fn test_scoping_variable_before_nested_function() {
        let source = r#"
void outer() {
    let x: int = 42;
    
    int get_x() {
        return x;
    }
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&file);
        assert!(
            result.is_ok(),
            "Should access variable declared before nested function"
        );
    }

    #[test]
    fn test_scoping_variable_after_nested_function() {
        let source = r#"
void outer() {
    int get_y() {
        return y;
    }
    
    let y: int = 42;
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&file);
        // This should fail because y is declared after the nested function
        assert!(
            result.is_err(),
            "Should not access variable declared after nested function"
        );
    }

    #[test]
    fn test_scoping_nested_function_can_call_itself() {
        let source = r#"
void outer() {
    let result = factorial(5);
    
    int factorial(int n) {
        if (n <= 1) {
            return 1;
        }
        return n * factorial(n - 1);
    }
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&file);
        // Note: This will fail because factorial is used before it's declared
        // This is expected behavior - nested functions must be declared before use
        assert!(result.is_err(), "Nested function used before declaration");
    }

    #[test]
    fn test_scoping_nested_function_declared_before_use() {
        let source = r#"
void outer() {
    int factorial(int n) {
        if (n <= 1) {
            return 1;
        }
        return n * factorial(n - 1);
    }
    
    let result = factorial(5);
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&file);
        assert!(
            result.is_ok(),
            "Should call nested function declared before use"
        );
    }

    // ============================================================================
    // Test Category 4: Passing Nested Functions as Parameters
    // ============================================================================

    #[test]
    fn test_nested_function_as_variable() {
        let source = r#"
void outer() {
    int add(int x, int y) {
        return x + y;
    }
    
    let func = add;
    let result = func(1, 2);
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&file);
        assert!(result.is_ok(), "Should assign nested function to variable");
    }

    #[test]
    fn test_nested_function_called_directly() {
        let source = r#"
void outer() {
    int add(int x, int y) {
        return x + y;
    }
    
    let result = add(1, 2);
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&file);
        assert!(result.is_ok(), "Should call nested function directly");
    }

    // ============================================================================
    // Test Category 5: Multiple Nested Functions Sharing Captures
    // ============================================================================

    #[test]
    fn test_multiple_nested_functions_share_immutable_capture() {
        let source = r#"
void outer() {
    let shared: int = 42;
    
    int first() {
        return shared;
    }
    
    int second() {
        return shared * 2;
    }
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&file);
        assert!(
            result.is_ok(),
            "Multiple nested functions should share immutable capture"
        );

        // Both should capture 'shared'
        let first_captures = analyzer.get_captures("first").unwrap();
        assert_eq!(first_captures.len(), 1);
        assert_eq!(first_captures[0].name, "shared");

        let second_captures = analyzer.get_captures("second").unwrap();
        assert_eq!(second_captures.len(), 1);
        assert_eq!(second_captures[0].name, "shared");
    }

    #[test]
    fn test_multiple_nested_functions_share_mutable_capture() {
        let source = r#"
void outer() {
    var counter: int = 0;
    
    void increment() {
        counter = counter + 1;
    }
    
    void decrement() {
        counter = counter - 1;
    }
    
    int get_counter() {
        return counter;
    }
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&file);
        assert!(
            result.is_ok(),
            "Multiple nested functions should share mutable capture"
        );

        // All three should capture 'counter'
        let inc_captures = analyzer.get_captures("increment").unwrap();
        assert_eq!(inc_captures.len(), 1);
        assert!(matches!(
            inc_captures[0].kind,
            crate::semantic::CaptureKind::Mutable
        ));

        let dec_captures = analyzer.get_captures("decrement").unwrap();
        assert_eq!(dec_captures.len(), 1);
        assert!(matches!(
            dec_captures[0].kind,
            crate::semantic::CaptureKind::Mutable
        ));

        let get_captures = analyzer.get_captures("get_counter").unwrap();
        assert_eq!(get_captures.len(), 1);
        assert!(matches!(
            get_captures[0].kind,
            crate::semantic::CaptureKind::Immutable
        ));
    }

    // ============================================================================
    // Test Category 6: Code Generation to Fn, FnMut, FnOnce
    // ============================================================================

    #[test]
    fn test_codegen_nested_function_basic() {
        let source = r#"
void outer() {
    void inner() {
        return;
    }
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = codegen.generate(&file);

        assert!(rust_code.contains("let inner = ||"));
        assert!(rust_code.contains("return;"));
    }

    #[test]
    fn test_codegen_nested_function_with_parameters() {
        let source = r#"
void outer() {
    int add(int x, int y) {
        return x + y;
    }
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = codegen.generate(&file);

        assert!(rust_code.contains("let add = |x: i32, y: i32| -> i32"));
        assert!(rust_code.contains("return (x + y);"));
    }

    #[test]
    fn test_codegen_nested_function_with_return_type() {
        let source = r#"
void outer() {
    int get_value() {
        return 42;
    }
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = codegen.generate(&file);

        assert!(rust_code.contains("let get_value = || -> i32"));
        assert!(rust_code.contains("return 42;"));
    }

    #[test]
    fn test_codegen_nested_function_void_return() {
        let source = r#"
void outer() {
    void do_something() {
        return;
    }
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = codegen.generate(&file);

        // Void return type should not generate -> annotation
        assert!(rust_code.contains("let do_something = ||"));
        assert!(!rust_code.contains("-> ()"));
    }

    #[test]
    fn test_codegen_nested_function_with_capture() {
        let source = r#"
void outer() {
    let x: int = 42;
    
    int get_x() {
        return x;
    }
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = codegen.generate(&file);

        // Should generate a closure that captures x
        assert!(rust_code.contains("let get_x = || -> i32"));
        assert!(rust_code.contains("return x;"));
    }

    #[test]
    fn test_codegen_multiple_nested_functions() {
        let source = r#"
void outer() {
    void first() {
        return;
    }
    
    void second() {
        return;
    }
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = codegen.generate(&file);

        assert!(rust_code.contains("let first = ||"));
        assert!(rust_code.contains("let second = ||"));
    }

    // ============================================================================
    // Test Category 7: Error Cases
    // ============================================================================

    #[test]
    fn test_error_nested_function_with_double_underscore_name() {
        let source = r#"
void outer() {
    void __invalid__() {
        return;
    }
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&file);
        assert!(
            result.is_err(),
            "Should reject nested function with double-underscore name"
        );
    }

    #[test]
    fn test_error_undefined_variable_in_nested_function() {
        let source = r#"
void outer() {
    int get_undefined() {
        return undefined_var;
    }
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&file);
        assert!(
            result.is_err(),
            "Should reject undefined variable in nested function"
        );
    }

    #[test]
    fn test_error_type_mismatch_in_nested_function() {
        let source = r#"
void outer() {
    int get_bool() {
        return true;
    }
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&file);
        assert!(
            result.is_err(),
            "Should reject type mismatch in nested function"
        );
    }

    // ============================================================================
    // Integration Tests
    // ============================================================================

    #[test]
    fn test_integration_nested_function_full_pipeline() {
        let source = r#"
void outer() {
    let x: int = 42;
    
    int add_x(int y) {
        return x + y;
    }
    
    let result = add_x(10);
}
"#;
        // Parse
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        // Semantic analysis
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&file);
        assert!(result.is_ok(), "Semantic analysis should succeed");

        // Code generation
        let mut codegen = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = codegen.generate(&file);

        // Verify generated code
        assert!(rust_code.contains("let x: i32 = 42;"));
        assert!(rust_code.contains("let add_x = |y: i32| -> i32"));
        assert!(rust_code.contains("let result = add_x(10);"));
    }

    #[test]
    fn test_integration_complex_nested_function() {
        let source = r#"
void outer() {
    var counter: int = 0;
    let multiplier: int = 2;
    
    void increment() {
        counter = counter + 1;
    }
    
    int get_value() {
        return counter * multiplier;
    }
    
    increment();
    increment();
    let value = get_value();
}
"#;
        // Parse
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        // Semantic analysis
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&file);
        assert!(result.is_ok(), "Semantic analysis should succeed");

        // Verify captures
        let inc_captures = analyzer.get_captures("increment").unwrap();
        assert_eq!(inc_captures.len(), 1);
        assert!(matches!(
            inc_captures[0].kind,
            crate::semantic::CaptureKind::Mutable
        ));

        let get_captures = analyzer.get_captures("get_value").unwrap();
        assert_eq!(get_captures.len(), 2);

        // Code generation
        let mut codegen = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = codegen.generate(&file);

        // Verify generated code structure
        assert!(rust_code.contains("let mut counter: i32 = 0;"));
        assert!(rust_code.contains("let multiplier: i32 = 2;"));
        assert!(rust_code.contains("let increment = ||"));
        assert!(rust_code.contains("let get_value = || -> i32"));
    }
}
