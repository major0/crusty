// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Pretty printing and formatting module

use crate::ast::*;
use crate::codegen::{CodeGenerator, TargetLanguage};

/// Pretty printer for formatting source code
pub struct PrettyPrinter {
    target: TargetLanguage,
}

impl PrettyPrinter {
    /// Create a new pretty printer for the specified target language
    pub fn new(target: TargetLanguage) -> Self {
        Self { target }
    }

    /// Format source code according to language conventions
    pub fn format(&self, code: &str) -> Result<String, String> {
        match self.target {
            TargetLanguage::Rust => self.format_rust(code),
            TargetLanguage::Crusty => self.format_crusty(code),
        }
    }

    /// Format Rust code using prettyplease
    fn format_rust(&self, code: &str) -> Result<String, String> {
        // Parse the Rust code into a syn AST
        let syntax_tree =
            syn::parse_file(code).map_err(|e| format!("Failed to parse Rust code: {}", e))?;

        // Use prettyplease to format the AST
        let formatted = prettyplease::unparse(&syntax_tree);

        Ok(formatted)
    }

    /// Format Crusty code
    fn format_crusty(&self, code: &str) -> Result<String, String> {
        // Parse the Crusty code into an AST
        use crate::parser::Parser;

        let mut parser =
            Parser::new(code).map_err(|e| format!("Failed to create parser: {:?}", e))?;
        let file = parser
            .parse_file()
            .map_err(|e| format!("Failed to parse Crusty code: {:?}", e))?;

        // Regenerate Crusty code from AST with proper formatting
        let mut generator = CodeGenerator::new(TargetLanguage::Crusty);
        let formatted = generator.generate(&file);

        Ok(formatted)
    }

    /// Format an AST as Rust code
    pub fn format_ast_as_rust(&self, file: &File) -> Result<String, String> {
        // Generate Rust code from AST
        let mut generator = CodeGenerator::new(TargetLanguage::Rust);
        let code = generator.generate(file);

        // Format the generated code
        self.format_rust(&code)
    }

    /// Format an AST as Crusty code
    pub fn format_ast_as_crusty(&self, file: &File) -> Result<String, String> {
        // Generate Crusty code from AST
        let mut generator = CodeGenerator::new(TargetLanguage::Crusty);
        let code = generator.generate(file);

        // Format the generated code (currently just returns as-is)
        Ok(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_pretty_printer() {
        let printer = PrettyPrinter::new(TargetLanguage::Rust);
        assert_eq!(printer.target, TargetLanguage::Rust);
    }

    #[test]
    fn test_format_simple_rust_code() {
        let printer = PrettyPrinter::new(TargetLanguage::Rust);
        let code = "fn main(){println!(\"Hello\");}";
        let result = printer.format(code);
        assert!(result.is_ok());
        let formatted = result.unwrap();
        // prettyplease should format this nicely
        assert!(formatted.contains("fn main()"));
        assert!(formatted.contains("println!"));
    }

    #[test]
    fn test_format_invalid_rust_code() {
        let printer = PrettyPrinter::new(TargetLanguage::Rust);
        let code = "fn main( { invalid syntax";
        let result = printer.format(code);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_rust_function() {
        let printer = PrettyPrinter::new(TargetLanguage::Rust);
        let code = r#"
pub fn add(a:i32,b:i32)->i32{
return a+b;
}
"#;
        let result = printer.format(code);
        assert!(result.is_ok());
        let formatted = result.unwrap();
        // Should have proper spacing and formatting
        assert!(formatted.contains("pub fn add"));
        assert!(formatted.contains("a: i32"));
        assert!(formatted.contains("b: i32"));
        assert!(formatted.contains("-> i32"));
    }

    #[test]
    fn test_format_ast_as_rust() {
        let printer = PrettyPrinter::new(TargetLanguage::Rust);
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::empty(),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let result = printer.format_ast_as_rust(&file);
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert!(formatted.contains("pub fn test()"));
    }

    #[test]
    fn test_format_crusty_code() {
        let printer = PrettyPrinter::new(TargetLanguage::Crusty);
        // Simple Crusty code
        let code = "int main() { return 0; }";
        let result = printer.format(code);
        // Should successfully format
        assert!(result.is_ok());
        let formatted = result.unwrap();
        println!("Formatted Crusty code:\n{}", formatted);
        // In Crusty, int is the return type, main is the function name
        assert!(formatted.contains("main()"));
    }
}
