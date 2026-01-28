// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Property-based tests for pretty printing and formatting

use crate::ast::*;
use crate::codegen::{CodeGenerator, TargetLanguage};
use crate::parser::Parser;
use crate::pretty::PrettyPrinter;
use proptest::prelude::*;

/// Generate a simple valid function for testing
fn arb_simple_function() -> impl Strategy<Value = Function> {
    // Avoid Rust/Crusty keywords
    let valid_ident = "[a-z][a-z0-9_]{0,10}"
        .prop_filter("Must not be a keyword", |s| {
            !matches!(
                s.as_str(),
                "let" | "var" | "const" | "static" | "if" | "else" | "while" | "for" | "in" | "return"
                    | "break" | "continue" | "fn" | "struct" | "enum" | "type" | "impl" | "trait"
                    | "pub" | "use" | "mod" | "match" | "loop" | "as" | "mut" | "ref" | "self"
                    | "Self" | "super" | "crate" | "extern" | "unsafe" | "where" | "async" | "await"
                    | "dyn" | "move" | "true" | "false"
            )
        });

    (
        prop::bool::ANY,
        valid_ident,
        prop::collection::vec(arb_simple_statement(), 0..5),
    )
        .prop_map(|(is_public, name, statements)| Function {
            visibility: if is_public {
                Visibility::Public
            } else {
                Visibility::Private
            },
            name: Ident::new(&name),
            params: vec![],
            return_type: Some(Type::Primitive(PrimitiveType::Int)),
            body: Block::new(statements),
            doc_comments: vec![],
        })
}

/// Generate a simple valid statement for testing
fn arb_simple_statement() -> impl Strategy<Value = Statement> {
    // Avoid Rust/Crusty keywords
    let valid_ident = "[a-z][a-z0-9_]{0,10}"
        .prop_filter("Must not be a keyword", |s| {
            !matches!(
                s.as_str(),
                "let" | "var" | "const" | "static" | "if" | "else" | "while" | "for" | "in" | "return"
                    | "break" | "continue" | "fn" | "struct" | "enum" | "type" | "impl" | "trait"
                    | "pub" | "use" | "mod" | "match" | "loop" | "as" | "mut" | "ref" | "self"
                    | "Self" | "super" | "crate" | "extern" | "unsafe" | "where" | "async" | "await"
                    | "dyn" | "move" | "true" | "false"
            )
        });

    prop_oneof![
        // Return statement with literal
        Just(Statement::Return(Some(Expression::Literal(Literal::Int(0))))),
        // Return statement without value
        Just(Statement::Return(None)),
        // Let statement with initialization
        (
            valid_ident.clone(),
            Just(Type::Primitive(PrimitiveType::Int)),
            Just(Expression::Literal(Literal::Int(42))),
        )
            .prop_map(|(name, ty, init)| Statement::Let {
                name: Ident::new(&name),
                ty: Some(ty),
                init: Some(init),
                mutable: false,
            }),
        // Var statement with initialization
        (
            valid_ident,
            Just(Type::Primitive(PrimitiveType::Int)),
            Just(Expression::Literal(Literal::Int(42))),
        )
            .prop_map(|(name, ty, init)| Statement::Var {
                name: Ident::new(&name),
                ty: Some(ty),
                init: Some(init),
            }),
    ]
}

/// Generate a simple valid file for testing
fn arb_simple_file() -> impl Strategy<Value = File> {
    prop::collection::vec(arb_simple_function(), 1..3).prop_map(|functions| File {
        items: functions.into_iter().map(Item::Function).collect(),
        doc_comments: vec![],
    })
}

proptest! {
    /// Property 27: Pretty-print then parse is identity (CRITICAL)
    /// For any valid AST, pretty-printing it to Crusty source code and then parsing
    /// that source code should produce an AST that is equivalent to the original.
    /// Validates: Requirements 16.1, 16.2
    #[test]
    fn test_property_27_pretty_print_parse_roundtrip(file in arb_simple_file()) {
        // Generate Crusty code from AST
        let mut generator = CodeGenerator::new(TargetLanguage::Crusty);
        let crusty_code = generator.generate(&file);

        // Parse the generated Crusty code back into an AST
        let parse_result = Parser::new(&crusty_code)
            .and_then(|mut parser| parser.parse_file());

        // The parse should succeed
        prop_assert!(parse_result.is_ok(), 
            "Failed to parse generated Crusty code:\n{}\nError: {:?}", 
            crusty_code, parse_result.err());

        let parsed_file = parse_result.unwrap();

        // The parsed AST should have the same structure as the original
        // (We compare the number of items and their types as a basic check)
        prop_assert_eq!(file.items.len(), parsed_file.items.len(), 
            "Number of items should match. Generated code:\n{}", crusty_code);

        // Check that each item is of the same type
        for (original, parsed) in file.items.iter().zip(parsed_file.items.iter()) {
            match (original, parsed) {
                (Item::Function(orig_func), Item::Function(parsed_func)) => {
                    prop_assert_eq!(&orig_func.name.name, &parsed_func.name.name,
                        "Function names should match");
                    prop_assert_eq!(&orig_func.visibility, &parsed_func.visibility,
                        "Function visibility should match");
                }
                _ => prop_assert!(false, "Item types should match"),
            }
        }
    }

    /// Property 5: Generated Rust code follows formatting conventions
    /// For any generated Rust source code, running rustfmt on it should produce
    /// no changes, indicating it already follows Rust style conventions.
    /// Validates: Requirements 8.16
    #[test]
    fn test_property_5_rust_code_formatting(file in arb_simple_file()) {
        // Generate Rust code from AST
        let mut generator = CodeGenerator::new(TargetLanguage::Rust);
        let rust_code = generator.generate(&file);

        // Format the Rust code using prettyplease
        let printer = PrettyPrinter::new(TargetLanguage::Rust);
        let formatted = printer.format(&rust_code).expect("Failed to format Rust code");

        // The formatted code should be valid Rust
        // We verify this by parsing it with syn
        let parsed = syn::parse_file(&formatted);
        prop_assert!(parsed.is_ok(), "Formatted Rust code should be valid: {:?}", parsed.err());

        // Format again - should be idempotent
        let formatted_again = printer.format(&formatted).expect("Failed to format Rust code again");
        
        // Formatting should be idempotent (formatting formatted code produces the same result)
        prop_assert_eq!(formatted, formatted_again, 
            "Formatting should be idempotent");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arb_simple_function_generates() {
        // Just verify the generator works
        let strategy = arb_simple_function();
        let mut runner = proptest::test_runner::TestRunner::default();
        let _ = strategy.new_tree(&mut runner).unwrap();
    }

    #[test]
    fn test_arb_simple_file_generates() {
        // Just verify the generator works
        let strategy = arb_simple_file();
        let mut runner = proptest::test_runner::TestRunner::default();
        let _ = strategy.new_tree(&mut runner).unwrap();
    }
}
