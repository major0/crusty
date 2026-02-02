// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Additional tests for Crusty target language code generation

#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::codegen::{CodeGenerator, TargetLanguage};

    #[test]
    fn test_crusty_for_in_loop() {
        let mut gen = CodeGenerator::new(TargetLanguage::Crusty);
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
            return_type: Some(Type::Primitive(PrimitiveType::Void)),
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
    fn test_crusty_for_in_with_label() {
        let mut gen = CodeGenerator::new(TargetLanguage::Crusty);
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
            return_type: Some(Type::Primitive(PrimitiveType::Void)),
            body: Block::new(vec![stmt]),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains(".outer: for i in items"));
    }

    #[test]
    fn test_crusty_switch_statement() {
        let mut gen = CodeGenerator::new(TargetLanguage::Crusty);
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
            return_type: Some(Type::Primitive(PrimitiveType::Void)),
            body: Block::new(vec![stmt]),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("switch (x) {"));
        assert!(output.contains("case 1:"));
        assert!(output.contains("case 2:"));
    }

    #[test]
    fn test_crusty_switch_with_default() {
        let mut gen = CodeGenerator::new(TargetLanguage::Crusty);
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
            return_type: Some(Type::Primitive(PrimitiveType::Void)),
            body: Block::new(vec![stmt]),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("switch (x) {"));
        assert!(output.contains("case 1:"));
        assert!(output.contains("default:"));
    }

    #[test]
    fn test_crusty_switch_with_multiple_values() {
        let mut gen = CodeGenerator::new(TargetLanguage::Crusty);
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
            return_type: Some(Type::Primitive(PrimitiveType::Void)),
            body: Block::new(vec![stmt]),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("switch (x) {"));
        assert!(output.contains("case 1, 2, 3:"));
    }

    #[test]
    fn test_crusty_break_with_label() {
        let mut gen = CodeGenerator::new(TargetLanguage::Crusty);
        let stmt = Statement::Break(Some(Ident::new("outer")));
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: Some(Type::Primitive(PrimitiveType::Void)),
            body: Block::new(vec![stmt]),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("break .outer;"));
    }

    #[test]
    fn test_crusty_continue_with_label() {
        let mut gen = CodeGenerator::new(TargetLanguage::Crusty);
        let stmt = Statement::Continue(Some(Ident::new("outer")));
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: Some(Type::Primitive(PrimitiveType::Void)),
            body: Block::new(vec![stmt]),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("continue .outer;"));
    }

    #[test]
    fn test_crusty_for_loop_with_var_init() {
        let mut gen = CodeGenerator::new(TargetLanguage::Crusty);
        let stmt = Statement::For {
            label: None,
            init: Box::new(Statement::Var {
                name: Ident::new("i"),
                ty: Some(Type::Primitive(PrimitiveType::Int)),
                init: Some(Expression::Literal(Literal::Int(0))),
            }),
            condition: Expression::Binary {
                op: BinaryOp::Lt,
                left: Box::new(Expression::Ident(Ident::new("i"))),
                right: Box::new(Expression::Literal(Literal::Int(10))),
            },
            increment: Expression::Binary {
                op: BinaryOp::AddAssign,
                left: Box::new(Expression::Ident(Ident::new("i"))),
                right: Box::new(Expression::Literal(Literal::Int(1))),
            },
            body: Block::empty(),
        };
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: Some(Type::Primitive(PrimitiveType::Void)),
            body: Block::new(vec![stmt]),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("for (var int i = 0;"));
    }

    #[test]
    fn test_crusty_primitive_types() {
        let gen = CodeGenerator::new(TargetLanguage::Crusty);
        assert_eq!(
            gen.generate_type_string(&Type::Primitive(PrimitiveType::Int)),
            "int"
        );
        assert_eq!(
            gen.generate_type_string(&Type::Primitive(PrimitiveType::Float)),
            "float"
        );
        assert_eq!(
            gen.generate_type_string(&Type::Primitive(PrimitiveType::Void)),
            "void"
        );
        assert_eq!(
            gen.generate_type_string(&Type::Primitive(PrimitiveType::Bool)),
            "bool"
        );
        assert_eq!(
            gen.generate_type_string(&Type::Primitive(PrimitiveType::Char)),
            "char"
        );
    }

    #[test]
    fn test_crusty_if_else_statement() {
        let mut gen = CodeGenerator::new(TargetLanguage::Crusty);
        let stmt = Statement::If {
            condition: Expression::Literal(Literal::Bool(true)),
            then_block: Block::empty(),
            else_block: Some(Block::empty()),
        };
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: Some(Type::Primitive(PrimitiveType::Void)),
            body: Block::new(vec![stmt]),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("if (true)"));
        assert!(output.contains("else"));
    }

    #[test]
    fn test_crusty_function_with_params() {
        let mut gen = CodeGenerator::new(TargetLanguage::Crusty);
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("add"),
            params: vec![
                Param {
                    name: Ident::new("a"),
                    ty: Type::Primitive(PrimitiveType::Int),
                },
                Param {
                    name: Ident::new("b"),
                    ty: Type::Primitive(PrimitiveType::Int),
                },
            ],
            return_type: Some(Type::Primitive(PrimitiveType::Int)),
            body: Block::empty(),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("int add(int a, int b)"));
    }

    #[test]
    fn test_crusty_return_statement() {
        let mut gen = CodeGenerator::new(TargetLanguage::Crusty);
        let stmt = Statement::Return(Some(Expression::Literal(Literal::Int(42))));
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: Some(Type::Primitive(PrimitiveType::Int)),
            body: Block::new(vec![stmt]),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("return 42;"));
    }

    #[test]
    fn test_crusty_let_without_type() {
        let mut gen = CodeGenerator::new(TargetLanguage::Crusty);
        let stmt = Statement::Let {
            name: Ident::new("x"),
            ty: None,
            init: Some(Expression::Literal(Literal::Int(42))),
            mutable: false,
        };
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: Some(Type::Primitive(PrimitiveType::Void)),
            body: Block::new(vec![stmt]),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("let x = 42;"));
    }

    #[test]
    fn test_crusty_var_without_type() {
        let mut gen = CodeGenerator::new(TargetLanguage::Crusty);
        let stmt = Statement::Var {
            name: Ident::new("x"),
            ty: None,
            init: Some(Expression::Literal(Literal::Int(42))),
        };
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: Some(Type::Primitive(PrimitiveType::Void)),
            body: Block::new(vec![stmt]),
            doc_comments: vec![],
            attributes: vec![],
        };
        let file = File {
            items: vec![Item::Function(func)],
            doc_comments: vec![],
        };
        let output = gen.generate(&file);
        assert!(output.contains("var x = 42;"));
    }
}
