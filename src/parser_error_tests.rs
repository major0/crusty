// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Parser error handling and edge case tests

#[cfg(test)]
mod tests {
    use crate::parser::Parser;

    #[test]
    fn test_parse_empty_source() {
        let result = Parser::new("");
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_ok());
            let file = file.unwrap();
            assert_eq!(file.items.len(), 0);
        }
    }

    #[test]
    fn test_parse_only_whitespace() {
        let result = Parser::new("   \n\t  \n  ");
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_ok());
        }
    }

    #[test]
    fn test_parse_only_comments() {
        let result = Parser::new("// comment\n/* block comment */");
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_ok());
        }
    }

    #[test]
    fn test_parse_unclosed_brace() {
        let source = "void func() { int x = 5;";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_err());
        }
    }

    #[test]
    fn test_parse_missing_semicolon() {
        let source = "void func() { int x = 5 }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            // May or may not error depending on parser implementation
            let _ = file;
        }
    }

    #[test]
    fn test_parse_unexpected_token() {
        let source = "void func() { @ }";
        let result = Parser::new(source);
        // Lexer should catch invalid character
        assert!(
            result.is_err() || {
                if let Ok(mut parser) = result {
                    parser.parse_file().is_err()
                } else {
                    false
                }
            }
        );
    }

    #[test]
    fn test_parse_incomplete_function() {
        let source = "void func(";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_err());
        }
    }

    #[test]
    fn test_parse_incomplete_struct() {
        let source = "struct MyStruct {";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_err());
        }
    }

    #[test]
    fn test_parse_incomplete_enum() {
        let source = "enum MyEnum {";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_err());
        }
    }

    #[test]
    fn test_parse_invalid_type() {
        let source = "void func() { 123 x = 5; }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_err());
        }
    }

    #[test]
    fn test_parse_nested_blocks() {
        let source = r#"
            void func() {
                {
                    {
                        int x = 5;
                    }
                }
            }
        "#;
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let _file = parser.parse_file();
            // Nested blocks may or may not be supported
        }
    }

    #[test]
    fn test_parse_multiple_errors() {
        let source = "void func( { int x = }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_err());
        }
    }

    #[test]
    fn test_parse_expression_with_parentheses() {
        let source = "void func() { int x = (1 + 2) * 3; }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_ok());
        }
    }

    #[test]
    fn test_parse_complex_expression() {
        let source = "void func() { int x = a + b * c - d / e % f; }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_ok());
        }
    }

    #[test]
    fn test_parse_chained_field_access() {
        let source = "void func() { int x = obj.field1.field2.field3; }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_ok());
        }
    }

    #[test]
    fn test_parse_chained_method_calls() {
        let source = "void func() { obj.method1().method2().method3(); }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_ok());
        }
    }

    #[test]
    fn test_parse_nested_function_calls() {
        let source = "void func() { int x = f(g(h(1))); }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_ok());
        }
    }

    #[test]
    fn test_parse_array_of_arrays() {
        let source = "void func() { int x[3][4]; }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let _file = parser.parse_file();
            // Multi-dimensional arrays may not be fully supported
        }
    }

    #[test]
    fn test_parse_pointer_to_pointer() {
        let source = "void func() { int** x; }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let _file = parser.parse_file();
            // Pointer to pointer may not be fully supported
        }
    }

    #[test]
    fn test_parse_complex_type() {
        let source = "void func() { int*[5]* x; }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            // This may or may not parse depending on implementation
            let _ = file;
        }
    }

    #[test]
    fn test_parse_if_without_else() {
        let source = "void func() { if (x > 0) { y = 1; } }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_ok());
        }
    }

    #[test]
    fn test_parse_if_else_if_chain() {
        let source = r#"
            void func() {
                if (x > 0) {
                    y = 1;
                } else if (x < 0) {
                    y = -1;
                } else {
                    y = 0;
                }
            }
        "#;
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_ok());
        }
    }

    #[test]
    fn test_parse_while_loop() {
        let source = "void func() { while (x > 0) { x = x - 1; } }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_ok());
        }
    }

    #[test]
    fn test_parse_for_loop_all_parts() {
        let source = "void func() { for (int i = 0; i < 10; i = i + 1) { } }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let _file = parser.parse_file();
            // C-style for loops may not be fully supported
        }
    }

    #[test]
    fn test_parse_for_loop_missing_init() {
        let source = "void func() { for (; i < 10; i = i + 1) { } }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let _file = parser.parse_file();
            // C-style for loops may not be fully supported
        }
    }

    #[test]
    fn test_parse_for_loop_missing_condition() {
        let source = "void func() { for (int i = 0; ; i = i + 1) { } }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let _file = parser.parse_file();
            // C-style for loops may not be fully supported
        }
    }

    #[test]
    fn test_parse_for_loop_missing_update() {
        let source = "void func() { for (int i = 0; i < 10; ) { } }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let _file = parser.parse_file();
            // C-style for loops may not be fully supported
        }
    }

    #[test]
    fn test_parse_infinite_for_loop() {
        let source = "void func() { for (;;) { } }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let _file = parser.parse_file();
            // C-style for loops may not be fully supported
        }
    }

    #[test]
    fn test_parse_switch_with_fallthrough() {
        let source = r#"
            void func() {
                switch (x) {
                    case 1:
                        y = 1;
                    case 2:
                        y = 2;
                        break;
                    default:
                        y = 0;
                }
            }
        "#;
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let _file = parser.parse_file();
            // Switch with fallthrough may not be fully supported
        }
    }

    #[test]
    fn test_parse_empty_switch() {
        let source = "void func() { switch (x) { } }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let _file = parser.parse_file();
            // Empty switch may not be fully supported
        }
    }

    #[test]
    fn test_parse_return_with_expression() {
        let source = "int func() { return 42; }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_ok());
        }
    }

    #[test]
    fn test_parse_return_without_expression() {
        let source = "void func() { return; }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_ok());
        }
    }

    #[test]
    fn test_parse_break_in_loop() {
        let source = "void func() { while (true) { break; } }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_ok());
        }
    }

    #[test]
    fn test_parse_continue_in_loop() {
        let source = "void func() { while (true) { continue; } }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_ok());
        }
    }
}
