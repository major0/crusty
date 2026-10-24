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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("main"),
            params: vec![],
            return_type: None,
            body: Some(Block::empty()),
            is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("foo"),
            params: vec![],
            return_type: Some(Type::Primitive(PrimitiveType::Void)),
            body: Some(Block::empty()),
            is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
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
            is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Private,
            name: Ident::new("helper"),
            params: vec![],
            return_type: None,
            body: Some(Block::empty()),
            is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            is_variadic: false,
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
    fn test_generate_attributed_statement() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let inner = Statement::Let {
            name: Ident::new("x"),
            ty: None,
            init: Some(Expression::Literal(Literal::Int(1))),
            mutable: false,
        };
        let stmt = Statement::Attributed {
            attributes: vec![Attribute {
                name: Ident::new("allow"),
                args: vec![AttributeArg::Ident(Ident::new("unused"))],
            }],
            stmt: Box::new(inner),
        };
        let func = Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            is_variadic: false,
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("#[allow(unused)]"), "output: {output}");
        assert!(output.contains("let x = 1;"), "output: {output}");
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            is_variadic: false,
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
    fn test_generate_loop_statement() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let stmt = Statement::Loop {
            label: None,
            body: Block::new(vec![Statement::Break { label: None, value: None }]),
        };
        let func = Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            is_variadic: false,
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("loop {"), "got: {}", output);
        assert!(!output.contains("while"), "loop must not emit a condition: {}", output);
    }

    #[test]
    fn test_generate_labeled_loop() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let stmt = Statement::Loop {
            label: Some(Ident::new("outer")),
            body: Block::empty(),
        };
        let func = Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            is_variadic: false,
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("'outer: loop"), "got: {}", output);
    }

    #[test]
    fn test_generate_loop_expression() {
        // `let x = loop { break 7; };` -> the loop renders in expression
        // position as `loop { break 7; }` on the let initializer.
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let loop_expr = Expression::Loop {
            label: None,
            body: Box::new(Block::new(vec![Statement::Break {
                label: None,
                value: Some(Expression::Literal(Literal::Int(7))),
            }])),
        };
        let let_stmt = Statement::Let {
            name: Ident::new("x"),
            ty: None,
            init: Some(loop_expr),
            mutable: false,
        };
        let func = Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![let_stmt])),
            is_variadic: false,
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("let x = loop {"), "got: {}", output);
        assert!(output.contains("break 7;"), "got: {}", output);
    }

    #[test]
    fn test_generate_break_with_label() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let stmt = Statement::Break { label: Some(Ident::new("outer")), value: None };
        let func = Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            is_variadic: false,
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
    fn test_generate_break_with_value() {
        // `break 42;` -> `break 42;` (value yielded from a loop expression).
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let stmt = Statement::Break {
            label: None,
            value: Some(Expression::Literal(Literal::Int(42))),
        };
        let func = Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            is_variadic: false,
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("break 42;"), "got: {}", output);
    }

    #[test]
    fn test_generate_break_label_and_value() {
        // `break .outer 42;` -> `break 'outer 42;`.
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let stmt = Statement::Break {
            label: Some(Ident::new("outer")),
            value: Some(Expression::Literal(Literal::Int(42))),
        };
        let func = Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            is_variadic: false,
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("break 'outer 42;"), "got: {}", output);
    }

    #[test]
    fn test_generate_continue_with_label() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let stmt = Statement::Continue(Some(Ident::new("outer")));
        let func = Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            is_variadic: false,
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
        assert_eq!(result, "1 + 2");
    }

    // --- PS-06 Phase 2: match-expression codegen (ADR-0012) ---

    fn match_arm(pattern: Pattern, guard: Option<Expression>, body: Expression) -> MatchArm {
        MatchArm {
            pattern,
            guard,
            body: Box::new(body),
        }
    }

    #[test]
    fn test_generate_match_literal_wildcard() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::Match {
            scrutinee: Box::new(Expression::Ident(Ident::new("n"))),
            arms: vec![
                match_arm(
                    Pattern::Literal(Literal::Int(1)),
                    None,
                    Expression::Literal(Literal::Int(10)),
                ),
                match_arm(Pattern::Wildcard, None, Expression::Literal(Literal::Int(0))),
            ],
        };
        let out = gen.generate_expression_string(&expr);
        assert!(out.starts_with("match n {"), "got: {out}");
        assert!(out.contains("1 => 10,"), "got: {out}");
        assert!(out.contains("_ => 0,"), "got: {out}");
    }

    #[test]
    fn test_generate_match_guard_range_or_path_raw() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let body = |n| Expression::Literal(Literal::Int(n));
        let expr = Expression::Match {
            scrutinee: Box::new(Expression::Ident(Ident::new("x"))),
            arms: vec![
                // inclusive range
                match_arm(
                    Pattern::Range {
                        lo: Box::new(Expression::Literal(Literal::Int(1))),
                        hi: Box::new(Expression::Literal(Literal::Int(5))),
                        inclusive: true,
                    },
                    None,
                    body(1),
                ),
                // binding with guard
                match_arm(
                    Pattern::Binding(Ident::new("v")),
                    Some(Expression::Binary {
                        op: BinaryOp::Gt,
                        left: Box::new(Expression::Ident(Ident::new("v"))),
                        right: Box::new(Expression::Literal(Literal::Int(0))),
                    }),
                    body(2),
                ),
                // or-alternation
                match_arm(
                    Pattern::Or(vec![
                        Pattern::Literal(Literal::Int(7)),
                        Pattern::Literal(Literal::Int(8)),
                    ]),
                    None,
                    body(3),
                ),
                // path pattern
                match_arm(
                    Pattern::Path(vec![Ident::new("Color"), Ident::new("Red")]),
                    None,
                    body(4),
                ),
                // raw passthrough
                match_arm(Pattern::Raw("Some(y)".to_string()), None, body(5)),
            ],
        };
        let out = gen.generate_expression_string(&expr);
        assert!(out.contains("1..=5 => 1,"), "range: {out}");
        assert!(out.contains("v if v > 0 => 2,"), "guard: {out}");
        assert!(out.contains("7 | 8 => 3,"), "or: {out}");
        assert!(out.contains("Color::Red => 4,"), "path: {out}");
        assert!(out.contains("Some(y) => 5,"), "raw: {out}");
    }

    // CG-1: parentheses are emitted only where precedence/associativity need
    // them, not around every binary (which tripped rustc's unused_parens).
    #[test]
    fn test_binary_minimal_parens() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let lit = |n| Box::new(Expression::Literal(Literal::Int(n)));
        let bin = |op, l, r| Expression::Binary { op, left: l, right: r };

        // a * b + c : `*` binds tighter, no parens needed anywhere.
        let e = bin(
            BinaryOp::Add,
            Box::new(bin(BinaryOp::Mul, lit(1), lit(2))),
            lit(3),
        );
        assert_eq!(gen.generate_expression_string(&e), "1 * 2 + 3");

        // a * (b + c) : the looser `+` child must be parenthesised.
        let e = bin(
            BinaryOp::Mul,
            lit(1),
            Box::new(bin(BinaryOp::Add, lit(2), lit(3))),
        );
        assert_eq!(gen.generate_expression_string(&e), "1 * (2 + 3)");

        // a - (b - c) : right child at equal precedence needs parens
        // (subtraction is left-associative).
        let e = bin(
            BinaryOp::Sub,
            lit(1),
            Box::new(bin(BinaryOp::Sub, lit(2), lit(3))),
        );
        assert_eq!(gen.generate_expression_string(&e), "1 - (2 - 3)");

        // (a - b) - c : left child at equal precedence needs NO parens.
        let e = bin(
            BinaryOp::Sub,
            Box::new(bin(BinaryOp::Sub, lit(1), lit(2))),
            lit(3),
        );
        assert_eq!(gen.generate_expression_string(&e), "1 - 2 - 3");
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
    fn test_generate_turbofish_call() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::TurbofishCall {
            ty: Type::Ident(Ident::new("mem")),
            method: Ident::new("size_of"),
            generics: vec![Type::Primitive(PrimitiveType::I32)],
            args: vec![],
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "mem::size_of::<i32>()");
    }

    #[test]
    fn test_generate_type_scoped_path() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::TypeScopedPath {
            ty: Type::Ident(Ident::new("Option")),
            member: Ident::new("None"),
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "Option::None");
    }

    #[test]
    fn test_generate_struct() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let struct_def = Struct {
            generic_params: vec![],
            where_clause: None,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("new"),
            params: vec![],
            return_type: Some(Type::Ident(Ident::new("Self"))),
            body: Some(Block::empty()),
            is_variadic: false,
            doc_comments: vec![],
            attributes: vec![],
        };
        let struct_def = Struct {
            generic_params: vec![],
            where_clause: None,
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
            generic_params: vec![],
            where_clause: None,
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
            is_variadic: false,
            doc_comments: vec![],
            attributes: vec![],
        };
        let struct_def = Struct {
            generic_params: vec![],
            where_clause: None,
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
            generic_params: vec![],
            where_clause: None,
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
            is_variadic: false,
            doc_comments: vec![],
            attributes: vec![],
        };
        let struct_def = Struct {
            generic_params: vec![],
            where_clause: None,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Private,
            name: Ident::new("origin"),
            params: vec![],
            return_type: Some(Type::Ident(Ident::new("Self"))),
            body: Some(Block::empty()),
            is_variadic: false,
            doc_comments: vec![],
            attributes: vec![],
        };
        let struct_def = Struct {
            generic_params: vec![],
            where_clause: None,
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
            generic_params: vec![],
            where_clause: None,
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

    // --- ADR-0011 Phase 2: generic parameters lower to Rust ---

    #[test]
    fn test_generate_generic_params_string_forms() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        // Empty list emits nothing.
        assert_eq!(gen.generate_generic_params_string(&[]), "");
        // A type param with bounds, a lifetime, and a const generic.
        let params = vec![
            GenericParam::Lifetime {
                name: Ident::new("a"),
            },
            GenericParam::Type {
                name: Ident::new("T"),
                bounds: vec!["Clone".to_string(), "Debug".to_string()],
            },
            GenericParam::Const {
                name: Ident::new("N"),
                ty: Type::Primitive(PrimitiveType::Usize),
            },
        ];
        assert_eq!(
            gen.generate_generic_params_string(&params),
            "<'a, T: Clone + Debug, const N: usize>"
        );
        // The bare argument form drops bounds and the const type.
        assert_eq!(gen.generate_generic_args_string(&params), "<'a, T, N>");
    }

    #[test]
    fn test_generate_generic_function_with_where() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let func = Function {
            generic_params: vec![GenericParam::Type {
                name: Ident::new("T"),
                bounds: vec!["Clone".to_string()],
            }],
            where_clause: Some("T: Copy".to_string()),
            visibility: Visibility::Public,
            name: Ident::new("echo"),
            params: vec![Param {
                name: Ident::new("x"),
                ty: Type::Ident(Ident::new("T")),
            }],
            return_type: Some(Type::Ident(Ident::new("T"))),
            body: Some(Block::empty()),
            is_variadic: false,
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(
            output.contains("pub fn echo<T: Clone>(x: T) -> T where T: Copy"),
            "got: {output}"
        );
    }

    #[test]
    fn test_generate_generic_struct_with_impl() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let method = Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Private,
            name: Ident::new("first"),
            params: vec![Param {
                name: Ident::new("self"),
                ty: Type::Reference {
                    ty: Box::new(Type::Ident(Ident::new("Self"))),
                    mutable: false,
                },
            }],
            return_type: Some(Type::Ident(Ident::new("A"))),
            body: Some(Block::empty()),
            is_variadic: false,
            doc_comments: vec![],
            attributes: vec![],
        };
        let struct_def = Struct {
            generic_params: vec![
                GenericParam::Type {
                    name: Ident::new("A"),
                    bounds: vec![],
                },
                GenericParam::Type {
                    name: Ident::new("B"),
                    bounds: vec![],
                },
            ],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("Pair"),
            fields: vec![
                Field {
                    visibility: Visibility::Public,
                    name: Ident::new("first"),
                    ty: Type::Ident(Ident::new("A")),
                    doc_comments: vec![],
                    attributes: vec![],
                },
                Field {
                    visibility: Visibility::Public,
                    name: Ident::new("second"),
                    ty: Type::Ident(Ident::new("B")),
                    doc_comments: vec![],
                    attributes: vec![],
                },
            ],
            methods: vec![method],
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Struct(struct_def)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("pub struct Pair<A, B>"), "got: {output}");
        // The impl declares the params and references them as arguments.
        assert!(output.contains("impl<A, B> Pair<A, B>"), "got: {output}");
    }

    #[test]
    fn test_generate_generic_enum() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let enum_def = Enum {
            generic_params: vec![GenericParam::Type {
                name: Ident::new("T"),
                bounds: vec![],
            }],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("Opt"),
            variants: vec![
                EnumVariant {
                    name: Ident::new("None"),
                    value: Some(0),
                },
                EnumVariant {
                    name: Ident::new("Some"),
                    value: Some(1),
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
        assert!(output.contains("pub enum Opt<T>"), "got: {output}");
    }

    #[test]
    fn test_generate_non_generic_definition_unchanged() {
        // A struct with no generic params must emit no angle brackets.
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let struct_def = Struct {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("Plain"),
            fields: vec![],
            methods: vec![],
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Struct(struct_def)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("pub struct Plain {"), "got: {output}");
        assert!(!output.contains("Plain<"), "got: {output}");
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
            gen.generate_type_string(&Type::Primitive(PrimitiveType::I8)),
            "i8"
        );
        assert_eq!(
            gen.generate_type_string(&Type::Primitive(PrimitiveType::I16)),
            "i16"
        );
        assert_eq!(
            gen.generate_type_string(&Type::Primitive(PrimitiveType::I128)),
            "i128"
        );
        assert_eq!(
            gen.generate_type_string(&Type::Primitive(PrimitiveType::U8)),
            "u8"
        );
        assert_eq!(
            gen.generate_type_string(&Type::Primitive(PrimitiveType::U16)),
            "u16"
        );
        assert_eq!(
            gen.generate_type_string(&Type::Primitive(PrimitiveType::U128)),
            "u128"
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
    fn test_generate_usize_isize() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        assert_eq!(
            gen.generate_type_string(&Type::Primitive(PrimitiveType::Usize)),
            "usize"
        );
        assert_eq!(
            gen.generate_type_string(&Type::Primitive(PrimitiveType::Isize)),
            "isize"
        );
    }

    #[test]
    fn test_generate_c_posix_aliases() {
        // C/POSIX alias identifier types lower to their Rust equivalents
        // (ADR-0003). Exercised through a full file so user_type_names is set.
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let func = Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("f"),
            params: vec![
                Param { name: Ident::new("a"), ty: Type::Ident(Ident::new("uint8_t")) },
                Param { name: Ident::new("b"), ty: Type::Ident(Ident::new("size_t")) },
                Param { name: Ident::new("c"), ty: Type::Ident(Ident::new("byte")) },
            ],
            return_type: Some(Type::Ident(Ident::new("ssize_t"))),
            body: Some(Block::empty()),
            is_variadic: false,
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File { items: vec![Item::Function(func)], doc_comments: vec![] };
        let output = gen.generate(&file);
        assert!(
            output.contains("fn f(a: u8, b: usize, c: u8) -> isize"),
            "C aliases should lower to Rust types: {}",
            output
        );
    }

    #[test]
    fn test_c_alias_unknown_ident_verbatim() {
        // A non-alias identifier type is emitted unchanged.
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        assert_eq!(
            gen.generate_type_string(&Type::Ident(Ident::new("Widget"))),
            "Widget"
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
    fn test_generate_item_macro_invocation_brace() {
        // An item-position brace macro (bitflags!) emits `name! { body }` with
        // the body verbatim and NO trailing semicolon.
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let file = File {
            items: vec![Item::MacroInvocation {
                name: Ident::new("bitflags".to_string()),
                args: vec![Token {
                    kind: TokenKind::Other,
                    text: " struct Flags: u32 { const A = 1; } ".to_string(),
                }],
                delimiter: MacroDelimiter::Braces,
            }],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("bitflags!{"), "brace form: {output}");
        assert!(output.contains("struct Flags: u32"), "body verbatim: {output}");
        assert!(!output.contains("};"), "brace item macro takes no semicolon: {output}");
    }

    #[test]
    fn test_generate_item_macro_invocation_paren_gets_semi() {
        // A paren-form item macro emits a trailing semicolon.
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let file = File {
            items: vec![Item::MacroInvocation {
                name: Ident::new("my_macro".to_string()),
                args: vec![Token {
                    kind: TokenKind::Other,
                    text: "a, b".to_string(),
                }],
                delimiter: MacroDelimiter::Parens,
            }],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("my_macro!(a, b);"), "paren form with semi: {output}");
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Private,
            name: Ident::new("f".to_string()),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![])),
            is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Private,
            name: Ident::new("f".to_string()),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![])),
            is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("add".to_string()),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![])),
            is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Private,
            name: Ident::new("helper".to_string()),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![])),
            is_variadic: false,
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
    fn test_generate_import_export() {
        // #import a.b.c -> use a::b::c;   #export a.b -> pub use a::b;
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let import = Item::Import(Import {
            path: vec![
                Ident::new("std".to_string()),
                Ident::new("collections".to_string()),
                Ident::new("HashMap".to_string()),
            ],
            alias: None,
        });
        let export = Item::Export(Export {
            path: vec![
                Ident::new("mylib".to_string()),
                Ident::new("Config".to_string()),
            ],
            alias: None,
        });
        let file = File {
            items: vec![import, export],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(
            output.contains("use std::collections::HashMap;"),
            "got: {}",
            output
        );
        assert!(output.contains("pub use mylib::Config;"), "got: {}", output);
    }

    #[test]
    fn test_generate_extern_c_block() {
        // extern "C" { int puts(char* s); } → extern "C" { pub fn puts(...) -> i32; }
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let decl = Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("puts".to_string()),
            params: vec![Param {
                name: Ident::new("s".to_string()),
                ty: Type::Pointer {
                    ty: Box::new(Type::Primitive(PrimitiveType::Char)),
                    mutable: false,
                },
            }],
            return_type: Some(Type::Primitive(PrimitiveType::I32)),
            body: None,
            is_variadic: false,
            doc_comments: vec![],
            attributes: vec![],
        };
        let ext = Extern {
            abi: Some("C".to_string()),
            items: vec![Item::Function(decl)],
        };
        let file = File {
            items: vec![Item::Extern(ext)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("extern \"C\" {"), "got: {}", output);
        assert!(output.contains("pub fn puts"), "got: {}", output);
        // A body-less declaration ends in `;`, never `{ }`.
        assert!(output.contains(";"), "declaration should end with ';': {}", output);
        assert!(!output.contains("puts() {"), "should have no body: {}", output);
    }

    #[test]
    fn test_generate_extern_default_abi() {
        // A bare extern (no ABI) emits `extern {`.
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let decl = Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("rust_fn".to_string()),
            params: vec![],
            return_type: None,
            body: None,
            is_variadic: false,
            doc_comments: vec![],
            attributes: vec![],
        };
        let ext = Extern {
            abi: None,
            items: vec![Item::Function(decl)],
        };
        let file = File {
            items: vec![Item::Extern(ext)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("extern {"), "got: {}", output);
        assert!(!output.contains("extern \"\""), "no empty ABI string: {}", output);
    }

    #[test]
    fn test_generate_extern_variadic() {
        // extern "C" { int printf(char* format, ...); }
        //   → extern "C" { pub fn printf(format: *const char, ...) -> i32; }
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let decl = Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("printf".to_string()),
            params: vec![Param {
                name: Ident::new("format".to_string()),
                ty: Type::Pointer {
                    ty: Box::new(Type::Primitive(PrimitiveType::Char)),
                    mutable: false,
                },
            }],
            return_type: Some(Type::Primitive(PrimitiveType::I32)),
            body: None,
            is_variadic: true,
            doc_comments: vec![],
            attributes: vec![],
        };
        let ext = Extern {
            abi: Some("C".to_string()),
            items: vec![Item::Function(decl)],
        };
        let file = File {
            items: vec![Item::Extern(ext)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        // Named param then a comma-separated trailing `...`.
        assert!(
            output.contains("pub fn printf(format: *const char, ...) -> i32;"),
            "got: {}",
            output
        );
    }

    #[test]
    fn test_generate_extern_variadic_bare() {
        // A variadic with no named params emits `(...)` — no leading comma.
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let decl = Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("log_event".to_string()),
            params: vec![],
            return_type: None,
            body: None,
            is_variadic: true,
            doc_comments: vec![],
            attributes: vec![],
        };
        let ext = Extern {
            abi: Some("C".to_string()),
            items: vec![Item::Function(decl)],
        };
        let file = File {
            items: vec![Item::Extern(ext)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(
            output.contains("pub fn log_event(...);"),
            "bare variadic should have no leading comma: {}",
            output
        );
    }

    #[test]
    fn test_generate_extern_rust_escape_hatch() {
        // A __rust__ item inside an extern block emits its raw Rust verbatim,
        // indented alongside the ordinary declarations.
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let decl = Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("simple".to_string()),
            params: vec![],
            return_type: None,
            body: None,
            is_variadic: false,
            doc_comments: vec![],
            attributes: vec![],
        };
        let raw = Item::RawRust {
            code: "static mut GLOBAL_STATE: i32;".to_string(),
        };
        let ext = Extern {
            abi: Some("C".to_string()),
            items: vec![Item::Function(decl), raw],
        };
        let file = File {
            items: vec![Item::Extern(ext)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        // The raw line is passed through untouched, indented inside the block.
        assert!(
            output.contains("    static mut GLOBAL_STATE: i32;"),
            "raw Rust should be emitted verbatim and indented: {}",
            output
        );
        // It must NOT be wrapped in braces or treated as an expression.
        assert!(
            !output.contains("{ static mut"),
            "raw item must not be brace-wrapped: {}",
            output
        );
    }

    #[test]
    fn test_generate_static_immutable() {
        // static int COUNTER = 0; → pub static COUNTER: i32 = 0;
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let s = Static {
            visibility: Visibility::Public,
            name: Ident::new("COUNTER".to_string()),
            ty: Type::Primitive(PrimitiveType::I32),
            value: Expression::Literal(Literal::Int(0)),
            mutable: false,
            doc_comments: vec![],
        };
        let file = File {
            items: vec![Item::Static(s)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("pub static COUNTER: i32 = 0;"), "got: {}", output);
        assert!(!output.contains("mut"), "immutable static has no mut: {}", output);
    }

    #[test]
    fn test_generate_static_mutable() {
        // static mut int TICKS = 100; → pub static mut TICKS: i32 = 100;
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let s = Static {
            visibility: Visibility::Public,
            name: Ident::new("TICKS".to_string()),
            ty: Type::Primitive(PrimitiveType::I32),
            value: Expression::Literal(Literal::Int(100)),
            mutable: true,
            doc_comments: vec![],
        };
        let file = File {
            items: vec![Item::Static(s)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("pub static mut TICKS: i32 = 100;"), "got: {}", output);
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            is_variadic: false,
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
        assert_eq!(result, "ptr == Option::None");
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
        // Assignment is a statement-level place-expression: no wrapping parens
        // (a parenthesised `(ptr = …)` is not valid as a statement in Rust).
        assert_eq!(result, "ptr = Option::None");
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            is_variadic: false,
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
                    Statement::Break { label: None, value: None },
                ]),
            }],
            default: None,
        };
        let func = Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![stmt])),
            is_variadic: false,
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
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("read_config"),
            params: vec![],
            return_type: Some(Type::Fallible {
                ty: Box::new(Type::Ident(Ident::new("Config"))),
            }),
            body: Some(Block::empty()),
            is_variadic: false,
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
            base: None,
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
            base: None,
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
                        base: None,
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
                        base: None,
                    },
                ),
            ],
            base: None,
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
            base: None,
        };

        let result = codegen.generate_expression_string(&struct_init);
        // Auto type should be omitted in Rust (type inference)
        // But for struct initializers, we need the type name
        // This is a limitation - we'll need semantic analysis to resolve Auto types
        assert!(result.contains("x: 10"));
        assert!(result.contains("y: 20"));
    }

    #[test]
    fn test_generate_struct_initializer_functional_update() {
        let codegen = CodeGenerator::new(TargetLanguage::Rust);

        let struct_init = Expression::StructInit {
            ty: Type::Ident(Ident::new("Point")),
            fields: vec![(Ident::new("x"), Expression::Literal(Literal::Int(1)))],
            base: Some(Box::new(Expression::Ident(Ident::new("p")))),
        };

        let result = codegen.generate_expression_string(&struct_init);
        assert_eq!(result, "Point { x: 1, ..p }");
    }

    #[test]
    fn test_generate_struct_initializer_base_only() {
        let codegen = CodeGenerator::new(TargetLanguage::Rust);

        let struct_init = Expression::StructInit {
            ty: Type::Ident(Ident::new("Point")),
            fields: vec![],
            base: Some(Box::new(Expression::Ident(Ident::new("p")))),
        };

        let result = codegen.generate_expression_string(&struct_init);
        assert_eq!(result, "Point { ..p }");
    }

    #[test]
    fn test_generate_array_repeat() {
        let codegen = CodeGenerator::new(TargetLanguage::Rust);

        let repeat = Expression::ArrayRepeat {
            value: Box::new(Expression::Literal(Literal::Int(0))),
            count: Box::new(Expression::Literal(Literal::Int(16))),
        };

        let result = codegen.generate_expression_string(&repeat);
        assert_eq!(result, "[0; 16]");
    }
}
