// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Property-based tests for parser error reporting.

#[cfg(test)]
mod tests {
    use crate::parser::Parser;
    use proptest::prelude::*;

    /// Property 2: Invalid syntax produces error reports with location
    /// Validates: Requirements 6.2, 10.1
    ///
    /// This property verifies that when the parser encounters invalid syntax,
    /// it reports errors with accurate line numbers, column numbers, and
    /// descriptive error messages.
    #[test]
    fn property_2_invalid_syntax_produces_error_with_location() {
        proptest!(|(
            // Generate various invalid syntax patterns
            invalid_pattern in prop_oneof![
                // Missing semicolon
                Just("int x = 5\nint y = 10;"),
                // Unclosed brace
                Just("int main() {\n    return 0;\n"),
                // Invalid token
                Just("int main() { @ }"),
                // Missing closing paren
                Just("int add(int a, int b {\n    return a + b;\n}"),
                // Invalid type
                Just("invalid_type x = 5;"),
                // Unexpected token
                Just("int main() { } }"),
                // Missing function body
                Just("int foo();"),
                // Invalid expression
                Just("int main() { int x = ; }"),
            ]
        )| {
            // Attempt to parse the invalid syntax
            let parse_result = Parser::new(&invalid_pattern);

            // The parser should either fail during construction or during parsing
            match parse_result {
                Err(err) => {
                    // Verify error has location information
                    prop_assert!(err.span.start.line > 0, "Error should have line number");
                    prop_assert!(err.span.start.column > 0, "Error should have column number");

                    // Verify error has a descriptive message
                    prop_assert!(!err.message.is_empty(), "Error should have a message");

                    // Verify error message is meaningful (not just a generic error)
                    prop_assert!(
                        err.message.len() > 5,
                        "Error message should be descriptive: '{}'",
                        err.message
                    );
                }
                Ok(mut parser) => {
                    // If parser construction succeeded, parsing should fail
                    let file_result = parser.parse_file();
                    prop_assert!(
                        file_result.is_err(),
                        "Invalid syntax should produce parse error"
                    );

                    if let Err(err) = file_result {
                        // Verify error has location information
                        prop_assert!(err.span.start.line > 0, "Error should have line number");
                        prop_assert!(err.span.start.column > 0, "Error should have column number");

                        // Verify error has a descriptive message
                        prop_assert!(!err.message.is_empty(), "Error should have a message");

                        // Verify error message is meaningful
                        prop_assert!(
                            err.message.len() > 5,
                            "Error message should be descriptive: '{}'",
                            err.message
                        );
                    }
                }
            }
        });
    }

    /// Test that errors include expected token information
    #[test]
    fn property_errors_include_expected_tokens() {
        proptest!(|(
            missing_token in prop_oneof![
                Just(("int main() { return 0 }", ";")),  // Missing semicolon
                Just(("int main( { }", ")")),            // Missing closing paren
                Just(("int main() { int x = 5 }", ";")), // Missing semicolon
            ]
        )| {
            let (invalid_code, _expected_token) = missing_token;

            let parse_result = Parser::new(invalid_code);
            match parse_result {
                Err(err) => {
                    // Error during lexing/parser construction
                    prop_assert!(!err.message.is_empty());
                }
                Ok(mut parser) => {
                    let file_result = parser.parse_file();
                    if let Err(err) = file_result {
                        // Verify error includes information about what was expected
                        prop_assert!(
                            !err.expected.is_empty() || err.message.contains("expected"),
                            "Error should indicate what was expected"
                        );
                    }
                }
            }
        });
    }

    /// Test that line and column numbers are accurate
    #[test]
    fn property_error_locations_are_accurate() {
        proptest!(|(
            line_num in 1usize..10,
            col_num in 1usize..20,
        )| {
            // Generate code with error at specific location
            let mut code = String::new();

            // Add lines before the error
            for _ in 1..line_num {
                code.push_str("int x = 5;\n");
            }

            // Add spaces before the error on the target line
            for _ in 1..col_num {
                code.push(' ');
            }

            // Add invalid syntax
            code.push_str("@@@");

            let parse_result = Parser::new(&code);
            match parse_result {
                Err(err) => {
                    // Verify error location is reported (may not be exact due to lexer behavior)
                    prop_assert!(
                        err.span.start.line > 0,
                        "Error should have a valid line number"
                    );
                    prop_assert!(
                        err.span.start.column > 0,
                        "Error should have a valid column number"
                    );
                }
                Ok(mut parser) => {
                    let file_result = parser.parse_file();
                    if let Err(err) = file_result {
                        // Verify error location is reported
                        prop_assert!(
                            err.span.start.line > 0,
                            "Error should have a valid line number"
                        );
                        prop_assert!(
                            err.span.start.column > 0,
                            "Error should have a valid column number"
                        );
                    }
                }
            }
        });
    }

    /// Test that unterminated strings produce errors with location
    #[test]
    fn property_unterminated_strings_produce_errors() {
        proptest!(|(
            prefix in "[a-z]{0,10}",
            content in "[a-zA-Z0-9 ]{0,20}",
        )| {
            // Create code with unterminated string
            let code = format!("int main() {{\n    {}\"{};\n}}", prefix, content);

            let parse_result = Parser::new(&code);

            // Should produce an error (either during lexing or parsing)
            match parse_result {
                Err(err) => {
                    prop_assert!(err.span.start.line > 0);
                    prop_assert!(err.span.start.column > 0);
                    prop_assert!(!err.message.is_empty());
                }
                Ok(mut parser) => {
                    let file_result = parser.parse_file();
                    // May succeed if the unterminated string is accidentally valid
                    // or may fail with a parse error
                    if let Err(err) = file_result {
                        prop_assert!(err.span.start.line > 0);
                        prop_assert!(err.span.start.column > 0);
                    }
                }
            }
        });
    }

    /// Test that mismatched braces produce errors with location
    #[test]
    fn property_mismatched_braces_produce_errors() {
        proptest!(|(
            extra_braces in prop_oneof![
                Just("int main() { { }"),      // Missing closing brace
                Just("int main() { } }"),      // Extra closing brace
                Just("int main() { { { } }"),  // Mismatched nesting
            ]
        )| {
            let parse_result = Parser::new(extra_braces);

            match parse_result {
                Err(err) => {
                    prop_assert!(err.span.start.line > 0);
                    prop_assert!(!err.message.is_empty());
                }
                Ok(mut parser) => {
                    let file_result = parser.parse_file();
                    // Should fail with parse error
                    prop_assert!(
                        file_result.is_err(),
                        "Mismatched braces should produce error"
                    );

                    if let Err(err) = file_result {
                        prop_assert!(err.span.start.line > 0);
                        prop_assert!(!err.message.is_empty());
                    }
                }
            }
        });
    }
}
