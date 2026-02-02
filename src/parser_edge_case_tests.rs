// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Parser edge case and error path tests for coverage

#[cfg(test)]
mod tests {
    use crate::parser::Parser;

    // ========== Error Path Tests ==========

    #[test]
    fn test_parse_error_missing_function_name() {
        let source = "int () { return 0; }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_err());
        }
    }

    #[test]
    fn test_parse_error_missing_param_name() {
        let source = "int func(int) { return 0; }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_err());
        }
    }

    #[test]
    fn test_parse_error_missing_struct_name() {
        let source = "struct { int x; }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_err());
        }
    }

    #[test]
    fn test_parse_error_missing_enum_name() {
        let source = "enum { A, B }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_err());
        }
    }

    #[test]
    fn test_parse_error_missing_typedef_name() {
        let source = "typedef int;";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_err());
        }
    }

    #[test]
    fn test_parse_error_invalid_macro_name_no_prefix() {
        let source = "#define MAX__ 100";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_err());
        }
    }

    #[test]
    fn test_parse_error_invalid_macro_name_no_suffix() {
        let source = "#define __MAX 100";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_err());
        }
    }

    #[test]
    fn test_parse_error_missing_attribute_name() {
        let source = "#[] int func() { return 0; }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_err());
        }
    }

    #[test]
    fn test_parse_error_missing_enum_variant_name() {
        let source = "enum Color { , Red }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_err());
        }
    }

    #[test]
    fn test_parse_error_invalid_enum_variant_value() {
        let source = "enum Color { Red = abc }";
        let result = Parser::new(source);
        assert!(result.is_ok());
        if let Ok(mut parser) = result {
            let file = parser.parse_file();
            assert!(file.is_err());
        }
    }

    // ========== Edge Case Tests ==========

    #[test]
    fn test_parse_multiple_items() {
        let source = r#"
            int func1() { return 1; }
            int func2() { return 2; }
            struct Point { int x; int y; }
            enum Color { Red, Green }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 4);
    }

    #[test]
    fn test_parse_nested_statements() {
        let source = r#"
            void func() {
                if (true) {
                    if (false) {
                        int x = 5;
                    }
                }
            }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_empty_function_body() {
        let source = "void func() { }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_function_with_single_statement() {
        let source = "void func() { return; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_struct_empty() {
        let source = "struct Empty { }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_enum_single_variant() {
        let source = "enum Single { Only }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_let_without_init() {
        let source = "void func() { let x; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_var_without_init() {
        let source = "void func() { var x; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_const_with_explicit_type() {
        let source = "void func() { const int MAX = 100; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_multiple_attributes() {
        let source = r#"
            #[derive(Debug)]
            #[test]
            #[inline]
            int func() { return 0; }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_attribute_with_multiple_args() {
        let source = r#"
            #[cfg(target, os = "linux", arch = "x86_64")]
            int func() { return 0; }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_attribute_with_bool_arg() {
        let source = r#"
            #[inline(true)]
            int func() { return 0; }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_method_with_self_param() {
        let source = r#"
            struct Point {
                int get_x(self) {
                    return 0;
                }
            }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_method_with_mut_self_param() {
        let source = r#"
            struct Point {
                void set_x(&mut self) {
                    return;
                }
            }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_method_with_regular_params() {
        let source = r#"
            struct Point {
                int add(int a, int b) {
                    return a + b;
                }
            }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_static_method() {
        let source = r#"
            struct Point {
                static int new() {
                    return 0;
                }
            }
        "#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_for_in_simple() {
        let source = "void func() { for (item in collection) { } }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_c_style_for_with_var_init() {
        let source = "void func() { for (var i = 0; i < 10; i++) { } }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_c_style_for_with_let_init() {
        let source = "void func() { for (let i = 0; i < 10; i++) { } }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_ternary_expression() {
        let source = "void func() { int x = a > b ? a : b; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_nested_ternary() {
        let source = "void func() { int x = a > b ? (c > d ? c : d) : b; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_chained_comparisons() {
        let source = "void func() { bool b = a < b && b < c; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_complex_boolean_expression() {
        let source = "void func() { bool b = (a && b) || (c && d); }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_simple_assignment() {
        let source = "void func() { x = 0; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_field_access_on_call_result() {
        let source = "void func() { int x = get_point().x; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_method_call_on_field() {
        let source = "void func() { obj.field.method(); }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_array_of_function_calls() {
        let source = "void func() { let arr = [f(), g(), h()]; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_tuple_of_expressions() {
        let source = "void func() { let t = (a + b, c * d, e - f); }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_nested_array_access() {
        let source = "void func() { int x = matrix[i][j]; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_function_call_with_complex_args() {
        let source = "void func() { result = calculate(a + b, c * d, e / f); }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_mixed_operators() {
        let source = "void func() { int x = a + b * c - d / e % f; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_bitwise_and_or() {
        let source = "void func() { int x = a & b | c; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_shift_operations() {
        let source = "void func() { int x = (a << 2) | (b >> 3); }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_multiple_unary_operators() {
        let source = "void func() { int x = -(-value); }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_pointer_operations() {
        let source = "void func() { int x = *&value; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_increment_in_expression() {
        let source = "void func() { int x = ++i + j++; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_decrement_in_expression() {
        let source = "void func() { int x = --i + j--; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_cast_in_expression() {
        let source = "void func() { int x = (int)value + 5; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_multiple_casts() {
        let source = "void func() { int x = (int)(float)value; }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }

    #[test]
    fn test_parse_parenthesized_complex_expression() {
        let source = "void func() { int x = ((a + b) * (c - d)) / ((e + f) * (g - h)); }";
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        assert_eq!(file.items.len(), 1);
    }
}
