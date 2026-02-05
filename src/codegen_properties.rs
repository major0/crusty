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
            .prop_filter("Must not be a Rust or Crusty keyword", |s| {
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
                        // Crusty type keywords
                        | "int"
                        | "i32"
                        | "i64"
                        | "u32"
                        | "u64"
                        | "float"
                        | "f32"
                        | "f64"
                        | "bool"
                        | "char"
                        | "void"
                        // Other Crusty keywords
                        | "typedef"
                        | "switch"
                        | "case"
                        | "default"
                        | "auto"
                        | "namespace"
                        | "define"
                )
            })
            .prop_map(Ident::new)
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

            let delimiter = if params.is_empty() {
                MacroDelimiter::None
            } else {
                MacroDelimiter::Parens
            };

            MacroDefinition {
                name: Ident::new(name),
                params,
                body,
                delimiter,
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

    // Property 9: Type casts translate to 'as' operator
    // Validates: Requirements 27.5
    proptest! {
        #[test]
        fn prop_type_cast_translates_to_as(expr in arb_simple_expression(), target_ty in arb_primitive_type()) {
            let gen = CodeGenerator::new(TargetLanguage::Rust);
            let cast_expr = Expression::Cast {
                expr: Box::new(expr),
                ty: target_ty,
            };
            let output = gen.generate_expression_string(&cast_expr);

            // Should contain 'as' keyword
            prop_assert!(output.contains(" as "),
                "Generated code should contain ' as ' operator: {}", output);
        }
    }

    // Property 10: Sizeof translates to std::mem functions
    // Validates: Requirements 28.6
    proptest! {
        #[test]
        fn prop_sizeof_translates_to_std_mem(ty in arb_primitive_type()) {
            let gen = CodeGenerator::new(TargetLanguage::Rust);
            let sizeof_expr = Expression::Sizeof { ty };
            let output = gen.generate_expression_string(&sizeof_expr);

            // Should contain std::mem::size_of
            prop_assert!(output.contains("std::mem::size_of"),
                "Generated code should contain 'std::mem::size_of': {}", output);
        }
    }

    // Property 11: Increment/decrement operators translate with correct semantics
    // Validates: Requirements 29.10, 29.11
    proptest! {
        #[test]
        fn prop_increment_translates_correctly(var in arb_ident()) {
            let gen = CodeGenerator::new(TargetLanguage::Rust);
            // Pre-increment: ++x translates to { x += 1; x }
            let pre_inc = Expression::Unary {
                op: UnaryOp::PreInc,
                expr: Box::new(Expression::Ident(var.clone())),
            };
            let output = gen.generate_expression_string(&pre_inc);

            // Should contain += 1
            prop_assert!(output.contains("+=") && output.contains("1"),
                "Pre-increment should translate to += 1: {}", output);
        }

        #[test]
        fn prop_decrement_translates_correctly(var in arb_ident()) {
            let gen = CodeGenerator::new(TargetLanguage::Rust);
            // Pre-decrement: --x translates to { x -= 1; x }
            let pre_dec = Expression::Unary {
                op: UnaryOp::PreDec,
                expr: Box::new(Expression::Ident(var.clone())),
            };
            let output = gen.generate_expression_string(&pre_dec);

            // Should contain -= 1
            prop_assert!(output.contains("-=") && output.contains("1"),
                "Pre-decrement should translate to -= 1: {}", output);
        }
    }

    // Property 13: C-style enums translate to Rust enums with discriminants
    // Validates: Requirements 32.8
    proptest! {
        #[test]
        fn prop_enum_with_discriminants(enum_def in arb_simple_enum()) {
            let mut gen = CodeGenerator::new(TargetLanguage::Rust);
            let file = File {
                items: vec![Item::Enum(enum_def.clone())],
                doc_comments: vec![],
            };
            let output = gen.generate(&file);

            // Should contain enum keyword
            prop_assert!(output.contains("enum "),
                "Generated code should contain 'enum' keyword: {}", output);

            // Should contain variant names
            for variant in &enum_def.variants {
                prop_assert!(output.contains(&variant.name.name),
                    "Generated code should contain variant '{}': {}", variant.name.name, output);

                // If variant has explicit value, should contain = value
                if let Some(value) = variant.value {
                    let discriminant = format!("= {}", value);
                    prop_assert!(output.contains(&discriminant),
                        "Generated code should contain discriminant '{}': {}", discriminant, output);
                }
            }
        }
    }

    // Property 14: NULL translates to Option types
    // Validates: Requirements 34.4, 34.5
    proptest! {
        #[test]
        fn prop_null_translates_to_option_none(_dummy in 0..1u8) {
            let gen = CodeGenerator::new(TargetLanguage::Rust);
            let null_expr = Expression::Literal(Literal::Null);
            let output = gen.generate_expression_string(&null_expr);

            // Should translate to Option::None
            prop_assert!(output.contains("Option::None") || output.contains("None"),
                "NULL should translate to Option::None: {}", output);
        }
    }

    // Property 15: Struct initializers translate to Rust struct literals
    // Validates: Requirements 39.6
    proptest! {
        #[test]
        fn prop_struct_init_translates(struct_name in arb_ident(), field_name in arb_ident()) {
            let gen = CodeGenerator::new(TargetLanguage::Rust);
            let struct_init = Expression::StructInit {
                ty: Type::Ident(struct_name.clone()),
                fields: vec![(field_name.clone(), Expression::Literal(Literal::Int(42)))],
            };
            let output = gen.generate_expression_string(&struct_init);

            // Should contain struct name
            prop_assert!(output.contains(&struct_name.name),
                "Generated code should contain struct name '{}': {}", struct_name.name, output);

            // Should contain field name
            prop_assert!(output.contains(&field_name.name),
                "Generated code should contain field name '{}': {}", field_name.name, output);

            // Should contain braces
            prop_assert!(output.contains('{') && output.contains('}'),
                "Generated code should contain braces: {}", output);
        }
    }

    // Property 16: Struct methods translate to impl blocks
    // Validates: Requirements 21.9
    proptest! {
        #[test]
        fn prop_struct_methods_translate_to_impl(struct_name in arb_ident(), method_name in arb_ident()) {
            let mut gen = CodeGenerator::new(TargetLanguage::Rust);
            let method = Function {
                visibility: Visibility::Public,
                name: method_name.clone(),
                params: vec![Param {
                    name: Ident::new("self"),
                    ty: Type::Reference {
                        ty: Box::new(Type::Ident(Ident::new("Self"))),
                        mutable: false,
                    },
                }],
                return_type: None,
                body: Block::empty(),
                doc_comments: vec![],
                attributes: vec![],
            };
            let struct_def = Struct {
                visibility: Visibility::Public,
                name: struct_name.clone(),
                fields: vec![],
                methods: vec![method],
                doc_comments: vec![],
                attributes: vec![],
            };
            let file = File {
                items: vec![Item::Struct(struct_def)],
                doc_comments: vec![],
            };
            let output = gen.generate(&file);

            // Should contain impl keyword
            prop_assert!(output.contains("impl "),
                "Generated code should contain 'impl' keyword: {}", output);

            // Should contain struct name in impl block
            let impl_line = format!("impl {}", struct_name.name);
            prop_assert!(output.contains(&impl_line),
                "Generated code should contain 'impl {}': {}", struct_name.name, output);

            // Should contain method name
            prop_assert!(output.contains(&method_name.name),
                "Generated code should contain method name '{}': {}", method_name.name, output);
        }
    }

    // Property 18: For loops translate appropriately
    // Validates: Requirements 38.4, 38.5, 38.7
    proptest! {
        #[test]
        fn prop_for_in_loop_translates(var in arb_ident(), iter_var in arb_ident()) {
            let mut gen = CodeGenerator::new(TargetLanguage::Rust);
            let for_in = Statement::ForIn {
                label: None,
                var: var.clone(),
                iter: Expression::Ident(iter_var.clone()),
                body: Block::empty(),
            };
            let func = Function {
                visibility: Visibility::Public,
                name: Ident::new("test"),
                params: vec![],
                return_type: None,
                body: Block::new(vec![for_in]),
                doc_comments: vec![],
                attributes: vec![],
            };
            let file = File {
                items: vec![Item::Function(func)],
                doc_comments: vec![],
            };
            let output = gen.generate(&file);

            // Should contain 'for' keyword
            prop_assert!(output.contains("for "),
                "Generated code should contain 'for' keyword: {}", output);

            // Should contain 'in' keyword
            prop_assert!(output.contains(" in "),
                "Generated code should contain 'in' keyword: {}", output);

            // Should contain loop variable
            prop_assert!(output.contains(&var.name),
                "Generated code should contain loop variable '{}': {}", var.name, output);
        }

        #[test]
        fn prop_c_style_for_loop_translates(var in arb_ident()) {
            let mut gen = CodeGenerator::new(TargetLanguage::Rust);
            let c_for = Statement::For {
                label: None,
                init: Box::new(Statement::Let {
                    name: var.clone(),
                    ty: Some(Type::Primitive(PrimitiveType::I32)),
                    init: Some(Expression::Literal(Literal::Int(0))),
                    mutable: true,
                }),
                condition: Expression::Binary {
                    op: BinaryOp::Lt,
                    left: Box::new(Expression::Ident(var.clone())),
                    right: Box::new(Expression::Literal(Literal::Int(10))),
                },
                increment: Expression::Unary {
                    op: UnaryOp::PostInc,
                    expr: Box::new(Expression::Ident(var.clone())),
                },
                body: Block::empty(),
            };
            let func = Function {
                visibility: Visibility::Public,
                name: Ident::new("test"),
                params: vec![],
                return_type: None,
                body: Block::new(vec![c_for]),
                doc_comments: vec![],
                attributes: vec![],
            };
            let file = File {
                items: vec![Item::Function(func)],
                doc_comments: vec![],
            };
            let output = gen.generate(&file);

            // C-style for loops should translate to while loops in Rust
            // Should contain loop variable initialization
            prop_assert!(output.contains(&var.name),
                "Generated code should contain loop variable '{}': {}", var.name, output);
        }
    }

    // Property 19: Switch statements translate to match expressions
    // Validates: Requirements 45.7
    proptest! {
        #[test]
        fn prop_switch_translates_to_match(var in arb_ident()) {
            let mut gen = CodeGenerator::new(TargetLanguage::Rust);
            let switch = Statement::Switch {
                expr: Expression::Ident(var.clone()),
                cases: vec![
                    SwitchCase {
                        values: vec![Expression::Literal(Literal::Int(1))],
                        body: Block::empty(),
                    },
                    SwitchCase {
                        values: vec![Expression::Literal(Literal::Int(2))],
                        body: Block::empty(),
                    },
                ],
                default: Some(Block::empty()),
            };
            let func = Function {
                visibility: Visibility::Public,
                name: Ident::new("test"),
                params: vec![],
                return_type: None,
                body: Block::new(vec![switch]),
                doc_comments: vec![],
                attributes: vec![],
            };
            let file = File {
                items: vec![Item::Function(func)],
                doc_comments: vec![],
            };
            let output = gen.generate(&file);

            // Should contain 'match' keyword
            prop_assert!(output.contains("match "),
                "Switch should translate to 'match': {}", output);

            // Should contain the switch expression variable
            prop_assert!(output.contains(&var.name),
                "Generated code should contain switch variable '{}': {}", var.name, output);

            // Should contain wildcard pattern for default case
            prop_assert!(output.contains("_"),
                "Generated code should contain wildcard pattern for default: {}", output);
        }
    }

    // Property 20: Error handling syntax translates correctly
    // Validates: Requirements 46.8, 46.9, 46.10
    proptest! {
        #[test]
        fn prop_fallible_type_translates(inner_ty in arb_primitive_type()) {
            let gen = CodeGenerator::new(TargetLanguage::Rust);
            let fallible_ty = Type::Fallible {
                ty: Box::new(inner_ty),
            };
            let output = gen.generate_type_string(&fallible_ty);

            // Type? should translate to Result<Type, E> or similar
            // The exact translation depends on implementation, but should contain Result
            prop_assert!(output.contains("Result") || output.contains("Option"),
                "Fallible type should translate to Result or Option: {}", output);
        }

        #[test]
        fn prop_error_propagation_translates(expr in arb_simple_expression()) {
            let gen = CodeGenerator::new(TargetLanguage::Rust);
            let error_prop = Expression::ErrorProp {
                expr: Box::new(expr),
            };
            let output = gen.generate_expression_string(&error_prop);

            // expr? should pass through to Rust ? operator
            prop_assert!(output.contains('?'),
                "Error propagation should contain '?' operator: {}", output);
        }
    }

    // Property 24: Explicit generic parameters translate correctly
    // Validates: Requirements 38.18, 38.19, 38.20, 38.21
    proptest! {
        #[test]
        fn prop_explicit_generic_call_translates(
            type_name in arb_ident(),
            method_name in arb_ident(),
            generic_ty in arb_primitive_type()
        ) {
            let gen = CodeGenerator::new(TargetLanguage::Rust);
            let explicit_generic = Expression::ExplicitGenericCall {
                ty: Type::Ident(type_name.clone()),
                generics: vec![generic_ty],
                method: method_name.clone(),
                args: vec![],
            };
            let output = gen.generate_expression_string(&explicit_generic);

            // Should contain type name
            prop_assert!(output.contains(&type_name.name),
                "Generated code should contain type name '{}': {}", type_name.name, output);

            // Should contain method name
            prop_assert!(output.contains(&method_name.name),
                "Generated code should contain method name '{}': {}", method_name.name, output);

            // Should contain turbofish syntax ::< >
            prop_assert!(output.contains("::<"),
                "Explicit generic parameters should use turbofish syntax: {}", output);
        }
    }

    // Property 35: Nested functions translate to Rust closures
    // Validates: Requirements 59.11, 59.12, 59.13
    proptest! {
        #[test]
        fn prop_nested_function_translates_to_closure(
            nested_name in arb_ident(),
            param_name in arb_ident(),
            param_type in arb_primitive_type(),
            return_type in arb_primitive_type()
        ) {
            use crate::semantic::SemanticAnalyzer;

            // Create a nested function with parameters and return type
            let nested_func = Statement::NestedFunction {
                name: nested_name.clone(),
                params: vec![Param {
                    name: param_name.clone(),
                    ty: param_type.clone(),
                }],
                return_type: Some(return_type.clone()),
                body: Block {
                    statements: vec![Statement::Return(Some(Expression::Ident(param_name.clone())))],
                },
            };

            // Create an outer function containing the nested function
            let outer_func = Function {
                visibility: Visibility::Public,
                name: Ident::new("outer"),
                params: vec![],
                return_type: Some(Type::Primitive(PrimitiveType::Void)),
                body: Block {
                    statements: vec![nested_func],
                },
                doc_comments: vec![],
                attributes: vec![],
            };

            let file = File {
                items: vec![Item::Function(outer_func)],
                doc_comments: vec![],
            };

            // Run semantic analysis to get captures
            let mut analyzer = SemanticAnalyzer::new();
            let _ = analyzer.analyze(&file);

            // Generate code with captures
            let mut gen_with_captures = CodeGenerator::new(TargetLanguage::Rust);
            gen_with_captures.set_captures(analyzer.get_all_captures().clone());
            let output = gen_with_captures.generate(&file);

            // Verify nested function translates to closure
            // Should contain: let <name> = |<params>| -> <return_type>
            prop_assert!(output.contains(&format!("let {} = |", nested_name.name)),
                "Nested function should translate to closure binding: {}", output);

            // Should contain parameter name
            prop_assert!(output.contains(&param_name.name),
                "Closure should contain parameter name '{}': {}", param_name.name, output);

            // Should contain closure syntax with pipes
            prop_assert!(output.contains('|'),
                "Closure should use pipe syntax: {}", output);
        }

        #[test]
        fn prop_nested_function_with_immutable_capture(
            nested_name in arb_ident(),
            capture_var in arb_ident(),
            return_type in arb_primitive_type()
        ) {
            use crate::semantic::SemanticAnalyzer;

            // Create a nested function that captures an immutable variable
            let nested_func = Statement::NestedFunction {
                name: nested_name.clone(),
                params: vec![],
                return_type: Some(return_type.clone()),
                body: Block {
                    statements: vec![Statement::Return(Some(Expression::Ident(capture_var.clone())))],
                },
            };

            // Create an outer function with a variable and nested function
            let outer_func = Function {
                visibility: Visibility::Public,
                name: Ident::new("outer"),
                params: vec![],
                return_type: Some(Type::Primitive(PrimitiveType::Void)),
                body: Block {
                    statements: vec![
                        Statement::Let {
                            name: capture_var.clone(),
                            ty: Some(return_type.clone()),
                            init: Some(Expression::Literal(Literal::Int(42))),
                            mutable: false,
                        },
                        nested_func,
                    ],
                },
                doc_comments: vec![],
                attributes: vec![],
            };

            let file = File {
                items: vec![Item::Function(outer_func)],
                doc_comments: vec![],
            };

            // Run semantic analysis to get captures
            let mut analyzer = SemanticAnalyzer::new();
            let _ = analyzer.analyze(&file);

            // Generate code with captures
            let mut gen_with_captures = CodeGenerator::new(TargetLanguage::Rust);
            gen_with_captures.set_captures(analyzer.get_all_captures().clone());
            let output = gen_with_captures.generate(&file);

            // Verify nested function translates to closure (Fn trait for immutable capture)
            prop_assert!(output.contains(&format!("let {} = ||", nested_name.name)),
                "Nested function with immutable capture should translate to closure: {}", output);

            // Should reference the captured variable
            prop_assert!(output.contains(&capture_var.name),
                "Closure should reference captured variable '{}': {}", capture_var.name, output);
        }

        #[test]
        fn prop_nested_function_with_mutable_capture(
            nested_name in arb_ident(),
            capture_var in arb_ident()
        ) {
            use crate::semantic::SemanticAnalyzer;

            // Create a nested function that mutates a captured variable
            let nested_func = Statement::NestedFunction {
                name: nested_name.clone(),
                params: vec![],
                return_type: Some(Type::Primitive(PrimitiveType::Void)),
                body: Block {
                    statements: vec![
                        Statement::Expr(Expression::Binary {
                            left: Box::new(Expression::Ident(capture_var.clone())),
                            op: BinaryOp::Assign,
                            right: Box::new(Expression::Binary {
                                left: Box::new(Expression::Ident(capture_var.clone())),
                                op: BinaryOp::Add,
                                right: Box::new(Expression::Literal(Literal::Int(1))),
                            }),
                        }),
                    ],
                },
            };

            // Create an outer function with a mutable variable and nested function
            let outer_func = Function {
                visibility: Visibility::Public,
                name: Ident::new("outer"),
                params: vec![],
                return_type: Some(Type::Primitive(PrimitiveType::Void)),
                body: Block {
                    statements: vec![
                        Statement::Var {
                            name: capture_var.clone(),
                            ty: Some(Type::Primitive(PrimitiveType::I32)),
                            init: Some(Expression::Literal(Literal::Int(0))),
                        },
                        nested_func,
                    ],
                },
                doc_comments: vec![],
                attributes: vec![],
            };

            let file = File {
                items: vec![Item::Function(outer_func)],
                doc_comments: vec![],
            };

            // Run semantic analysis to get captures
            let mut analyzer = SemanticAnalyzer::new();
            let _ = analyzer.analyze(&file);

            // Generate code with captures
            let mut gen_with_captures = CodeGenerator::new(TargetLanguage::Rust);
            gen_with_captures.set_captures(analyzer.get_all_captures().clone());
            let output = gen_with_captures.generate(&file);

            // Verify nested function translates to mutable closure (FnMut trait for mutable capture)
            prop_assert!(output.contains(&format!("let mut {} = ||", nested_name.name)),
                "Nested function with mutable capture should translate to mut closure: {}", output);

            // Should reference the captured variable
            prop_assert!(output.contains(&capture_var.name),
                "Closure should reference captured variable '{}': {}", capture_var.name, output);
        }
    }
}
