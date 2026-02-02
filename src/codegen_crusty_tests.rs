// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Tests for Crusty target language code generation

#[cfg(test)]
mod tests {
    use crate::codegen::{CodeGenerator, TargetLanguage};
    use crate::parser::Parser;

    #[test]
    fn test_crusty_let_statement() {
        let source = r#"
void main() {
    let int x = 42;
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Crusty);
        let crusty_code = codegen.generate(&file);

        assert!(crusty_code.contains("let x"));
        assert!(crusty_code.contains("(int)42"));
    }

    #[test]
    fn test_crusty_var_statement() {
        let source = r#"
void main() {
    var int x = 42;
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Crusty);
        let crusty_code = codegen.generate(&file);

        assert!(crusty_code.contains("var x"));
        assert!(crusty_code.contains("(int)42"));
    }

    #[test]
    fn test_crusty_const_statement() {
        let source = r#"
void main() {
    const int PI = 3;
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Crusty);
        let crusty_code = codegen.generate(&file);

        assert!(crusty_code.contains("const PI"));
        assert!(crusty_code.contains("(int)3"));
    }

    #[test]
    fn test_crusty_if_statement() {
        let source = r#"
void main() {
    if (x > 0) {
        return;
    }
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Crusty);
        let crusty_code = codegen.generate(&file);

        assert!(crusty_code.contains("if"));
        assert!(crusty_code.contains("x > 0"));
    }

    #[test]
    fn test_crusty_while_statement() {
        let source = r#"
void main() {
    while (x > 0) {
        x = x - 1;
    }
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Crusty);
        let crusty_code = codegen.generate(&file);

        // The parser removes the parentheses, codegen adds them back for Crusty
        assert!(crusty_code.contains("while"));
        assert!(crusty_code.contains("x > 0"));
    }

    #[test]
    fn test_crusty_while_with_label() {
        let source = r#"
void main() {
    .loop_label: while (true) {
        break loop_label;
    }
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Crusty);
        let crusty_code = codegen.generate(&file);

        assert!(crusty_code.contains(".loop_label:"));
        assert!(crusty_code.contains("while (true)"));
    }

    #[test]
    fn test_crusty_function_signature() {
        let source = r#"
int add(int x, int y) {
    return x + y;
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Crusty);
        let crusty_code = codegen.generate(&file);

        assert!(crusty_code.contains("int add(int x, int y)"));
    }

    #[test]
    fn test_crusty_private_function() {
        let source = r#"
static int helper() {
    return 42;
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Crusty);
        let crusty_code = codegen.generate(&file);

        assert!(crusty_code.contains("static int helper()"));
    }

    #[test]
    fn test_crusty_void_function() {
        let source = r#"
void do_something() {
    return;
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Crusty);
        let crusty_code = codegen.generate(&file);

        assert!(crusty_code.contains("void do_something()"));
    }

    #[test]
    fn test_crusty_let_without_init() {
        let source = r#"
void main() {
    let int x;
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Crusty);
        let crusty_code = codegen.generate(&file);

        assert!(crusty_code.contains("let x"));
    }

    #[test]
    fn test_crusty_var_without_init() {
        let source = r#"
void main() {
    var int x;
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Crusty);
        let crusty_code = codegen.generate(&file);

        assert!(crusty_code.contains("var x"));
    }

    #[test]
    fn test_crusty_let_with_cast() {
        let source = r#"
void main() {
    let int x = (int)42;
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Crusty);
        let crusty_code = codegen.generate(&file);

        assert!(crusty_code.contains("let x"));
        // Should contain the cast
        assert!(crusty_code.contains("int"));
        assert!(crusty_code.contains("42"));
    }

    #[test]
    fn test_crusty_var_with_cast() {
        let source = r#"
void main() {
    var int x = (int)42;
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Crusty);
        let crusty_code = codegen.generate(&file);

        assert!(crusty_code.contains("var x"));
        assert!(crusty_code.contains("int"));
        assert!(crusty_code.contains("42"));
    }

    #[test]
    fn test_crusty_const_with_cast() {
        let source = r#"
void main() {
    const int PI = (int)3;
}
"#;
        let mut parser = Parser::new(source).unwrap();
        let file = parser.parse_file().unwrap();

        let mut codegen = CodeGenerator::new(TargetLanguage::Crusty);
        let crusty_code = codegen.generate(&file);

        assert!(crusty_code.contains("const PI"));
        assert!(crusty_code.contains("int"));
        assert!(crusty_code.contains("3"));
    }
}
