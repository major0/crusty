// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Advanced code generation tests for uncovered code paths

#[cfg(test)]
mod tests {
    use crate::codegen::{CodeGenerator, TargetLanguage};
    use crate::parser::Parser;

    #[test]
    fn test_generate_const_statement() {
        let source = r#"
int main() {
    const int PI = 3;
    return PI;
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = codegen.generate(&file);

        assert!(rust_code.contains("const PI"));
        assert!(rust_code.contains("i32"));
    }

    #[test]
    fn test_generate_method_call() {
        let source = r#"
void main() {
    int x = obj.method(42);
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = codegen.generate(&file);

        assert!(rust_code.contains(".method"));
    }

    #[test]
    fn test_generate_unary_ref() {
        let source = r#"
void main() {
    int x = 42;
    int* ptr = &x;
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = codegen.generate(&file);

        assert!(rust_code.contains("&"));
    }

    #[test]
    fn test_generate_pre_increment() {
        let source = r#"
void main() {
    var int x = 0;
    ++x;
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = codegen.generate(&file);

        assert!(rust_code.contains("+="));
    }

    #[test]
    fn test_generate_pre_decrement() {
        let source = r#"
void main() {
    var int x = 10;
    --x;
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = codegen.generate(&file);

        assert!(rust_code.contains("-="));
    }

    #[test]
    fn test_generate_post_increment() {
        let source = r#"
void main() {
    var int x = 0;
    x++;
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = codegen.generate(&file);

        assert!(rust_code.contains("+="));
    }

    #[test]
    fn test_generate_post_decrement() {
        let source = r#"
void main() {
    var int x = 10;
    x--;
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = codegen.generate(&file);

        assert!(rust_code.contains("-="));
    }

    #[test]
    fn test_post_increment_borrows_place_once() {
        // Regression (CG-3): x++ must borrow the place ONCE — no double
        // evaluation of the operand and no `&mut` of a copy. The old codegen
        // emitted `let __old = (x); let __tmp = &mut (x); …` (operand twice).
        let source = r#"
void main() {
    var int x = 0;
    let y = x++;
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        let mut codegen = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = codegen.generate(&file);

        assert_eq!(rust_code.matches("&mut").count(), 1, "one borrow only: {rust_code}");
        assert!(rust_code.contains("let __old = *__p"), "old value via borrow: {rust_code}");
    }

    #[test]
    fn test_compound_assignment_is_not_parenthesised() {
        // Regression (CG-2): `i += 1;` as a statement must emit `i += 1;`, not
        // `(i += 1);` — a parenthesised assignment is not a valid Rust statement.
        let source = r#"
void main() {
    var int i = 0;
    i += 1;
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();
        let mut codegen = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = codegen.generate(&file);

        assert!(rust_code.contains("i += 1;"), "unparenthesised compound assign: {rust_code}");
        assert!(!rust_code.contains("(i += 1)"), "no wrapping parens: {rust_code}");
    }

    #[test]
    fn test_generate_bool_literal() {
        let source = r#"
void main() {
    bool t = true;
    bool f = false;
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = codegen.generate(&file);

        assert!(rust_code.contains("true"));
        assert!(rust_code.contains("false"));
    }

    #[test]
    fn test_generate_int_literal() {
        let source = r#"
void main() {
    int x = 42;
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = codegen.generate(&file);

        assert!(rust_code.contains("42"));
    }

    #[test]
    fn test_generate_float_literal() {
        let source = r#"
void main() {
    float x = 3.14;
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = codegen.generate(&file);

        assert!(rust_code.contains("3.14"));
    }

    #[test]
    fn test_generate_whole_valued_float_keeps_decimal() {
        // Regression: a whole-valued float literal (2.0) must emit as `2.0`,
        // not `2` — otherwise rustc infers an integer in a float context and
        // rejects it (e.g. `f64 * {integer}`).
        let source = r#"
void main() {
    float x = 2.0;
    float y = 0.0;
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = codegen.generate(&file);

        assert!(rust_code.contains("2.0"), "got: {rust_code}");
        assert!(rust_code.contains("0.0"), "got: {rust_code}");
        // Must NOT emit the bare integer form that trips integer inference.
        assert!(!rust_code.contains("= 2;"), "got: {rust_code}");
        assert!(!rust_code.contains("= 0;"), "got: {rust_code}");
    }

    #[test]
    fn test_generate_tuple_literal() {
        let source = r#"
void main() {
    let t = (1, 2, 3);
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = codegen.generate(&file);

        assert!(rust_code.contains("(1, 2, 3)"));
    }

    #[test]
    fn test_generate_array_literal() {
        let source = r#"
void main() {
    let arr = [1, 2, 3];
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = codegen.generate(&file);

        assert!(rust_code.contains("[1, 2, 3]"));
    }

    #[test]
    fn test_generate_comparison_operations() {
        let source = r#"
void main() {
    bool a = 1 < 2;
    bool b = 3 > 4;
    bool c = 5 <= 6;
    bool d = 7 >= 8;
    bool e = 9 == 10;
    bool f = 11 != 12;
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = codegen.generate(&file);

        assert!(rust_code.contains("<"));
        assert!(rust_code.contains(">"));
        assert!(rust_code.contains("<="));
        assert!(rust_code.contains(">="));
        assert!(rust_code.contains("=="));
        assert!(rust_code.contains("!="));
    }

    #[test]
    fn test_generate_logical_operations() {
        let source = r#"
void main() {
    bool a = true && false;
    bool b = true || false;
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = codegen.generate(&file);

        assert!(rust_code.contains("&&"));
        assert!(rust_code.contains("||"));
    }

    #[test]
    fn test_generate_function_call() {
        let source = r#"
int add(int x, int y) {
    return x + y;
}

void main() {
    int result = add(1, 2);
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = codegen.generate(&file);

        assert!(rust_code.contains("add(1, 2)"));
    }

    #[test]
    fn test_generate_nested_expressions() {
        let source = r#"
void main() {
    int result = (1 + 2) * (3 + 4);
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = codegen.generate(&file);

        assert!(rust_code.contains("1 + 2"));
        assert!(rust_code.contains("3 + 4"));
    }
}
