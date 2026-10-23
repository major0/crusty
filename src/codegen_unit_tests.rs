// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Unit tests for the code generator, extracted from the inline `#[cfg(test)]`
//! module in `src/codegen.rs` to match the project's sibling-test-file
//! convention. Private fields and methods are reached through the generator's
//! `#[cfg(test)]` accessors (`target()`, `indent_level()`, `indent_for_test()`,
//! `dedent_for_test()`), so no production visibility is widened.

#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::codegen::{CodeGenerator, TargetLanguage};

    #[test]
    fn test_create_code_generator() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        assert_eq!(gen.target(), TargetLanguage::Rust);
        assert_eq!(gen.indent_level(), 0);
    }

    #[test]
    fn test_generate_empty_file() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let file = File {
            items: vec![],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert_eq!(output, "");
    }

    #[test]
    fn test_generate_file_with_doc_comments() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let file = File {
            items: vec![],
            doc_comments: vec!["This is a test file".to_string()],
        };
        let output = gen.generate(&file);
        assert!(output.contains("//! This is a test file"));
    }

    #[test]
    fn test_indentation() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        assert_eq!(gen.indent_level(), 0);

        gen.indent_for_test();
        assert_eq!(gen.indent_level(), 1);

        gen.indent_for_test();
        assert_eq!(gen.indent_level(), 2);

        gen.dedent_for_test();
        assert_eq!(gen.indent_level(), 1);

        gen.dedent_for_test();
        assert_eq!(gen.indent_level(), 0);

        // Should not go below 0
        gen.dedent_for_test();
        assert_eq!(gen.indent_level(), 0);
    }

    #[test]
    fn test_generate_simple_function() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("main"),
            params: vec![],
            return_type: None,
            body: Some(Block::empty()),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("pub fn main()"));
        assert!(output.contains("{"));
        assert!(output.contains("}"));
    }

    #[test]
    fn test_generate_function_with_void_return() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("foo"),
            params: vec![],
            return_type: Some(Type::Primitive(PrimitiveType::Void)),
            body: Some(Block::empty()),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("pub fn foo()"));
        // Should not have -> () for void
        assert!(!output.contains("-> ()"));
    }

    #[test]
    fn test_generate_function_with_return_type() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("add"),
            params: vec![
                Param {
                    name: Ident::new("a"),
                    ty: Type::Primitive(PrimitiveType::I32),
                },
                Param {
                    name: Ident::new("b"),
                    ty: Type::Primitive(PrimitiveType::I32),
                },
            ],
            return_type: Some(Type::Primitive(PrimitiveType::I32)),
            body: Some(Block::empty()),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("pub fn add(a: i32, b: i32) -> i32"));
    }

    #[test]
    fn test_generate_static_function() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let func = Function {
            visibility: Visibility::Private,
            name: Ident::new("helper"),
            params: vec![],
            return_type: None,
            body: Some(Block::empty()),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("fn helper()"));
        assert!(!output.contains("pub fn helper()"));
    }

    #[test]
    fn test_generate_let_statement() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let stmt = Statement::Let {
            name: Ident::new("x"),
            ty: Some(Type::Primitive(PrimitiveType::I32)),
            init: Some(Expression::Literal(Literal::Int(42))),
            mutable: false,
        };
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("let x: i32 = 42;"));
    }

    #[test]
    fn test_generate_var_statement() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let stmt = Statement::Var {
            name: Ident::new("x"),
            ty: Some(Type::Primitive(PrimitiveType::I32)),
            init: Some(Expression::Literal(Literal::Int(42))),
        };
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("let mut x: i32 = 42;"));
    }

    #[test]
    fn test_generate_if_statement() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let stmt = Statement::If {
            condition: Expression::Literal(Literal::Bool(true)),
            then_block: Block::empty(),
            else_block: None,
        };
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("if true"));
    }

    #[test]
    fn test_generate_while_statement() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let stmt = Statement::While {
            label: None,
            condition: Expression::Literal(Literal::Bool(true)),
            body: Block::empty(),
        };
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("while true"));
    }

    #[test]
    fn test_generate_labeled_while() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let stmt = Statement::While {
            label: Some(Ident::new("outer")),
            condition: Expression::Literal(Literal::Bool(true)),
            body: Block::empty(),
        };
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("'outer: while true"));
    }

    #[test]
    fn test_generate_break_with_label() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let stmt = Statement::Break(Some(Ident::new("outer")));
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("break 'outer;"));
    }

    #[test]
    fn test_generate_continue_with_label() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let stmt = Statement::Continue(Some(Ident::new("outer")));
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("continue 'outer;"));
    }

    #[test]
    fn test_generate_binary_expression() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expression::Literal(Literal::Int(1))),
            right: Box::new(Expression::Literal(Literal::Int(2))),
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "(1 + 2)");
    }

    #[test]
    fn test_generate_cast_expression() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::Cast {
            expr: Box::new(Expression::Literal(Literal::Int(42))),
            ty: Type::Primitive(PrimitiveType::F64),
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "(42 as f64)");
    }

    #[test]
    fn test_generate_sizeof_expression() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::Sizeof {
            ty: Type::Primitive(PrimitiveType::I32),
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "std::mem::size_of::<i32>()");
    }

    #[test]
    fn test_generate_type_scoped_call() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::TypeScopedCall {
            ty: Type::Ident(Ident::new("Vec")),
            method: Ident::new("new"),
            args: vec![],
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "Vec::new()");
    }

    #[test]
    fn test_generate_explicit_generic_call() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::ExplicitGenericCall {
            ty: Type::Ident(Ident::new("Vec")),
            generics: vec![Type::Primitive(PrimitiveType::I32)],
            method: Ident::new("new"),
            args: vec![],
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "Vec::<i32>::new()");
    }

    #[test]
    fn test_generate_struct() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let struct_def = Struct {
            visibility: Visibility::Public,
            name: Ident::new("Point"),
            fields: vec![
                Field {
                    visibility: Visibility::Public,
                    name: Ident::new("x"),
                    ty: Type::Primitive(PrimitiveType::I32),
                    doc_comments: vec![],
                    attributes: vec![],
                },
                Field {
                    visibility: Visibility::Public,
                    name: Ident::new("y"),
                    ty: Type::Primitive(PrimitiveType::I32),
                    doc_comments: vec![],
                    attributes: vec![],
                },
            ],
            methods: vec![],
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Struct(struct_def)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("pub struct Point"));
        assert!(output.contains("pub x: i32,"));
        assert!(output.contains("pub y: i32,"));
    }

    #[test]
    fn test_generate_struct_with_methods() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let method = Function {
            visibility: Visibility::Public,
            name: Ident::new("new"),
            params: vec![],
            return_type: Some(Type::Ident(Ident::new("Self"))),
            body: Some(Block::empty()),
            doc_comments: vec![],
            attributes: vec![],
        };
        let struct_def = Struct {
            visibility: Visibility::Public,
            name: Ident::new("Point"),
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
        assert!(output.contains("pub struct Point"));
        assert!(output.contains("impl Point"));
        assert!(output.contains("pub fn new() -> Self"));
    }

    #[test]
    fn test_generate_struct_with_self_parameter() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let method = Function {
            visibility: Visibility::Public,
            name: Ident::new("get_x"),
            params: vec![Param {
                name: Ident::new("self"),
                ty: Type::Reference {
                    ty: Box::new(Type::Ident(Ident::new("Self"))),
                    mutable: false,
                },
            }],
            return_type: Some(Type::Primitive(PrimitiveType::I32)),
            body: Some(Block::empty()),
            doc_comments: vec![],
            attributes: vec![],
        };
        let struct_def = Struct {
            visibility: Visibility::Public,
            name: Ident::new("Point"),
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
        assert!(output.contains("pub struct Point"));
        assert!(output.contains("impl Point"));
        assert!(output.contains("pub fn get_x(&self) -> i32"));
    }

    #[test]
    fn test_generate_struct_with_mut_self_parameter() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let method = Function {
            visibility: Visibility::Public,
            name: Ident::new("set_x"),
            params: vec![
                Param {
                    name: Ident::new("self"),
                    ty: Type::Reference {
                        ty: Box::new(Type::Ident(Ident::new("Self"))),
                        mutable: true,
                    },
                },
                Param {
                    name: Ident::new("new_x"),
                    ty: Type::Primitive(PrimitiveType::I32),
                },
            ],
            return_type: None,
            body: Some(Block::empty()),
            doc_comments: vec![],
            attributes: vec![],
        };
        let struct_def = Struct {
            visibility: Visibility::Public,
            name: Ident::new("Point"),
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
        assert!(output.contains("pub struct Point"));
        assert!(output.contains("impl Point"));
        assert!(output.contains("pub fn set_x(&mut self, new_x: i32)"));
    }

    #[test]
    fn test_generate_struct_with_static_method() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let method = Function {
            visibility: Visibility::Private,
            name: Ident::new("origin"),
            params: vec![],
            return_type: Some(Type::Ident(Ident::new("Self"))),
            body: Some(Block::empty()),
            doc_comments: vec![],
            attributes: vec![],
        };
        let struct_def = Struct {
            visibility: Visibility::Public,
            name: Ident::new("Point"),
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
        assert!(output.contains("pub struct Point"));
        assert!(output.contains("impl Point"));
        assert!(output.contains("fn origin() -> Self"));
        assert!(!output.contains("pub fn origin"));
    }

    #[test]
    fn test_generate_enum() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let enum_def = Enum {
            visibility: Visibility::Public,
            name: Ident::new("Color"),
            variants: vec![
                EnumVariant {
                    name: Ident::new("Red"),
                    value: Some(0),
                },
                EnumVariant {
                    name: Ident::new("Green"),
                    value: Some(1),
                },
                EnumVariant {
                    name: Ident::new("Blue"),
                    value: Some(2),
                },
            ],
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Enum(enum_def)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("pub enum Color"));
        assert!(output.contains("Red = 0,"));
        assert!(output.contains("Green = 1,"));
        assert!(output.contains("Blue = 2,"));
    }

    #[test]
    fn test_generate_primitive_types() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        assert_eq!(
            gen.generate_type_string(&Type::Primitive(PrimitiveType::Int)),
            "i32"
        );
        assert_eq!(
            gen.generate_type_string(&Type::Primitive(PrimitiveType::I32)),
            "i32"
        );
        assert_eq!(
            gen.generate_type_string(&Type::Primitive(PrimitiveType::I64)),
            "i64"
        );
        assert_eq!(
            gen.generate_type_string(&Type::Primitive(PrimitiveType::U32)),
            "u32"
        );
        assert_eq!(
            gen.generate_type_string(&Type::Primitive(PrimitiveType::U64)),
            "u64"
        );
        assert_eq!(
            gen.generate_type_string(&Type::Primitive(PrimitiveType::Float)),
            "f64"
        );
        assert_eq!(
            gen.generate_type_string(&Type::Primitive(PrimitiveType::F32)),
            "f32"
        );
        assert_eq!(
            gen.generate_type_string(&Type::Primitive(PrimitiveType::F64)),
            "f64"
        );
        assert_eq!(
            gen.generate_type_string(&Type::Primitive(PrimitiveType::Bool)),
            "bool"
        );
        assert_eq!(
            gen.generate_type_string(&Type::Primitive(PrimitiveType::Char)),
            "char"
        );
        assert_eq!(
            gen.generate_type_string(&Type::Primitive(PrimitiveType::Void)),
            "()"
        );
    }

    #[test]
    fn test_generate_reference_types() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let immutable_ref = Type::Reference {
            ty: Box::new(Type::Primitive(PrimitiveType::I32)),
            mutable: false,
        };
        assert_eq!(gen.generate_type_string(&immutable_ref), "&i32");

        let mutable_ref = Type::Reference {
            ty: Box::new(Type::Primitive(PrimitiveType::I32)),
            mutable: true,
        };
        assert_eq!(gen.generate_type_string(&mutable_ref), "&mut i32");
    }

    #[test]
    fn test_generate_array_type() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let array = Type::Array {
            ty: Box::new(Type::Primitive(PrimitiveType::I32)),
            size: Some(10),
        };
        assert_eq!(gen.generate_type_string(&array), "[i32; 10]");
    }

    #[test]
    fn test_generate_tuple_type() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let tuple = Type::Tuple {
            types: vec![
                Type::Primitive(PrimitiveType::I32),
                Type::Primitive(PrimitiveType::Bool),
            ],
        };
        assert_eq!(gen.generate_type_string(&tuple), "(i32, bool)");
    }

    #[test]
    fn test_generate_generic_type() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let generic = Type::Generic {
            base: Box::new(Type::Ident(Ident::new("Vec"))),
            args: vec![Type::Primitive(PrimitiveType::I32)],
        };
        assert_eq!(gen.generate_type_string(&generic), "Vec<i32>");
    }

    #[test]
    fn test_generate_array_literal() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::ArrayLit {
            elements: vec![
                Expression::Literal(Literal::Int(1)),
                Expression::Literal(Literal::Int(2)),
                Expression::Literal(Literal::Int(3)),
            ],
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "[1, 2, 3]");
    }

    #[test]
    fn test_generate_tuple_literal() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::TupleLit {
            elements: vec![
                Expression::Literal(Literal::Int(1)),
                Expression::Literal(Literal::Bool(true)),
            ],
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "(1, true)");
    }

    #[test]
    fn test_generate_range_expression() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);

        // 0..10
        let range = Expression::Range {
            start: Some(Box::new(Expression::Literal(Literal::Int(0)))),
            end: Some(Box::new(Expression::Literal(Literal::Int(10)))),
            inclusive: false,
        };
        assert_eq!(gen.generate_expression_string(&range), "0..10");

        // 0..=10
        let range_inclusive = Expression::Range {
            start: Some(Box::new(Expression::Literal(Literal::Int(0)))),
            end: Some(Box::new(Expression::Literal(Literal::Int(10)))),
            inclusive: true,
        };
        assert_eq!(gen.generate_expression_string(&range_inclusive), "0..=10");
    }

    #[test]
    fn test_generate_simple_macro() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let macro_def = MacroDefinition {
            name: Ident::new("__MAX__".to_string()),
            params: vec![],
            body: "100".to_string(),
        };

        let file = File {
            items: vec![Item::MacroDefinition(macro_def)],
            doc_comments: vec![],
        };

        let output = gen.generate(&file);
        assert!(output.contains("macro_rules! max"));
        assert!(output.contains("100"));
    }

    #[test]
    fn test_generate_macro_with_params() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let macro_def = MacroDefinition {
            name: Ident::new("__ADD__".to_string()),
            params: vec![Ident::new("a".to_string()), Ident::new("b".to_string())],
            body: "(a + b)".to_string(),
        };

        let file = File {
            items: vec![Item::MacroDefinition(macro_def)],
            doc_comments: vec![],
        };

        let output = gen.generate(&file);
        assert!(output.contains("macro_rules! add"));
        assert!(output.contains("$a:expr"));
        assert!(output.contains("$b:expr"));
        // Parameter references in the body are substituted with their $ form.
        assert!(output.contains("$a"));
        assert!(output.contains("$b"));
    }

    #[test]
    fn test_generate_macro_params_substituted_not_bare() {
        // A parameter reference becomes $name; a non-parameter identifier is
        // left untouched (word-boundary substitution, ADR-0001).
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let macro_def = MacroDefinition {
            name: Ident::new("__SCALE__".to_string()),
            params: vec![Ident::new("x".to_string())],
            body: "x * factor".to_string(),
        };

        let file = File {
            items: vec![Item::MacroDefinition(macro_def)],
            doc_comments: vec![],
        };

        let output = gen.generate(&file);
        assert!(output.contains("$x * factor"), "got: {}", output);
    }

    #[test]
    fn test_generate_macro_with_ternary() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let macro_def = MacroDefinition {
            name: Ident::new("__MAX__".to_string()),
            params: vec![Ident::new("a".to_string()), Ident::new("b".to_string())],
            body: "a > b ? a : b".to_string(),
        };

        let file = File {
            items: vec![Item::MacroDefinition(macro_def)],
            doc_comments: vec![],
        };

        let output = gen.generate(&file);
        assert!(output.contains("macro_rules! max"));
        assert!(output.contains("$a:expr"));
        assert!(output.contains("$b:expr"));
        // Body is passed through verbatim (with params substituted).
        assert!(output.contains("$a > $b ? $a : $b"), "got: {}", output);
    }

    #[test]
    fn test_generate_macro_removes_double_underscores() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let macro_def = MacroDefinition {
            name: Ident::new("__MY_MACRO__".to_string()),
            params: vec![],
            body: "42".to_string(),
        };

        let file = File {
            items: vec![Item::MacroDefinition(macro_def)],
            doc_comments: vec![],
        };

        let output = gen.generate(&file);
        // Should remove double-underscores and convert to lowercase
        assert!(output.contains("macro_rules! my_macro"));
        assert!(!output.contains("__MY_MACRO__"));
    }

    #[test]
    fn test_generate_macro_empty_body() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let macro_def = MacroDefinition {
            name: Ident::new("__EMPTY__".to_string()),
            params: vec![],
            body: String::new(),
        };

        let file = File {
            items: vec![Item::MacroDefinition(macro_def)],
            doc_comments: vec![],
        };

        let output = gen.generate(&file);
        assert!(output.contains("macro_rules! empty"));
        // Should have empty body
        assert!(output.contains("{{"));
        assert!(output.contains("}}"));
    }

    #[test]
    fn test_generate_define_constant_is_const() {
        // #define __PI__ 3.14159 lowers to a Rust const (ADR-0001).
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let const_item = Const {
            visibility: Visibility::Private,
            name: Ident::new("RATE".to_string()),
            ty: Type::Primitive(PrimitiveType::F64),
            value: Expression::Literal(Literal::Float(2.5)),
            doc_comments: vec![],
        };

        let file = File {
            items: vec![Item::Const(const_item)],
            doc_comments: vec![],
        };

        let output = gen.generate(&file);
        assert!(output.contains("const RATE: f64 = 2.5;"), "got: {}", output);
        assert!(!output.contains("macro_rules!"));
    }

    #[test]
    fn test_generate_conditional_compilation() {
        // #ifdef FEATURE … #endif emits #[cfg(FEATURE)] on each item.
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let func = Function {
            visibility: Visibility::Private,
            name: Ident::new("f".to_string()),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![])),
            doc_comments: vec![],
            attributes: vec![],
        };
        let cc = ConditionalCompilation {
            cfg: Ident::new("FEATURE".to_string()),
            negated: false,
            items: vec![Item::Function(func)],
        };

        let file = File {
            items: vec![Item::ConditionalCompilation(cc)],
            doc_comments: vec![],
        };

        let output = gen.generate(&file);
        assert!(output.contains("#[cfg(FEATURE)]"), "got: {}", output);
    }

    #[test]
    fn test_generate_conditional_compilation_negated() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let func = Function {
            visibility: Visibility::Private,
            name: Ident::new("f".to_string()),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![])),
            doc_comments: vec![],
            attributes: vec![],
        };
        let cc = ConditionalCompilation {
            cfg: Ident::new("FEATURE".to_string()),
            negated: true,
            items: vec![Item::Function(func)],
        };

        let file = File {
            items: vec![Item::ConditionalCompilation(cc)],
            doc_comments: vec![],
        };

        let output = gen.generate(&file);
        assert!(output.contains("#[cfg(not(FEATURE))]"), "got: {}", output);
    }

    #[test]
    fn test_generate_namespace_emits_mod() {
        // namespace Name { fn } → pub mod Name { pub fn }
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("add".to_string()),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![])),
            doc_comments: vec![],
            attributes: vec![],
        };
        let ns = Namespace {
            name: Ident::new("math".to_string()),
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let file = File {
            items: vec![Item::Namespace(ns)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("pub mod math {"), "got: {}", output);
        assert!(output.contains("pub fn add"), "got: {}", output);
        assert!(output.trim_end().ends_with('}'), "mod should close: {}", output);
    }

    #[test]
    fn test_generate_namespace_nested() {
        // Nested namespaces produce nested mods.
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let inner = Namespace {
            name: Ident::new("inner".to_string()),
            items: vec![],
            doc_comments: vec![],
        };
        let outer = Namespace {
            name: Ident::new("outer".to_string()),
            items: vec![Item::Namespace(inner)],
            doc_comments: vec![],
        };
        let file = File {
            items: vec![Item::Namespace(outer)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("pub mod outer {"), "got: {}", output);
        assert!(output.contains("pub mod inner {"), "got: {}", output);
    }

    #[test]
    fn test_generate_namespace_private_member() {
        // A static (private) member inside a namespace emits no `pub`.
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let func = Function {
            visibility: Visibility::Private,
            name: Ident::new("helper".to_string()),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![])),
            doc_comments: vec![],
            attributes: vec![],
        };
        let ns = Namespace {
            name: Ident::new("util".to_string()),
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let file = File {
            items: vec![Item::Namespace(ns)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("    fn helper"), "got: {}", output);
        assert!(!output.contains("pub fn helper"), "helper should be private: {}", output);
    }

    #[test]
    fn test_generate_typedef_simple() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let typedef = Typedef {
            visibility: Visibility::Public,
            name: Ident::new("MyInt".to_string()),
            target: Type::Primitive(PrimitiveType::Int),
            doc_comments: vec![],
        };

        let file = File {
            items: vec![Item::Typedef(typedef)],
            doc_comments: vec![],
        };

        let output = gen.generate(&file);
        assert!(output.contains("pub type MyInt = i32;"));
    }

    #[test]
    fn test_generate_typedef_with_pointer() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let typedef = Typedef {
            visibility: Visibility::Public,
            name: Ident::new("IntPtr".to_string()),
            target: Type::Pointer {
                ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                mutable: false,
            },
            doc_comments: vec![],
        };

        let file = File {
            items: vec![Item::Typedef(typedef)],
            doc_comments: vec![],
        };

        let output = gen.generate(&file);
        assert!(output.contains("pub type IntPtr = *const i32;"));
    }

    #[test]
    fn test_generate_typedef_with_reference() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let typedef = Typedef {
            visibility: Visibility::Public,
            name: Ident::new("IntRef".to_string()),
            target: Type::Reference {
                ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                mutable: false,
            },
            doc_comments: vec![],
        };

        let file = File {
            items: vec![Item::Typedef(typedef)],
            doc_comments: vec![],
        };

        let output = gen.generate(&file);
        assert!(output.contains("pub type IntRef = &i32;"));
    }

    #[test]
    fn test_generate_typedef_private() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let typedef = Typedef {
            visibility: Visibility::Private,
            name: Ident::new("PrivateInt".to_string()),
            target: Type::Primitive(PrimitiveType::Int),
            doc_comments: vec![],
        };

        let file = File {
            items: vec![Item::Typedef(typedef)],
            doc_comments: vec![],
        };

        let output = gen.generate(&file);
        assert!(output.contains("type PrivateInt = i32;"));
        assert!(!output.contains("pub type"));
    }

    #[test]
    fn test_generate_typedef_with_doc_comments() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let typedef = Typedef {
            visibility: Visibility::Public,
            name: Ident::new("MyInt".to_string()),
            target: Type::Primitive(PrimitiveType::Int),
            doc_comments: vec![" A custom integer type".to_string()],
        };

        let file = File {
            items: vec![Item::Typedef(typedef)],
            doc_comments: vec![],
        };

        let output = gen.generate(&file);
        assert!(output.contains("/// A custom integer type"));
        assert!(output.contains("pub type MyInt = i32;"));
    }

    #[test]
    fn test_generate_typedef_struct_pattern() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let typedef = Typedef {
            visibility: Visibility::Public,
            name: Ident::new("Point".to_string()),
            target: Type::Ident(Ident::new("PointStruct".to_string())),
            doc_comments: vec![],
        };

        let file = File {
            items: vec![Item::Typedef(typedef)],
            doc_comments: vec![],
        };

        let output = gen.generate(&file);
        assert!(output.contains("pub type Point = PointStruct;"));
    }

    #[test]
    fn test_explicit_generic_simple() {
        // Test: Type(T) → Type::<T>
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::ExplicitGenericCall {
            ty: Type::Ident(Ident::new("Vec")),
            generics: vec![Type::Primitive(PrimitiveType::I32)],
            method: Ident::new("new"),
            args: vec![],
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "Vec::<i32>::new()");
    }

    #[test]
    fn test_explicit_generic_nested() {
        // Test: Type(Inner[T]) → Type::<Inner<T>>
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let inner_generic = Type::Generic {
            base: Box::new(Type::Ident(Ident::new("Inner"))),
            args: vec![Type::Ident(Ident::new("T"))],
        };
        let expr = Expression::ExplicitGenericCall {
            ty: Type::Ident(Ident::new("Outer")),
            generics: vec![inner_generic],
            method: Ident::new("create"),
            args: vec![],
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "Outer::<Inner<T>>::create()");
    }

    #[test]
    fn test_explicit_generic_multiple_params() {
        // Test: Type(T1, T2) → Type::<T1, T2>
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::ExplicitGenericCall {
            ty: Type::Ident(Ident::new("HashMap")),
            generics: vec![
                Type::Ident(Ident::new("String")),
                Type::Primitive(PrimitiveType::I32),
            ],
            method: Ident::new("new"),
            args: vec![],
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "HashMap::<String, i32>::new()");
    }

    #[test]
    fn test_explicit_generic_with_args() {
        // Test: Type(T).method(arg1, arg2) → Type::<T>::method(arg1, arg2)
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::ExplicitGenericCall {
            ty: Type::Ident(Ident::new("Vec")),
            generics: vec![Type::Primitive(PrimitiveType::I32)],
            method: Ident::new("from_iter"),
            args: vec![Expression::Ident(Ident::new("iter"))],
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "Vec::<i32>::from_iter(iter)");
    }

    #[test]
    fn test_explicit_generic_deeply_nested() {
        // Test: Type(Result[String, Error]) → Type::<Result<String, Error>>
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let result_generic = Type::Generic {
            base: Box::new(Type::Ident(Ident::new("Result"))),
            args: vec![
                Type::Ident(Ident::new("String")),
                Type::Ident(Ident::new("Error")),
            ],
        };
        let expr = Expression::ExplicitGenericCall {
            ty: Type::Ident(Ident::new("Option")),
            generics: vec![result_generic],
            method: Ident::new("Some"),
            args: vec![],
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "Option::<Result<String, Error>>::Some()");
    }

    #[test]
    fn test_generate_null_literal_to_rust() {
        // Test: NULL → Option::None
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let lit = Literal::Null;
        let result = gen.generate_literal_string_for_test(&lit);
        assert_eq!(result, "Option::None");
    }

    #[test]
    fn test_generate_null_literal_to_crusty() {
        // Test: NULL → NULL (when generating Crusty)
        let gen = CodeGenerator::new(TargetLanguage::Crusty);
        let lit = Literal::Null;
        let result = gen.generate_literal_string_for_test(&lit);
        assert_eq!(result, "NULL");
    }

    #[test]
    fn test_generate_null_in_expression() {
        // Test: NULL in expression context
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let stmt = Statement::Let {
            name: Ident::new("ptr"),
            ty: None,
            init: Some(Expression::Literal(Literal::Null)),
            mutable: false,
        };
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("let ptr = Option::None;"));
    }

    #[test]
    fn test_generate_null_comparison() {
        // Test: ptr == NULL → ptr.is_none() (future enhancement)
        // For now, just test that NULL generates correctly
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::Binary {
            op: BinaryOp::Eq,
            left: Box::new(Expression::Ident(Ident::new("ptr"))),
            right: Box::new(Expression::Literal(Literal::Null)),
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "(ptr == Option::None)");
    }

    #[test]
    fn test_generate_null_assignment() {
        // Test: ptr = NULL
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::Binary {
            op: BinaryOp::Assign,
            left: Box::new(Expression::Ident(Ident::new("ptr"))),
            right: Box::new(Expression::Literal(Literal::Null)),
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "(ptr = Option::None)");
    }

    #[test]
    fn test_generate_switch_simple() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let stmt = Statement::Switch {
            expr: Expression::Ident(Ident::new("x")),
            cases: vec![
                SwitchCase {
                    values: vec![Expression::Literal(Literal::Int(1))],
                    body: Block::new(vec![Statement::Expr(Expression::Literal(Literal::Int(10)))]),
                },
                SwitchCase {
                    values: vec![Expression::Literal(Literal::Int(2))],
                    body: Block::new(vec![Statement::Expr(Expression::Literal(Literal::Int(20)))]),
                },
            ],
            default: None,
        };
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);

        assert!(output.contains("match x {"));
        assert!(output.contains("1 => {"));
        assert!(output.contains("2 => {"));
    }

    #[test]
    fn test_generate_switch_with_default() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let stmt = Statement::Switch {
            expr: Expression::Ident(Ident::new("x")),
            cases: vec![SwitchCase {
                values: vec![Expression::Literal(Literal::Int(1))],
                body: Block::new(vec![Statement::Expr(Expression::Literal(Literal::Int(10)))]),
            }],
            default: Some(Block::new(vec![Statement::Expr(Expression::Literal(
                Literal::Int(0),
            ))])),
        };
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);

        assert!(output.contains("match x {"));
        assert!(output.contains("1 => {"));
        assert!(output.contains("_ => {"));
    }

    #[test]
    fn test_generate_switch_with_multiple_values() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let stmt = Statement::Switch {
            expr: Expression::Ident(Ident::new("x")),
            cases: vec![SwitchCase {
                values: vec![
                    Expression::Literal(Literal::Int(1)),
                    Expression::Literal(Literal::Int(2)),
                    Expression::Literal(Literal::Int(3)),
                ],
                body: Block::new(vec![Statement::Expr(Expression::Literal(Literal::Int(10)))]),
            }],
            default: None,
        };
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);

        assert!(output.contains("match x {"));
        assert!(output.contains("1 | 2 | 3 => {"));
    }

    #[test]
    fn test_generate_switch_with_break() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let stmt = Statement::Switch {
            expr: Expression::Ident(Ident::new("x")),
            cases: vec![SwitchCase {
                values: vec![Expression::Literal(Literal::Int(1))],
                body: Block::new(vec![
                    Statement::Expr(Expression::Literal(Literal::Int(10))),
                    Statement::Break(None),
                ]),
            }],
            default: None,
        };
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);

        assert!(output.contains("match x {"));
        assert!(output.contains("1 => {"));
        assert!(output.contains("break;"));
    }

    #[test]
    fn test_generate_fallible_type() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let fallible_type = Type::Fallible {
            ty: Box::new(Type::Primitive(PrimitiveType::I32)),
        };
        let result = gen.generate_type_string(&fallible_type);
        assert_eq!(result, "Result<i32, Box<dyn std::error::Error>>");
    }

    #[test]
    fn test_generate_error_prop_operator() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::ErrorProp {
            expr: Box::new(Expression::Call {
                func: Box::new(Expression::Ident(Ident::new("read_file"))),
                args: vec![],
            }),
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "read_file()?");
    }

    #[test]
    fn test_generate_fallible_function_return() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("read_config"),
            params: vec![],
            return_type: Some(Type::Fallible {
                ty: Box::new(Type::Ident(Ident::new("Config"))),
            }),
            body: Some(Block::empty()),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);

        assert!(output.contains("fn read_config()"));
        assert!(output.contains("-> Result<Config, Box<dyn std::error::Error>>"));
    }

    #[test]
    fn test_generate_nested_error_prop() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::ErrorProp {
            expr: Box::new(Expression::ErrorProp {
                expr: Box::new(Expression::Call {
                    func: Box::new(Expression::Ident(Ident::new("parse"))),
                    args: vec![],
                }),
            }),
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "parse()??");
    }
}

#[cfg(test)]
mod struct_init_tests {
    use crate::ast::*;
    use crate::codegen::{CodeGenerator, TargetLanguage};

    #[test]
    fn test_generate_struct_initializer() {
        let codegen = CodeGenerator::new(TargetLanguage::Rust);

        let struct_init = Expression::StructInit {
            ty: Type::Ident(Ident::new("Point")),
            fields: vec![
                (Ident::new("x"), Expression::Literal(Literal::Int(10))),
                (Ident::new("y"), Expression::Literal(Literal::Int(20))),
            ],
        };

        let result = codegen.generate_expression_string(&struct_init);
        assert_eq!(result, "Point { x: 10, y: 20 }");
    }

    #[test]
    fn test_generate_struct_initializer_partial() {
        let codegen = CodeGenerator::new(TargetLanguage::Rust);

        let struct_init = Expression::StructInit {
            ty: Type::Ident(Ident::new("Point")),
            fields: vec![(Ident::new("x"), Expression::Literal(Literal::Int(10)))],
        };

        let result = codegen.generate_expression_string(&struct_init);
        assert_eq!(result, "Point { x: 10 }");
    }

    #[test]
    fn test_generate_struct_initializer_nested() {
        let codegen = CodeGenerator::new(TargetLanguage::Rust);

        let struct_init = Expression::StructInit {
            ty: Type::Ident(Ident::new("Rect")),
            fields: vec![
                (
                    Ident::new("origin"),
                    Expression::StructInit {
                        ty: Type::Ident(Ident::new("Point")),
                        fields: vec![
                            (Ident::new("x"), Expression::Literal(Literal::Int(0))),
                            (Ident::new("y"), Expression::Literal(Literal::Int(0))),
                        ],
                    },
                ),
                (
                    Ident::new("size"),
                    Expression::StructInit {
                        ty: Type::Ident(Ident::new("Size")),
                        fields: vec![
                            (Ident::new("w"), Expression::Literal(Literal::Int(10))),
                            (Ident::new("h"), Expression::Literal(Literal::Int(20))),
                        ],
                    },
                ),
            ],
        };

        let result = codegen.generate_expression_string(&struct_init);
        assert_eq!(
            result,
            "Rect { origin: Point { x: 0, y: 0 }, size: Size { w: 10, h: 20 } }"
        );
    }

    #[test]
    fn test_generate_struct_initializer_with_auto_type() {
        let codegen = CodeGenerator::new(TargetLanguage::Rust);

        // When type is Auto, it should be inferred from context
        // For now, we'll just generate the type as-is
        let struct_init = Expression::StructInit {
            ty: Type::Auto,
            fields: vec![
                (Ident::new("x"), Expression::Literal(Literal::Int(10))),
                (Ident::new("y"), Expression::Literal(Literal::Int(20))),
            ],
        };

        let result = codegen.generate_expression_string(&struct_init);
        // Auto type should be omitted in Rust (type inference)
        // But for struct initializers, we need the type name
        // This is a limitation - we'll need semantic analysis to resolve Auto types
        assert!(result.contains("x: 10"));
        assert!(result.contains("y: 20"));
    }
}
