// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Property-based tests for code generation

#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::codegen::{CodeGenerator, TargetLanguage};
    use proptest::prelude::*;

    // Helper to create simple valid AST nodes for testing
    fn arb_ident() -> impl Strategy<Value = Ident> {
        "[a-z][a-z0-9_]{0,10}"
            .prop_filter("Must not be a Rust keyword", |s| {
                !matches!(
                    s.as_str(),
                    "let"
                        | "var"
                        | "const"
                        | "static"
                        | "if"
                        | "else"
                        | "while"
                        | "for"
                        | "in"
                        | "return"
                        | "break"
                        | "continue"
                        | "fn"
                        | "struct"
                        | "enum"
                        | "type"
                        | "impl"
                        | "trait"
                        | "pub"
                        | "use"
                        | "mod"
                        | "match"
                        | "loop"
                        | "as"
                        | "mut"
                        | "ref"
                        | "self"
                        | "Self"
                        | "super"
                        | "crate"
                        | "extern"
                        | "unsafe"
                        | "where"
                        | "async"
                        | "await"
                        | "dyn"
                        | "move"
                        | "true"
                        | "false"
                        | "do"
                        | "box"
                        | "yield"
                        | "become"
                        | "abstract"
                        | "final"
                        | "override"
                        | "macro"
                        | "priv"
                        | "typeof"
                        | "unsized"
                        | "virtual"
                        | "try"
                )
            })
            .prop_map(|s| Ident::new(s))
    }

    fn arb_primitive_type() -> impl Strategy<Value = Type> {
        prop_oneof![
            Just(Type::Primitive(PrimitiveType::I32)),
            Just(Type::Primitive(PrimitiveType::I64)),
            Just(Type::Primitive(PrimitiveType::U32)),
            Just(Type::Primitive(PrimitiveType::U64)),
            Just(Type::Primitive(PrimitiveType::F32)),
            Just(Type::Primitive(PrimitiveType::F64)),
            Just(Type::Primitive(PrimitiveType::Bool)),
            Just(Type::Primitive(PrimitiveType::Char)),
            Just(Type::Primitive(PrimitiveType::Void)),
        ]
    }

    fn arb_literal() -> impl Strategy<Value = Literal> {
        prop_oneof![
            any::<i64>().prop_map(Literal::Int),
            any::<f64>().prop_map(Literal::Float),
            "[a-zA-Z0-9 ]{0,20}".prop_map(Literal::String),
            any::<char>().prop_map(Literal::Char),
            any::<bool>().prop_map(Literal::Bool),
        ]
    }

    fn arb_simple_expression() -> impl Strategy<Value = Expression> {
        prop_oneof![
            arb_literal().prop_map(Expression::Literal),
            arb_ident().prop_map(Expression::Ident),
        ]
    }

    fn arb_visibility() -> impl Strategy<Value = Visibility> {
        prop_oneof![Just(Visibility::Public), Just(Visibility::Private),]
    }

    fn arb_simple_function() -> impl Strategy<Value = Function> {
        (arb_visibility(), arb_ident(), arb_primitive_type()).prop_map(|(vis, name, ret_type)| {
            Function {
                visibility: vis,
                name,
                params: vec![],
                return_type: Some(ret_type),
                body: Block::empty(),
                doc_comments: vec![],
                attributes: vec![],
            }
        })
    }

    fn arb_simple_struct() -> impl Strategy<Value = Struct> {
        (arb_visibility(), arb_ident()).prop_map(|(vis, name)| Struct {
            visibility: vis,
            name,
            fields: vec![],
            methods: vec![],
            doc_comments: vec![],
            attributes: vec![],
        })
    }

    fn arb_simple_enum() -> impl Strategy<Value = Enum> {
        (arb_visibility(), arb_ident()).prop_map(|(vis, name)| Enum {
            visibility: vis,
            name,
            variants: vec![
                EnumVariant {
                    name: Ident::new("Variant1"),
                    value: Some(0),
                },
                EnumVariant {
                    name: Ident::new("Variant2"),
                    value: Some(1),
                },
            ],
            doc_comments: vec![],
            attributes: vec![],
        })
    }

    // Property 4: Generated Rust code is syntactically valid
    // Validates: Requirements 8.1
    proptest! {
        #[test]
        fn prop_generated_rust_is_syntactically_valid(func in arb_simple_function()) {
            let mut gen = CodeGenerator::new(TargetLanguage::Rust);
            let file = File {
                items: vec![Item::Function(func)],
                doc_comments: vec![],
            };
            let output = gen.generate(&file);

            // The output should be parseable by syn
            let parse_result = syn::parse_file(&output);
            prop_assert!(parse_result.is_ok(), "Generated Rust code should be syntactically valid: {}", output);
        }
    }

    // Property 6: Transparent syntax preservation
    // Validates: Requirements 19.7, 20.4, 23.6, 25.8, 26.8
    proptest! {
        #[test]
        fn prop_transparent_syntax_preservation_tuples(elements in prop::collection::vec(arb_simple_expression(), 0..5)) {
            let gen = CodeGenerator::new(TargetLanguage::Rust);
            let expr = Expression::TupleLit { elements };
            let output = gen.generate_expression_string(&expr);

            // Tuple syntax should be preserved (parentheses with comma-separated elements)
            prop_assert!(output.starts_with('('));
            prop_assert!(output.ends_with(')'));
        }

        #[test]
        fn prop_transparent_syntax_preservation_arrays(elements in prop::collection::vec(arb_simple_expression(), 0..5)) {
            let gen = CodeGenerator::new(TargetLanguage::Rust);
            let expr = Expression::ArrayLit { elements };
            let output = gen.generate_expression_string(&expr);

            // Array syntax should be preserved (brackets with comma-separated elements)
            prop_assert!(output.starts_with('['));
            prop_assert!(output.ends_with(']'));
        }
    }

    // Property 7: Variable declarations translate correctly
    // Validates: Requirements 35.7, 35.8, 35.9
    proptest! {
        #[test]
        fn prop_let_translates_correctly(name in arb_ident(), mutable in any::<bool>()) {
            let mut gen = CodeGenerator::new(TargetLanguage::Rust);
            let stmt = Statement::Let {
                name: name.clone(),
                ty: Some(Type::Primitive(PrimitiveType::I32)),
                init: Some(Expression::Literal(Literal::Int(42))),
                mutable,
            };
            let func = Function {
                visibility: Visibility::Public,
                name: Ident::new("test"),
                params: vec![],
                return_type: None,
                body: Block::new(vec![stmt]),
                doc_comments: vec![],
            attributes: vec![],
            };
            let file = File {
                items: vec![Item::Function(func)],
                doc_comments: vec![],
            };
            let output = gen.generate(&file);

            // Should contain "let" keyword
            prop_assert!(output.contains("let "));
            // Should contain variable name
            prop_assert!(output.contains(&name.name));
            // If mutable, should contain "mut"
            if mutable {
                prop_assert!(output.contains("let mut "));
            }
        }

        #[test]
        fn prop_var_translates_to_let_mut(name in arb_ident()) {
            let mut gen = CodeGenerator::new(TargetLanguage::Rust);
            let stmt = Statement::Var {
                name: name.clone(),
                ty: Some(Type::Primitive(PrimitiveType::I32)),
                init: Some(Expression::Literal(Literal::Int(42))),
            };
            let func = Function {
                visibility: Visibility::Public,
                name: Ident::new("test"),
                params: vec![],
                return_type: None,
                body: Block::new(vec![stmt]),
                doc_comments: vec![],
            attributes: vec![],
            };
            let file = File {
                items: vec![Item::Function(func)],
                doc_comments: vec![],
            };
            let output = gen.generate(&file);

            // var should translate to "let mut"
            prop_assert!(output.contains("let mut "));
            prop_assert!(output.contains(&name.name));
        }
    }

    // Property 8: Reference syntax translates correctly
    // Validates: Requirements 36.10, 36.11
    proptest! {
        #[test]
        fn prop_reference_syntax_translates(mutable in any::<bool>()) {
            let gen = CodeGenerator::new(TargetLanguage::Rust);
            let ref_type = Type::Reference {
                ty: Box::new(Type::Primitive(PrimitiveType::I32)),
                mutable,
            };
            let output = gen.generate_type_string(&ref_type);

            // Should start with &
            prop_assert!(output.starts_with('&'));
            // If mutable, should contain "mut"
            if mutable {
                prop_assert!(output.contains("&mut "));
            } else {
                prop_assert!(!output.contains("mut"));
            }
            // Should end with the inner type
            prop_assert!(output.contains("i32"));
        }
    }

    // Property 23: Label syntax translates correctly
    // Validates: Requirements 6.13, 6.14, 6.15
    proptest! {
        #[test]
        fn prop_labeled_break_translates(label in arb_ident()) {
            let mut gen = CodeGenerator::new(TargetLanguage::Rust);
            let stmt = Statement::Break(Some(label.clone()));
            let func = Function {
                visibility: Visibility::Public,
                name: Ident::new("test"),
                params: vec![],
                return_type: None,
                body: Block::new(vec![stmt]),
                doc_comments: vec![],
            attributes: vec![],
            };
            let file = File {
                items: vec![Item::Function(func)],
                doc_comments: vec![],
            };
            let output = gen.generate(&file);

            // Should translate .label to 'label
            prop_assert!(output.contains("break '"));
            prop_assert!(output.contains(&label.name));
        }

        #[test]
        fn prop_labeled_continue_translates(label in arb_ident()) {
            let mut gen = CodeGenerator::new(TargetLanguage::Rust);
            let stmt = Statement::Continue(Some(label.clone()));
            let func = Function {
                visibility: Visibility::Public,
                name: Ident::new("test"),
                params: vec![],
                return_type: None,
                body: Block::new(vec![stmt]),
                doc_comments: vec![],
            attributes: vec![],
            };
            let file = File {
                items: vec![Item::Function(func)],
                doc_comments: vec![],
            };
            let output = gen.generate(&file);

            // Should translate .label to 'label
            prop_assert!(output.contains("continue '"));
            prop_assert!(output.contains(&label.name));
        }

        #[test]
        fn prop_labeled_loop_translates(label in arb_ident()) {
            let mut gen = CodeGenerator::new(TargetLanguage::Rust);
            let stmt = Statement::While {
                label: Some(label.clone()),
                condition: Expression::Literal(Literal::Bool(true)),
                body: Block::empty(),
            };
            let func = Function {
                visibility: Visibility::Public,
                name: Ident::new("test"),
                params: vec![],
                return_type: None,
                body: Block::new(vec![stmt]),
                doc_comments: vec![],
            attributes: vec![],
            };
            let file = File {
                items: vec![Item::Function(func)],
                doc_comments: vec![],
            };
            let output = gen.generate(&file);

            // Should translate .label: to 'label:
            let expected = format!("'{}:", label.name);
            prop_assert!(output.contains(&expected));
        }
    }

    // Additional property tests for struct and enum generation
    proptest! {
        #[test]
        fn prop_struct_generation_is_valid(struct_def in arb_simple_struct()) {
            let mut gen = CodeGenerator::new(TargetLanguage::Rust);
            let file = File {
                items: vec![Item::Struct(struct_def)],
                doc_comments: vec![],
            };
            let output = gen.generate(&file);

            // Should be parseable by syn
            let parse_result = syn::parse_file(&output);
            prop_assert!(parse_result.is_ok(), "Generated struct should be syntactically valid: {}", output);
        }

        #[test]
        fn prop_enum_generation_is_valid(enum_def in arb_simple_enum()) {
            let mut gen = CodeGenerator::new(TargetLanguage::Rust);
            let file = File {
                items: vec![Item::Enum(enum_def)],
                doc_comments: vec![],
            };
            let output = gen.generate(&file);

            // Should be parseable by syn
            let parse_result = syn::parse_file(&output);
            prop_assert!(parse_result.is_ok(), "Generated enum should be syntactically valid: {}", output);
        }
    }

    // Generator for macro definitions
    fn arb_macro_name() -> impl Strategy<Value = String> {
        "[A-Z][A-Z0-9_]{0,10}".prop_map(|s| format!("__{}__", s))
    }

    fn arb_simple_macro_definition() -> impl Strategy<Value = MacroDefinition> {
        use crate::error::{Position, Span};
        use crate::lexer::{Token, TokenKind};

        (arb_macro_name(), prop::collection::vec(arb_ident(), 0..3)).prop_map(|(name, params)| {
            // Create a simple macro body with some tokens
            let body = vec![Token {
                kind: TokenKind::IntLiteral("100".to_string()),
                span: Span {
                    start: Position { line: 1, column: 1 },
                    end: Position { line: 1, column: 4 },
                },
                text: "100".to_string(),
            }];

            MacroDefinition {
                name: Ident::new(name),
                params,
                body,
            }
        })
    }

    // Property 22: #define macros translate to macro_rules!
    // Validates: Requirements 24.7, 24.8, 24.9
    proptest! {
        #[test]
        fn prop_define_translates_to_macro_rules(macro_def in arb_simple_macro_definition()) {
            let mut gen = CodeGenerator::new(TargetLanguage::Rust);
            let file = File {
                items: vec![Item::MacroDefinition(macro_def.clone())],
                doc_comments: vec![],
            };
            let output = gen.generate(&file);

            // Should contain macro_rules!
            prop_assert!(output.contains("macro_rules!"),
                "Generated code should contain macro_rules!: {}", output);

            // Should remove double-underscore prefix and suffix from macro name
            let rust_name = macro_def.name.name
                .trim_start_matches("__")
                .trim_end_matches("__")
                .to_lowercase();
            prop_assert!(output.contains(&rust_name),
                "Generated code should contain macro name '{}': {}", rust_name, output);

            // If macro has parameters, should translate to pattern variables
            if !macro_def.params.is_empty() {
                for param in &macro_def.params {
                    let pattern_var = format!("${}:expr", param.name);
                    prop_assert!(output.contains(&pattern_var),
                        "Generated code should contain pattern variable '{}': {}", pattern_var, output);
                }
            }

            // Generated code should be syntactically valid Rust
            let parse_result = syn::parse_file(&output);
            prop_assert!(parse_result.is_ok(),
                "Generated macro should be syntactically valid: {}", output);
        }
    }

    // Property 12: Typedef translates to type alias
    // Validates: Requirements 31.9
    proptest! {
        #[test]
        fn prop_typedef_translates_to_type_alias(name in arb_ident()) {
            let mut gen = CodeGenerator::new(TargetLanguage::Rust);
            let typedef = Typedef {
                visibility: Visibility::Public,
                name: Ident::new(name.name.clone()),
                target: Type::Primitive(PrimitiveType::Int),
                doc_comments: vec![],
            };
            let file = File {
                items: vec![Item::Typedef(typedef)],
                doc_comments: vec![],
            };
            let output = gen.generate(&file);

            // Should contain 'type' keyword
            prop_assert!(output.contains("type"),
                "Generated code should contain 'type' keyword: {}", output);

            // Should contain the typedef name
            prop_assert!(output.contains(&name.name),
                "Generated code should contain typedef name '{}': {}", name.name, output);

            // Should contain '=' for type alias
            prop_assert!(output.contains("="),
                "Generated code should contain '=' for type alias: {}", output);

            // Should be syntactically valid Rust
            let parse_result = syn::parse_file(&output);
            prop_assert!(parse_result.is_ok(),
                "Generated typedef should be syntactically valid: {}", output);
        }
    }
}
