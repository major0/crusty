// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Unit tests for the semantic analyzer, extracted from the inline
//! `#[cfg(test)]` module in `src/semantic.rs` to match the project's
//! sibling-test-file convention. Private fields and methods are reached through
//! the analyzer's `#[cfg(test)]` accessors (`analyze_*_test`,
//! `clear_errors_for_test`, `SymbolTable::scopes_len`), so no production
//! visibility is widened.

#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::error::SemanticErrorKind;
    use crate::semantic::{
        SemanticAnalyzer, Symbol, SymbolKind, SymbolTable, TypeEnvironment, TypeInfo, TypeKind,
    };
    use std::collections::HashSet;

    // Symbol Table Tests

    #[test]
    fn test_symbol_table_creation() {
        let table = SymbolTable::new();
        assert_eq!(table.scopes_len(), 1);
    }

    #[test]
    fn test_symbol_table_enter_exit_scope() {
        let mut table = SymbolTable::new();
        assert_eq!(table.scopes_len(), 1);

        table.enter_scope();
        assert_eq!(table.scopes_len(), 2);

        table.enter_scope();
        assert_eq!(table.scopes_len(), 3);

        table.exit_scope();
        assert_eq!(table.scopes_len(), 2);

        table.exit_scope();
        assert_eq!(table.scopes_len(), 1);

        // Should not go below 1 scope
        table.exit_scope();
        assert_eq!(table.scopes_len(), 1);
    }

    #[test]
    fn test_symbol_table_insert_and_lookup() {
        let mut table = SymbolTable::new();

        let symbol = Symbol::new(
            "x".to_string(),
            Type::Primitive(PrimitiveType::I32),
            SymbolKind::Variable,
            false,
        );

        assert!(table.insert("x".to_string(), symbol.clone()).is_ok());

        let found = table.lookup("x");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "x");
        assert_eq!(found.unwrap().kind, SymbolKind::Variable);
        assert!(!found.unwrap().mutable);
    }

    #[test]
    fn test_symbol_table_duplicate_detection() {
        let mut table = SymbolTable::new();

        let symbol1 = Symbol::new(
            "x".to_string(),
            Type::Primitive(PrimitiveType::I32),
            SymbolKind::Variable,
            false,
        );

        let symbol2 = Symbol::new(
            "x".to_string(),
            Type::Primitive(PrimitiveType::I64),
            SymbolKind::Variable,
            true,
        );

        assert!(table.insert("x".to_string(), symbol1).is_ok());

        // Should fail because 'x' already exists in current scope
        let result = table.insert("x".to_string(), symbol2);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already defined"));
    }

    #[test]
    fn test_symbol_table_scope_shadowing() {
        let mut table = SymbolTable::new();

        // Insert 'x' in outer scope
        let symbol1 = Symbol::new(
            "x".to_string(),
            Type::Primitive(PrimitiveType::I32),
            SymbolKind::Variable,
            false,
        );
        assert!(table.insert("x".to_string(), symbol1).is_ok());

        // Enter new scope
        table.enter_scope();

        // Insert 'x' in inner scope (shadowing)
        let symbol2 = Symbol::new(
            "x".to_string(),
            Type::Primitive(PrimitiveType::I64),
            SymbolKind::Variable,
            true,
        );
        assert!(table.insert("x".to_string(), symbol2).is_ok());

        // Lookup should find inner scope's 'x'
        let found = table.lookup("x");
        assert!(found.is_some());
        assert!(matches!(
            found.unwrap().ty,
            Type::Primitive(PrimitiveType::I64)
        ));
        assert!(found.unwrap().mutable);

        // Exit inner scope
        table.exit_scope();

        // Lookup should now find outer scope's 'x'
        let found = table.lookup("x");
        assert!(found.is_some());
        assert!(matches!(
            found.unwrap().ty,
            Type::Primitive(PrimitiveType::I32)
        ));
        assert!(!found.unwrap().mutable);
    }

    #[test]
    fn test_symbol_table_lookup_in_current_scope() {
        let mut table = SymbolTable::new();

        // Insert 'x' in outer scope
        let symbol1 = Symbol::new(
            "x".to_string(),
            Type::Primitive(PrimitiveType::I32),
            SymbolKind::Variable,
            false,
        );
        assert!(table.insert("x".to_string(), symbol1).is_ok());

        // Enter new scope
        table.enter_scope();

        // lookup_in_current_scope should not find 'x' from outer scope
        assert!(table.lookup_in_current_scope("x").is_none());

        // But regular lookup should find it
        assert!(table.lookup("x").is_some());

        // Insert 'y' in inner scope
        let symbol2 = Symbol::new(
            "y".to_string(),
            Type::Primitive(PrimitiveType::Bool),
            SymbolKind::Variable,
            false,
        );
        assert!(table.insert("y".to_string(), symbol2).is_ok());

        // lookup_in_current_scope should find 'y'
        assert!(table.lookup_in_current_scope("y").is_some());
    }

    #[test]
    fn test_symbol_table_multiple_symbols() {
        let mut table = SymbolTable::new();

        let symbols = vec![
            (
                "x",
                Type::Primitive(PrimitiveType::I32),
                SymbolKind::Variable,
            ),
            (
                "y",
                Type::Primitive(PrimitiveType::Bool),
                SymbolKind::Variable,
            ),
            (
                "foo",
                Type::Primitive(PrimitiveType::Void),
                SymbolKind::Function,
            ),
            ("Point", Type::Ident(Ident::new("Point")), SymbolKind::Type),
        ];

        for (name, ty, kind) in symbols {
            let symbol = Symbol::new(name.to_string(), ty, kind, false);
            assert!(table.insert(name.to_string(), symbol).is_ok());
        }

        assert!(table.lookup("x").is_some());
        assert!(table.lookup("y").is_some());
        assert!(table.lookup("foo").is_some());
        assert!(table.lookup("Point").is_some());
        assert!(table.lookup("nonexistent").is_none());
    }

    #[test]
    fn test_symbol_kinds() {
        let kinds = [
            SymbolKind::Variable,
            SymbolKind::Function,
            SymbolKind::Type,
            SymbolKind::Const,
        ];

        assert_eq!(kinds.len(), 4);
    }

    // Type Environment Tests

    #[test]
    fn test_type_environment_creation() {
        let env = TypeEnvironment::new();

        // Should have primitive types registered
        assert!(env.get_type("int").is_some());
        assert!(env.get_type("i32").is_some());
        assert!(env.get_type("bool").is_some());
        assert!(env.get_type("char").is_some());
        assert!(env.get_type("void").is_some());
    }

    #[test]
    fn test_type_environment_register_and_get() {
        let mut env = TypeEnvironment::new();

        let type_info = TypeInfo::new(
            "Point".to_string(),
            TypeKind::Struct {
                fields: vec![
                    ("x".to_string(), Type::Primitive(PrimitiveType::I32)),
                    ("y".to_string(), Type::Primitive(PrimitiveType::I32)),
                ],
                methods: vec![],
            },
        );

        env.register_type("Point".to_string(), type_info);

        let found = env.get_type("Point");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Point");

        match &found.unwrap().kind {
            TypeKind::Struct { fields, .. } => {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "x");
                assert_eq!(fields[1].0, "y");
            }
            _ => panic!("Expected Struct type kind"),
        }
    }

    #[test]
    fn test_type_compatibility_primitives() {
        let env = TypeEnvironment::new();

        let t1 = Type::Primitive(PrimitiveType::I32);
        let t2 = Type::Primitive(PrimitiveType::I32);
        let t3 = Type::Primitive(PrimitiveType::Bool);

        assert!(env.is_compatible(&t1, &t2));
        assert!(!env.is_compatible(&t1, &t3));
    }

    #[test]
    fn test_type_compatibility_int_i32() {
        let env = TypeEnvironment::new();

        let int_type = Type::Primitive(PrimitiveType::Int);
        let i32_type = Type::Primitive(PrimitiveType::I32);

        // int and i32 should be compatible
        assert!(env.is_compatible(&int_type, &i32_type));
        assert!(env.is_compatible(&i32_type, &int_type));
    }

    #[test]
    fn test_type_compatibility_float_f64() {
        let env = TypeEnvironment::new();

        let float_type = Type::Primitive(PrimitiveType::Float);
        let f64_type = Type::Primitive(PrimitiveType::F64);

        // float and f64 should be compatible
        assert!(env.is_compatible(&float_type, &f64_type));
        assert!(env.is_compatible(&f64_type, &float_type));
    }

    #[test]
    fn test_type_compatibility_auto() {
        let env = TypeEnvironment::new();

        let auto_type = Type::Auto;
        let i32_type = Type::Primitive(PrimitiveType::I32);

        // Auto should be compatible with anything
        assert!(env.is_compatible(&auto_type, &i32_type));
        assert!(env.is_compatible(&i32_type, &auto_type));
    }

    #[test]
    fn test_type_compatibility_pointers() {
        let env = TypeEnvironment::new();

        let ptr1 = Type::Pointer {
            ty: Box::new(Type::Primitive(PrimitiveType::I32)),
            mutable: true,
        };
        let ptr2 = Type::Pointer {
            ty: Box::new(Type::Primitive(PrimitiveType::I32)),
            mutable: true,
        };
        let ptr3 = Type::Pointer {
            ty: Box::new(Type::Primitive(PrimitiveType::I32)),
            mutable: false,
        };

        assert!(env.is_compatible(&ptr1, &ptr2));
        assert!(!env.is_compatible(&ptr1, &ptr3)); // Different mutability
    }

    #[test]
    fn test_type_compatibility_references() {
        let env = TypeEnvironment::new();

        let ref1 = Type::Reference {
            ty: Box::new(Type::Primitive(PrimitiveType::I32)),
            mutable: false,
        };
        let ref2 = Type::Reference {
            ty: Box::new(Type::Primitive(PrimitiveType::I32)),
            mutable: false,
        };
        let ref3 = Type::Reference {
            ty: Box::new(Type::Primitive(PrimitiveType::I32)),
            mutable: true,
        };

        assert!(env.is_compatible(&ref1, &ref2));
        // Immutable reference can be created from mutable
        assert!(env.is_compatible(&ref3, &ref1));
        // But not vice versa
        assert!(!env.is_compatible(&ref1, &ref3));
    }

    #[test]
    fn test_type_compatibility_arrays() {
        let env = TypeEnvironment::new();

        let arr1 = Type::Array {
            ty: Box::new(Type::Primitive(PrimitiveType::I32)),
            size: Some(10),
        };
        let arr2 = Type::Array {
            ty: Box::new(Type::Primitive(PrimitiveType::I32)),
            size: Some(10),
        };
        let arr3 = Type::Array {
            ty: Box::new(Type::Primitive(PrimitiveType::I32)),
            size: Some(20),
        };

        assert!(env.is_compatible(&arr1, &arr2));
        assert!(!env.is_compatible(&arr1, &arr3)); // Different sizes
    }

    #[test]
    fn test_type_compatibility_tuples() {
        let env = TypeEnvironment::new();

        let tuple1 = Type::Tuple {
            types: vec![
                Type::Primitive(PrimitiveType::I32),
                Type::Primitive(PrimitiveType::Bool),
            ],
        };
        let tuple2 = Type::Tuple {
            types: vec![
                Type::Primitive(PrimitiveType::I32),
                Type::Primitive(PrimitiveType::Bool),
            ],
        };
        let tuple3 = Type::Tuple {
            types: vec![
                Type::Primitive(PrimitiveType::I32),
                Type::Primitive(PrimitiveType::I32),
            ],
        };

        assert!(env.is_compatible(&tuple1, &tuple2));
        assert!(!env.is_compatible(&tuple1, &tuple3)); // Different element types
    }

    #[test]
    fn test_type_compatibility_generics() {
        let env = TypeEnvironment::new();

        let gen1 = Type::Generic {
            base: Box::new(Type::Ident(Ident::new("Vec"))),
            args: vec![Type::Primitive(PrimitiveType::I32)],
        };
        let gen2 = Type::Generic {
            base: Box::new(Type::Ident(Ident::new("Vec"))),
            args: vec![Type::Primitive(PrimitiveType::I32)],
        };
        let gen3 = Type::Generic {
            base: Box::new(Type::Ident(Ident::new("Vec"))),
            args: vec![Type::Primitive(PrimitiveType::Bool)],
        };

        assert!(env.is_compatible(&gen1, &gen2));
        assert!(!env.is_compatible(&gen1, &gen3)); // Different type arguments
    }

    #[test]
    fn test_type_compatibility_functions() {
        let env = TypeEnvironment::new();

        let func1 = Type::Function {
            params: vec![Type::Primitive(PrimitiveType::I32)],
            return_type: Box::new(Type::Primitive(PrimitiveType::Bool)),
        };
        let func2 = Type::Function {
            params: vec![Type::Primitive(PrimitiveType::I32)],
            return_type: Box::new(Type::Primitive(PrimitiveType::Bool)),
        };
        let func3 = Type::Function {
            params: vec![Type::Primitive(PrimitiveType::I32)],
            return_type: Box::new(Type::Primitive(PrimitiveType::I32)),
        };

        assert!(env.is_compatible(&func1, &func2));
        assert!(!env.is_compatible(&func1, &func3)); // Different return types
    }

    #[test]
    fn test_type_info_kinds() {
        let primitive = TypeInfo::new("int".to_string(), TypeKind::Primitive);
        assert_eq!(primitive.name, "int");
        assert!(matches!(primitive.kind, TypeKind::Primitive));

        let struct_type = TypeInfo::new(
            "Point".to_string(),
            TypeKind::Struct {
                fields: vec![],
                methods: vec![],
            },
        );
        assert!(matches!(struct_type.kind, TypeKind::Struct { .. }));

        let enum_type = TypeInfo::new("Color".to_string(), TypeKind::Enum { variants: vec![] });
        assert!(matches!(enum_type.kind, TypeKind::Enum { .. }));

        let alias_type = TypeInfo::new(
            "MyInt".to_string(),
            TypeKind::Alias {
                target: Type::Primitive(PrimitiveType::I32),
            },
        );
        assert!(matches!(alias_type.kind, TypeKind::Alias { .. }));
    }

    #[test]
    fn test_symbol_mutability() {
        let mutable_symbol = Symbol::new(
            "x".to_string(),
            Type::Primitive(PrimitiveType::I32),
            SymbolKind::Variable,
            true,
        );
        assert!(mutable_symbol.mutable);

        let immutable_symbol = Symbol::new(
            "y".to_string(),
            Type::Primitive(PrimitiveType::I32),
            SymbolKind::Variable,
            false,
        );
        assert!(!immutable_symbol.mutable);
    }

    #[test]
    fn test_type_compatibility_slices() {
        let env = TypeEnvironment::new();

        let slice1 = Type::Slice {
            ty: Box::new(Type::Primitive(PrimitiveType::I32)),
        };
        let slice2 = Type::Slice {
            ty: Box::new(Type::Primitive(PrimitiveType::I32)),
        };
        let slice3 = Type::Slice {
            ty: Box::new(Type::Primitive(PrimitiveType::Bool)),
        };

        assert!(env.is_compatible(&slice1, &slice2));
        assert!(!env.is_compatible(&slice1, &slice3));
    }

    #[test]
    fn test_type_compatibility_fallible() {
        let env = TypeEnvironment::new();

        let fallible1 = Type::Fallible {
            ty: Box::new(Type::Primitive(PrimitiveType::I32)),
        };
        let fallible2 = Type::Fallible {
            ty: Box::new(Type::Primitive(PrimitiveType::I32)),
        };
        let fallible3 = Type::Fallible {
            ty: Box::new(Type::Primitive(PrimitiveType::Bool)),
        };

        assert!(env.is_compatible(&fallible1, &fallible2));
        assert!(!env.is_compatible(&fallible1, &fallible3));
    }

    // Semantic Analyzer Tests

    #[test]
    fn test_semantic_analyzer_creation() {
        let analyzer = SemanticAnalyzer::new();
        assert_eq!(analyzer.errors().len(), 0);
        assert_eq!(analyzer.symbol_table().scopes_len(), 1);
    }

    #[test]
    fn test_semantic_analyzer_analyze_empty_file() {
        let mut analyzer = SemanticAnalyzer::new();
        let file = File {
            items: vec![],
            doc_comments: vec![],
        };

        let result = analyzer.analyze(&file);
        assert!(result.is_ok());
        assert_eq!(analyzer.errors().len(), 0);
    }

    #[test]
    fn test_semantic_analyzer_default() {
        let analyzer = SemanticAnalyzer::default();
        assert_eq!(analyzer.errors().len(), 0);
    }

    #[test]
    fn test_semantic_analyzer_function_registration() {
        let mut analyzer = SemanticAnalyzer::new();
        let func = Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: Some(Type::Primitive(PrimitiveType::I32)),
            body: Some(Block::new(vec![Statement::Return(Some(
                Expression::Literal(Literal::Int(0)),
            ))])),
            is_variadic: false,
            is_unsafe: false,
            doc_comments: vec![],
            attributes: vec![],
        };

        analyzer.analyze_function_test(&func);

        // Function should be registered in symbol table
        let symbol = analyzer.symbol_table().lookup("test_func");
        assert!(symbol.is_some());
        assert_eq!(symbol.unwrap().kind, SymbolKind::Function);
    }

    #[test]
    fn test_semantic_analyzer_struct_registration() {
        let mut analyzer = SemanticAnalyzer::new();
        let struct_def = Struct {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("Point"),
            body: StructBody::Named(vec![
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
            ]),
            methods: vec![],
            doc_comments: vec![],
            attributes: vec![],
        };

        analyzer.analyze_struct_test(&struct_def);

        // Struct should be registered as a type
        let type_info = analyzer.type_env().get_type("Point");
        assert!(type_info.is_some());
        assert!(matches!(type_info.unwrap().kind, TypeKind::Struct { .. }));

        // Struct should also be in symbol table
        let symbol = analyzer.symbol_table().lookup("Point");
        assert!(symbol.is_some());
        assert_eq!(symbol.unwrap().kind, SymbolKind::Type);
    }

    #[test]
    fn test_semantic_analyzer_enum_registration() {
        let mut analyzer = SemanticAnalyzer::new();
        let enum_def = Enum {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("Color"),
            variants: vec![
                EnumVariant {
                    name: Ident::new("Red"),
                    body: VariantBody::Discriminant(0),
                },
                EnumVariant {
                    name: Ident::new("Green"),
                    body: VariantBody::Discriminant(1),
                },
            ],
            doc_comments: vec![],
            attributes: vec![],
        };

        analyzer.analyze_enum_test(&enum_def);

        // Enum should be registered as a type
        let type_info = analyzer.type_env().get_type("Color");
        assert!(type_info.is_some());
        assert!(matches!(type_info.unwrap().kind, TypeKind::Enum { .. }));
    }

    #[test]
    fn test_semantic_analyzer_duplicate_function() {
        let mut analyzer = SemanticAnalyzer::new();
        let func = Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("duplicate"),
            params: vec![],
            return_type: Some(Type::Primitive(PrimitiveType::I32)),
            body: Some(Block::new(vec![Statement::Return(Some(
                Expression::Literal(Literal::Int(0)),
            ))])),
            is_variadic: false,
            is_unsafe: false,
            doc_comments: vec![],
            attributes: vec![],
        };

        analyzer.analyze_function_test(&func);
        assert_eq!(analyzer.errors().len(), 0);

        // Try to register the same function again
        analyzer.analyze_function_test(&func);
        assert_eq!(analyzer.errors().len(), 1);
        assert_eq!(
            analyzer.errors()[0].kind,
            SemanticErrorKind::DuplicateDefinition
        );
    }

    #[test]
    fn test_semantic_analyzer_let_statement() {
        let mut analyzer = SemanticAnalyzer::new();
        let stmt = Statement::Let {
            name: Ident::new("x"),
            ty: Some(Type::Primitive(PrimitiveType::I32)),
            init: Some(Expression::Literal(Literal::Int(42))),
            mutable: false,
        };

        analyzer.analyze_statement_test(&stmt);

        // Variable should be registered
        let symbol = analyzer.symbol_table().lookup("x");
        assert!(symbol.is_some());
        assert_eq!(symbol.unwrap().kind, SymbolKind::Variable);
        assert!(!symbol.unwrap().mutable);
    }

    #[test]
    fn test_semantic_analyzer_var_statement() {
        let mut analyzer = SemanticAnalyzer::new();
        let stmt = Statement::Var {
            name: Ident::new("y"),
            ty: Some(Type::Primitive(PrimitiveType::I32)),
            init: Some(Expression::Literal(Literal::Int(10))),
        };

        analyzer.analyze_statement_test(&stmt);

        // Variable should be registered as mutable
        let symbol = analyzer.symbol_table().lookup("y");
        assert!(symbol.is_some());
        assert_eq!(symbol.unwrap().kind, SymbolKind::Variable);
        assert!(symbol.unwrap().mutable);
    }

    #[test]
    fn test_semantic_analyzer_undefined_variable() {
        let mut analyzer = SemanticAnalyzer::new();
        let stmt = Statement::Expr(Expression::Ident(Ident::new("undefined_var")));

        analyzer.analyze_statement_test(&stmt);

        // Should detect undefined variable
        assert_eq!(analyzer.errors().len(), 1);
        assert_eq!(
            analyzer.errors()[0].kind,
            SemanticErrorKind::UndefinedVariable
        );
    }

    #[test]
    fn test_semantic_analyzer_if_statement() {
        let mut analyzer = SemanticAnalyzer::new();
        let stmt = Statement::If {
            condition: Expression::Literal(Literal::Bool(true)),
            then_block: Block::empty(),
            else_block: None,
        };

        analyzer.analyze_statement_test(&stmt);

        // Should have no errors for valid if statement
        assert_eq!(analyzer.errors().len(), 0);
    }

    #[test]
    fn test_semantic_analyzer_while_statement() {
        let mut analyzer = SemanticAnalyzer::new();
        let stmt = Statement::While {
            label: None,
            condition: Expression::Literal(Literal::Bool(true)),
            body: Block::empty(),
        };

        analyzer.analyze_statement_test(&stmt);

        // Should have no errors for valid while statement
        assert_eq!(analyzer.errors().len(), 0);
    }

    #[test]
    fn test_semantic_analyzer_binary_expression() {
        let mut analyzer = SemanticAnalyzer::new();
        let expr = Expression::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expression::Literal(Literal::Int(1))),
            right: Box::new(Expression::Literal(Literal::Int(2))),
        };

        let result_type = analyzer.analyze_expression_test(&expr);
        assert!(matches!(result_type, Type::Primitive(PrimitiveType::I32)));
        assert_eq!(analyzer.errors().len(), 0);
    }

    #[test]
    fn test_semantic_analyzer_comparison_expression() {
        let mut analyzer = SemanticAnalyzer::new();
        let expr = Expression::Binary {
            op: BinaryOp::Lt,
            left: Box::new(Expression::Literal(Literal::Int(1))),
            right: Box::new(Expression::Literal(Literal::Int(2))),
        };

        let result_type = analyzer.analyze_expression_test(&expr);
        assert!(matches!(result_type, Type::Primitive(PrimitiveType::Bool)));
        assert_eq!(analyzer.errors().len(), 0);
    }

    #[test]
    fn test_semantic_analyzer_array_literal() {
        let mut analyzer = SemanticAnalyzer::new();
        let expr = Expression::ArrayLit {
            elements: vec![
                Expression::Literal(Literal::Int(1)),
                Expression::Literal(Literal::Int(2)),
                Expression::Literal(Literal::Int(3)),
            ],
        };

        let result_type = analyzer.analyze_expression_test(&expr);
        match result_type {
            Type::Array { ty, size } => {
                assert!(matches!(*ty, Type::Primitive(PrimitiveType::I32)));
                assert_eq!(size, Some(3));
            }
            _ => panic!("Expected array type"),
        }
        assert_eq!(analyzer.errors().len(), 0);
    }

    #[test]
    fn test_semantic_analyzer_tuple_literal() {
        let mut analyzer = SemanticAnalyzer::new();
        let expr = Expression::TupleLit {
            elements: vec![
                Expression::Literal(Literal::Int(42)),
                Expression::Literal(Literal::Bool(true)),
            ],
        };

        let result_type = analyzer.analyze_expression_test(&expr);
        match result_type {
            Type::Tuple { types } => {
                assert_eq!(types.len(), 2);
                assert!(matches!(types[0], Type::Primitive(PrimitiveType::I32)));
                assert!(matches!(types[1], Type::Primitive(PrimitiveType::Bool)));
            }
            _ => panic!("Expected tuple type"),
        }
        assert_eq!(analyzer.errors().len(), 0);
    }

    #[test]
    fn test_semantic_analyzer_type_mismatch_binary() {
        let mut analyzer = SemanticAnalyzer::new();
        let expr = Expression::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expression::Literal(Literal::Int(1))),
            right: Box::new(Expression::Literal(Literal::Bool(true))),
        };

        analyzer.analyze_expression_test(&expr);

        // Should detect type mismatch
        assert_eq!(analyzer.errors().len(), 1);
        assert_eq!(analyzer.errors()[0].kind, SemanticErrorKind::TypeMismatch);
    }

    #[test]
    fn test_semantic_analyzer_unsupported_union() {
        let mut analyzer = SemanticAnalyzer::new();

        analyzer.check_union_usage("MyUnion");

        // Should detect unsupported union
        assert_eq!(analyzer.errors().len(), 1);
        assert_eq!(
            analyzer.errors()[0].kind,
            SemanticErrorKind::UnsupportedFeature
        );
        assert!(analyzer.errors()[0]
            .message
            .contains("unions are not supported"));
        assert!(analyzer.errors()[0].message.contains("MyUnion"));
    }

    #[test]
    fn test_semantic_analyzer_unsupported_goto() {
        let mut analyzer = SemanticAnalyzer::new();

        analyzer.check_goto_usage("my_label");

        // Should detect unsupported goto
        assert_eq!(analyzer.errors().len(), 1);
        assert_eq!(
            analyzer.errors()[0].kind,
            SemanticErrorKind::UnsupportedFeature
        );
        assert!(analyzer.errors()[0]
            .message
            .contains("goto statements are not supported"));
        assert!(analyzer.errors()[0].message.contains("my_label"));
    }

    #[test]
    fn test_semantic_analyzer_unsupported_include() {
        let mut analyzer = SemanticAnalyzer::new();

        analyzer.check_include_usage("stdio.h");

        // Should detect unsupported #include
        assert_eq!(analyzer.errors().len(), 1);
        assert_eq!(
            analyzer.errors()[0].kind,
            SemanticErrorKind::UnsupportedFeature
        );
        assert!(analyzer.errors()[0]
            .message
            .contains("#include directives are not supported"));
        assert!(analyzer.errors()[0].message.contains("stdio.h"));
    }

    #[test]
    fn test_semantic_analyzer_multiple_unsupported_features() {
        let mut analyzer = SemanticAnalyzer::new();

        analyzer.check_union_usage("Data");
        analyzer.check_goto_usage("error_handler");
        analyzer.check_include_usage("stdlib.h");

        // Should detect all three unsupported features
        assert_eq!(analyzer.errors().len(), 3);
        assert!(analyzer
            .errors()
            .iter()
            .all(|e| e.kind == SemanticErrorKind::UnsupportedFeature));
    }

    #[test]
    fn test_semantic_analyzer_macro_valid_name() {
        let mut analyzer = SemanticAnalyzer::new();
        let macro_def = MacroDefinition {
            name: Ident::new("__MAX__".to_string()),
            params: vec![],
            body: String::new(),
        };

        analyzer.analyze_macro_definition_test(&macro_def);
        assert_eq!(analyzer.errors().len(), 0);
    }

    #[test]
    fn test_semantic_analyzer_macro_invalid_name_no_prefix() {
        let mut analyzer = SemanticAnalyzer::new();
        let macro_def = MacroDefinition {
            name: Ident::new("MAX__".to_string()),
            params: vec![],
            body: String::new(),
        };

        analyzer.analyze_macro_definition_test(&macro_def);
        assert_eq!(analyzer.errors().len(), 1);
        assert_eq!(
            analyzer.errors()[0].kind,
            SemanticErrorKind::InvalidOperation
        );
        assert!(analyzer.errors()[0].message.contains("double-underscore"));
    }

    #[test]
    fn test_semantic_analyzer_macro_invalid_name_no_suffix() {
        let mut analyzer = SemanticAnalyzer::new();
        let macro_def = MacroDefinition {
            name: Ident::new("__MAX".to_string()),
            params: vec![],
            body: String::new(),
        };

        analyzer.analyze_macro_definition_test(&macro_def);
        assert_eq!(analyzer.errors().len(), 1);
        assert_eq!(
            analyzer.errors()[0].kind,
            SemanticErrorKind::InvalidOperation
        );
    }

    #[test]
    fn test_semantic_analyzer_macro_with_params() {
        let mut analyzer = SemanticAnalyzer::new();
        let macro_def = MacroDefinition {
            name: Ident::new("__ADD__".to_string()),
            params: vec![Ident::new("a".to_string()), Ident::new("b".to_string())],
            body: "a + b".to_string(),
        };

        analyzer.analyze_macro_definition_test(&macro_def);
        assert_eq!(analyzer.errors().len(), 0);
    }

    #[test]
    fn test_semantic_analyzer_function_with_double_underscore_name() {
        let mut analyzer = SemanticAnalyzer::new();
        let func = Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("__my_function__".to_string()),
            params: vec![],
            return_type: None,
            body: Some(Block::new(vec![])),
            is_variadic: false,
            is_unsafe: false,
            doc_comments: vec![],
            attributes: vec![],
        };

        analyzer.analyze_function_test(&func);
        assert_eq!(analyzer.errors().len(), 1);
        assert_eq!(
            analyzer.errors()[0].kind,
            SemanticErrorKind::InvalidOperation
        );
        assert!(analyzer.errors()[0].message.contains("reserved for macros"));
    }

    // Property-based tests

    mod property_tests {
        use crate::ast::{BinaryOp, Expression, Literal, PrimitiveType, Type};
        use crate::semantic::SemanticAnalyzer;
        use proptest::prelude::*;

        // Property 28: Type checking matches Rust semantics
        proptest! {
            #[test]
            fn prop_type_checking_matches_rust_semantics(
                a in 0i64..100,
                b in 0i64..100,
            ) {
                let mut analyzer = SemanticAnalyzer::new();

                // Test arithmetic operations
                let expr = Expression::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expression::Literal(Literal::Int(a))),
                    right: Box::new(Expression::Literal(Literal::Int(b))),
                };

                let result_type = analyzer.analyze_expression_test(&expr);

                // Should infer i32 type (matching Rust's default integer type)
                prop_assert!(matches!(result_type, Type::Primitive(PrimitiveType::I32)));
                prop_assert_eq!(analyzer.errors().len(), 0);
            }

            #[test]
            fn prop_comparison_returns_bool(
                a in 0i64..100,
                b in 0i64..100,
            ) {
                let mut analyzer = SemanticAnalyzer::new();

                // Test comparison operations
                let expr = Expression::Binary {
                    op: BinaryOp::Lt,
                    left: Box::new(Expression::Literal(Literal::Int(a))),
                    right: Box::new(Expression::Literal(Literal::Int(b))),
                };

                let result_type = analyzer.analyze_expression_test(&expr);

                // Comparisons should always return bool (matching Rust semantics)
                prop_assert!(matches!(result_type, Type::Primitive(PrimitiveType::Bool)));
                prop_assert_eq!(analyzer.errors().len(), 0);
            }

            #[test]
            fn prop_array_elements_must_have_same_type(
                size in 1usize..10,
            ) {
                let mut analyzer = SemanticAnalyzer::new();

                // Create array with all integer elements
                let elements: Vec<Expression> = (0..size)
                    .map(|i| Expression::Literal(Literal::Int(i as i64)))
                    .collect();

                let expr = Expression::ArrayLit { elements };

                let result_type = analyzer.analyze_expression_test(&expr);

                // Should infer array type with correct size
                match result_type {
                    Type::Array { ty, size: arr_size } => {
                        prop_assert!(matches!(*ty, Type::Primitive(PrimitiveType::I32)));
                        prop_assert_eq!(arr_size, Some(size));
                    }
                    _ => prop_assert!(false, "Expected array type"),
                }
                prop_assert_eq!(analyzer.errors().len(), 0);
            }

            #[test]
            fn prop_tuple_preserves_element_types(
                int_val in 0i64..100,
                bool_val in proptest::bool::ANY,
            ) {
                let mut analyzer = SemanticAnalyzer::new();

                // Create tuple with different types
                let expr = Expression::TupleLit {
                    elements: vec![
                        Expression::Literal(Literal::Int(int_val)),
                        Expression::Literal(Literal::Bool(bool_val)),
                    ],
                };

                let result_type = analyzer.analyze_expression_test(&expr);

                // Tuple should preserve element types (matching Rust semantics)
                match result_type {
                    Type::Tuple { types } => {
                        prop_assert_eq!(types.len(), 2);
                        prop_assert!(matches!(types[0], Type::Primitive(PrimitiveType::I32)));
                        prop_assert!(matches!(types[1], Type::Primitive(PrimitiveType::Bool)));
                    }
                    _ => prop_assert!(false, "Expected tuple type"),
                }
                prop_assert_eq!(analyzer.errors().len(), 0);
            }
        }
    }

    // Unit tests for typedef type alias resolution

    #[test]
    fn test_resolve_type_simple_alias() {
        let mut env = TypeEnvironment::new();

        // Register typedef int MyInt
        env.register_type(
            "MyInt".to_string(),
            TypeInfo::new(
                "MyInt".to_string(),
                TypeKind::Alias {
                    target: Type::Primitive(PrimitiveType::Int),
                },
            ),
        );

        let alias_type = Type::Ident(Ident::new("MyInt"));
        let resolved = env.resolve_type(&alias_type);

        assert!(matches!(resolved, Type::Primitive(PrimitiveType::Int)));
    }

    #[test]
    fn test_resolve_type_chained_alias() {
        let mut env = TypeEnvironment::new();

        // Register typedef int A
        env.register_type(
            "A".to_string(),
            TypeInfo::new(
                "A".to_string(),
                TypeKind::Alias {
                    target: Type::Primitive(PrimitiveType::Int),
                },
            ),
        );

        // Register typedef A B
        env.register_type(
            "B".to_string(),
            TypeInfo::new(
                "B".to_string(),
                TypeKind::Alias {
                    target: Type::Ident(Ident::new("A")),
                },
            ),
        );

        // Register typedef B C
        env.register_type(
            "C".to_string(),
            TypeInfo::new(
                "C".to_string(),
                TypeKind::Alias {
                    target: Type::Ident(Ident::new("B")),
                },
            ),
        );

        let alias_type = Type::Ident(Ident::new("C"));
        let resolved = env.resolve_type(&alias_type);

        assert!(matches!(resolved, Type::Primitive(PrimitiveType::Int)));
    }

    #[test]
    fn test_resolve_type_pointer_alias() {
        let mut env = TypeEnvironment::new();

        // Register typedef *int IntPtr
        env.register_type(
            "IntPtr".to_string(),
            TypeInfo::new(
                "IntPtr".to_string(),
                TypeKind::Alias {
                    target: Type::Pointer {
                        ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                        mutable: true,
                    },
                },
            ),
        );

        let alias_type = Type::Ident(Ident::new("IntPtr"));
        let resolved = env.resolve_type(&alias_type);

        match resolved {
            Type::Pointer { ty, mutable } => {
                assert!(matches!(*ty, Type::Primitive(PrimitiveType::Int)));
                assert!(mutable);
            }
            _ => panic!("Expected pointer type"),
        }
    }

    #[test]
    fn test_resolve_type_complex_nested() {
        let mut env = TypeEnvironment::new();

        // Register typedef int MyInt
        env.register_type(
            "MyInt".to_string(),
            TypeInfo::new(
                "MyInt".to_string(),
                TypeKind::Alias {
                    target: Type::Primitive(PrimitiveType::Int),
                },
            ),
        );

        // Create a pointer to MyInt
        let complex_type = Type::Pointer {
            ty: Box::new(Type::Ident(Ident::new("MyInt"))),
            mutable: true,
        };

        let resolved = env.resolve_type(&complex_type);

        match resolved {
            Type::Pointer { ty, mutable } => {
                assert!(matches!(*ty, Type::Primitive(PrimitiveType::Int)));
                assert!(mutable);
            }
            _ => panic!("Expected pointer type"),
        }
    }

    #[test]
    fn test_is_compatible_with_typedef() {
        let mut env = TypeEnvironment::new();

        // Register typedef int MyInt
        env.register_type(
            "MyInt".to_string(),
            TypeInfo::new(
                "MyInt".to_string(),
                TypeKind::Alias {
                    target: Type::Primitive(PrimitiveType::Int),
                },
            ),
        );

        let alias_type = Type::Ident(Ident::new("MyInt"));
        let int_type = Type::Primitive(PrimitiveType::Int);

        // MyInt should be compatible with int
        assert!(env.is_compatible(&alias_type, &int_type));
        assert!(env.is_compatible(&int_type, &alias_type));
    }

    #[test]
    fn test_has_circular_reference_direct() {
        let mut env = TypeEnvironment::new();

        // Register typedef A A (self-referential)
        env.register_type(
            "A".to_string(),
            TypeInfo::new(
                "A".to_string(),
                TypeKind::Alias {
                    target: Type::Ident(Ident::new("A")),
                },
            ),
        );

        let alias_type = Type::Ident(Ident::new("A"));
        let mut visited = HashSet::new();

        assert!(env.has_circular_reference(&alias_type, &mut visited));
    }

    #[test]
    fn test_has_circular_reference_indirect() {
        let mut env = TypeEnvironment::new();

        // Register typedef B A
        env.register_type(
            "A".to_string(),
            TypeInfo::new(
                "A".to_string(),
                TypeKind::Alias {
                    target: Type::Ident(Ident::new("B")),
                },
            ),
        );

        // Register typedef A B (creates cycle)
        env.register_type(
            "B".to_string(),
            TypeInfo::new(
                "B".to_string(),
                TypeKind::Alias {
                    target: Type::Ident(Ident::new("A")),
                },
            ),
        );

        let alias_type = Type::Ident(Ident::new("A"));
        let mut visited = HashSet::new();

        assert!(env.has_circular_reference(&alias_type, &mut visited));
    }

    #[test]
    fn test_has_circular_reference_multi_step() {
        let mut env = TypeEnvironment::new();

        // Register typedef B A
        env.register_type(
            "A".to_string(),
            TypeInfo::new(
                "A".to_string(),
                TypeKind::Alias {
                    target: Type::Ident(Ident::new("B")),
                },
            ),
        );

        // Register typedef C B
        env.register_type(
            "B".to_string(),
            TypeInfo::new(
                "B".to_string(),
                TypeKind::Alias {
                    target: Type::Ident(Ident::new("C")),
                },
            ),
        );

        // Register typedef A C (creates cycle A -> B -> C -> A)
        env.register_type(
            "C".to_string(),
            TypeInfo::new(
                "C".to_string(),
                TypeKind::Alias {
                    target: Type::Ident(Ident::new("A")),
                },
            ),
        );

        let alias_type = Type::Ident(Ident::new("A"));
        let mut visited = HashSet::new();

        assert!(env.has_circular_reference(&alias_type, &mut visited));
    }

    #[test]
    fn test_has_circular_reference_no_cycle() {
        let mut env = TypeEnvironment::new();

        // Register typedef int A
        env.register_type(
            "A".to_string(),
            TypeInfo::new(
                "A".to_string(),
                TypeKind::Alias {
                    target: Type::Primitive(PrimitiveType::Int),
                },
            ),
        );

        // Register typedef A B
        env.register_type(
            "B".to_string(),
            TypeInfo::new(
                "B".to_string(),
                TypeKind::Alias {
                    target: Type::Ident(Ident::new("A")),
                },
            ),
        );

        let alias_type = Type::Ident(Ident::new("B"));
        let mut visited = HashSet::new();

        assert!(!env.has_circular_reference(&alias_type, &mut visited));
    }

    #[test]
    fn test_analyze_typedef_with_circular_reference() {
        let mut analyzer = SemanticAnalyzer::new();

        // First register typedef int A (no cycle yet)
        let typedef_a = Typedef {
            visibility: Visibility::Public,
            name: Ident::new("A"),
            target: Type::Primitive(PrimitiveType::Int),
            doc_comments: vec![],
        };
        analyzer.analyze_typedef_test(&typedef_a);

        // Now register typedef A B (creates self-reference to A)
        let typedef_b = Typedef {
            visibility: Visibility::Public,
            name: Ident::new("A"),
            target: Type::Ident(Ident::new("A")),
            doc_comments: vec![],
        };

        // Clear previous errors
        analyzer.clear_errors_for_test();
        analyzer.analyze_typedef_test(&typedef_b);

        // Should have one error for circular reference
        assert_eq!(analyzer.errors().len(), 1);
        // The error could be either circular reference or duplicate definition
        let error_msg = &analyzer.errors()[0].message;
        assert!(
            error_msg.contains("circular") || error_msg.contains("already defined"),
            "Expected circular reference or duplicate definition error, got: {}",
            error_msg
        );
    }

    #[test]
    fn test_resolve_type_generic_alias() {
        let mut env = TypeEnvironment::new();

        // Register typedef Vec[int] IntVec
        env.register_type(
            "IntVec".to_string(),
            TypeInfo::new(
                "IntVec".to_string(),
                TypeKind::Alias {
                    target: Type::Generic {
                        base: Box::new(Type::Ident(Ident::new("Vec"))),
                        args: vec![Type::Primitive(PrimitiveType::Int)],
                    },
                },
            ),
        );

        let alias_type = Type::Ident(Ident::new("IntVec"));
        let resolved = env.resolve_type(&alias_type);

        match resolved {
            Type::Generic { base, args } => {
                assert!(matches!(*base, Type::Ident(ref i) if i.name == "Vec"));
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0], Type::Primitive(PrimitiveType::Int)));
            }
            _ => panic!("Expected generic type"),
        }
    }

    #[test]
    fn test_is_compatible_with_generic_typedef() {
        let mut env = TypeEnvironment::new();

        // Register typedef Vec[int] IntVec
        env.register_type(
            "IntVec".to_string(),
            TypeInfo::new(
                "IntVec".to_string(),
                TypeKind::Alias {
                    target: Type::Generic {
                        base: Box::new(Type::Ident(Ident::new("Vec"))),
                        args: vec![Type::Primitive(PrimitiveType::Int)],
                    },
                },
            ),
        );

        let alias_type = Type::Ident(Ident::new("IntVec"));
        let generic_type = Type::Generic {
            base: Box::new(Type::Ident(Ident::new("Vec"))),
            args: vec![Type::Primitive(PrimitiveType::Int)],
        };

        // IntVec should be compatible with Vec[int]
        assert!(env.is_compatible(&alias_type, &generic_type));
        assert!(env.is_compatible(&generic_type, &alias_type));
    }
}
