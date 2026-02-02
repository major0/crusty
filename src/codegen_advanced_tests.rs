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
