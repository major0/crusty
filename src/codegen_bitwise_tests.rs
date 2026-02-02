// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Additional tests for codegen module to improve coverage - bitwise and modulo operations

#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::codegen::{CodeGenerator, TargetLanguage};

    #[test]
    fn test_generate_modulo_operation() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::Binary {
            op: BinaryOp::Mod,
            left: Box::new(Expression::Literal(Literal::Int(10))),
            right: Box::new(Expression::Literal(Literal::Int(3))),
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "(10 % 3)");
    }

    #[test]
    fn test_generate_bitwise_and() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::Binary {
            op: BinaryOp::BitAnd,
            left: Box::new(Expression::Literal(Literal::Int(15))),
            right: Box::new(Expression::Literal(Literal::Int(7))),
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "(15 & 7)");
    }

    #[test]
    fn test_generate_bitwise_or() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::Binary {
            op: BinaryOp::BitOr,
            left: Box::new(Expression::Literal(Literal::Int(8))),
            right: Box::new(Expression::Literal(Literal::Int(4))),
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "(8 | 4)");
    }

    #[test]
    fn test_generate_bitwise_xor() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::Binary {
            op: BinaryOp::BitXor,
            left: Box::new(Expression::Literal(Literal::Int(12))),
            right: Box::new(Expression::Literal(Literal::Int(5))),
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "(12 ^ 5)");
    }

    #[test]
    fn test_generate_left_shift() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::Binary {
            op: BinaryOp::Shl,
            left: Box::new(Expression::Literal(Literal::Int(1))),
            right: Box::new(Expression::Literal(Literal::Int(3))),
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "(1 << 3)");
    }

    #[test]
    fn test_generate_right_shift() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::Binary {
            op: BinaryOp::Shr,
            left: Box::new(Expression::Literal(Literal::Int(16))),
            right: Box::new(Expression::Literal(Literal::Int(2))),
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "(16 >> 2)");
    }

    #[test]
    fn test_generate_slice_type() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let slice_type = Type::Slice {
            ty: Box::new(Type::Primitive(PrimitiveType::I32)),
        };
        let result = gen.generate_type_string(&slice_type);
        assert_eq!(result, "[i32]");
    }

    #[test]
    fn test_generate_function_type() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let func_type = Type::Function {
            params: vec![
                Type::Primitive(PrimitiveType::I32),
                Type::Primitive(PrimitiveType::I32),
            ],
            return_type: Box::new(Type::Primitive(PrimitiveType::I32)),
        };
        let result = gen.generate_type_string(&func_type);
        assert_eq!(result, "fn(i32, i32) -> i32");
    }

    #[test]
    fn test_generate_auto_type() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let auto_type = Type::Auto;
        let result = gen.generate_type_string(&auto_type);
        assert_eq!(result, "_");
    }

    #[test]
    fn test_generate_for_in_loop() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let stmt = Statement::ForIn {
            label: None,
            var: Ident::new("i"),
            iter: Expression::Range {
                start: Some(Box::new(Expression::Literal(Literal::Int(0)))),
                end: Some(Box::new(Expression::Literal(Literal::Int(10)))),
                inclusive: false,
            },
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
        assert!(output.contains("for i in 0..10"));
    }

    #[test]
    fn test_generate_for_in_loop_with_label() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let stmt = Statement::ForIn {
            label: Some(Ident::new("outer")),
            var: Ident::new("i"),
            iter: Expression::Ident(Ident::new("items")),
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
        assert!(output.contains("'outer: for i in items"));
    }

    #[test]
    fn test_generate_array_without_size() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let array_type = Type::Array {
            ty: Box::new(Type::Primitive(PrimitiveType::I32)),
            size: None,
        };
        let result = gen.generate_type_string(&array_type);
        assert_eq!(result, "[i32]");
    }

    #[test]
    fn test_generate_mutable_pointer() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let ptr_type = Type::Pointer {
            ty: Box::new(Type::Primitive(PrimitiveType::I32)),
            mutable: true,
        };
        let result = gen.generate_type_string(&ptr_type);
        assert_eq!(result, "*mut i32");
    }

    #[test]
    fn test_generate_const_pointer() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let ptr_type = Type::Pointer {
            ty: Box::new(Type::Primitive(PrimitiveType::I32)),
            mutable: false,
        };
        let result = gen.generate_type_string(&ptr_type);
        assert_eq!(result, "*const i32");
    }

    #[test]
    fn test_generate_string_literal() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::Literal(Literal::String("hello world".to_string()));
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "\"hello world\"");
    }

    #[test]
    fn test_generate_char_literal() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::Literal(Literal::Char('a'));
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "'a'");
    }

    #[test]
    fn test_generate_float_literal() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::Literal(Literal::Float(3.14));
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "3.14");
    }

    #[test]
    fn test_generate_bool_true_literal() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::Literal(Literal::Bool(true));
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "true");
    }

    #[test]
    fn test_generate_bool_false_literal() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::Literal(Literal::Bool(false));
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "false");
    }

    #[test]
    fn test_generate_field_access() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::FieldAccess {
            expr: Box::new(Expression::Ident(Ident::new("point"))),
            field: Ident::new("x"),
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "point.x");
    }

    #[test]
    fn test_generate_index_expression() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::Index {
            expr: Box::new(Expression::Ident(Ident::new("arr"))),
            index: Box::new(Expression::Literal(Literal::Int(0))),
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "arr[0]");
    }

    #[test]
    fn test_generate_ternary_expression() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::Ternary {
            condition: Box::new(Expression::Literal(Literal::Bool(true))),
            then_expr: Box::new(Expression::Literal(Literal::Int(1))),
            else_expr: Box::new(Expression::Literal(Literal::Int(2))),
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "if true { 1 } else { 2 }");
    }

    #[test]
    fn test_generate_macro_call_expression() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::MacroCall {
            name: Ident::new("println"),
            args: vec![Token {
                kind: TokenKind::Literal,
                text: "\"hello\"".to_string(),
            }],
        };
        let result = gen.generate_expression_string(&expr);
        assert_eq!(result, "println!(\"hello\")");
    }

    #[test]
    fn test_generate_rust_block_expression() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        let expr = Expression::RustBlock {
            tokens: vec![
                Token {
                    kind: TokenKind::Keyword,
                    text: "let".to_string(),
                },
                Token {
                    kind: TokenKind::Ident,
                    text: "x".to_string(),
                },
            ],
        };
        let result = gen.generate_expression_string(&expr);
        assert!(result.contains("let"));
        assert!(result.contains("x"));
    }
}
