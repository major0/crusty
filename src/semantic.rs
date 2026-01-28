// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Semantic analysis module for type checking and validation.

use crate::ast::Type;

#[cfg(test)]
use crate::ast::Ident;
use std::collections::HashMap;

/// Symbol kind classification
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Variable,
    Function,
    Type,
    Const,
}

/// Symbol information stored in the symbol table
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub ty: Type,
    pub kind: SymbolKind,
    pub mutable: bool,
}

impl Symbol {
    pub fn new(name: String, ty: Type, kind: SymbolKind, mutable: bool) -> Self {
        Self {
            name,
            ty,
            kind,
            mutable,
        }
    }
}

/// A single scope containing symbols
#[derive(Debug, Clone)]
pub struct Scope {
    symbols: HashMap<String, Symbol>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, symbol: Symbol) -> Result<(), String> {
        if self.symbols.contains_key(&name) {
            return Err(format!("Symbol '{}' already defined in this scope", name));
        }
        self.symbols.insert(name, symbol);
        Ok(())
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }
}

/// Symbol table with scope management
#[derive(Debug, Clone)]
pub struct SymbolTable {
    scopes: Vec<Scope>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
        }
    }

    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    /// Exit the current scope
    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Insert a symbol into the current scope
    pub fn insert(&mut self, name: String, symbol: Symbol) -> Result<(), String> {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(name, symbol)
        } else {
            Err("No active scope".to_string())
        }
    }

    /// Lookup a symbol in all scopes (from innermost to outermost)
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.lookup(name) {
                return Some(symbol);
            }
        }
        None
    }

    /// Lookup a symbol only in the current scope
    pub fn lookup_in_current_scope(&self, name: &str) -> Option<&Symbol> {
        self.scopes.last().and_then(|scope| scope.lookup(name))
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Type information kind
#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    Primitive,
    Struct { fields: Vec<(String, Type)> },
    Enum { variants: Vec<String> },
    Alias { target: Type },
}

/// Type information stored in the type environment
#[derive(Debug, Clone, PartialEq)]
pub struct TypeInfo {
    pub name: String,
    pub kind: TypeKind,
}

impl TypeInfo {
    pub fn new(name: String, kind: TypeKind) -> Self {
        Self { name, kind }
    }
}

/// Type environment for type checking
#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    types: HashMap<String, TypeInfo>,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        let mut env = Self {
            types: HashMap::new(),
        };
        
        // Register primitive types
        env.register_type(
            "int".to_string(),
            TypeInfo::new("int".to_string(), TypeKind::Primitive),
        );
        env.register_type(
            "i32".to_string(),
            TypeInfo::new("i32".to_string(), TypeKind::Primitive),
        );
        env.register_type(
            "i64".to_string(),
            TypeInfo::new("i64".to_string(), TypeKind::Primitive),
        );
        env.register_type(
            "u32".to_string(),
            TypeInfo::new("u32".to_string(), TypeKind::Primitive),
        );
        env.register_type(
            "u64".to_string(),
            TypeInfo::new("u64".to_string(), TypeKind::Primitive),
        );
        env.register_type(
            "float".to_string(),
            TypeInfo::new("float".to_string(), TypeKind::Primitive),
        );
        env.register_type(
            "f32".to_string(),
            TypeInfo::new("f32".to_string(), TypeKind::Primitive),
        );
        env.register_type(
            "f64".to_string(),
            TypeInfo::new("f64".to_string(), TypeKind::Primitive),
        );
        env.register_type(
            "bool".to_string(),
            TypeInfo::new("bool".to_string(), TypeKind::Primitive),
        );
        env.register_type(
            "char".to_string(),
            TypeInfo::new("char".to_string(), TypeKind::Primitive),
        );
        env.register_type(
            "void".to_string(),
            TypeInfo::new("void".to_string(), TypeKind::Primitive),
        );
        
        env
    }

    /// Register a new type in the environment
    pub fn register_type(&mut self, name: String, info: TypeInfo) {
        self.types.insert(name, info);
    }

    /// Get type information by name
    pub fn get_type(&self, name: &str) -> Option<&TypeInfo> {
        self.types.get(name)
    }

    /// Check if two types are compatible
    pub fn is_compatible(&self, t1: &Type, t2: &Type) -> bool {
        use crate::ast::PrimitiveType;
        
        match (t1, t2) {
            // Auto type is compatible with anything
            (Type::Auto, _) | (_, Type::Auto) => true,
            
            // Numeric type compatibility (int can be used as i32, etc.)
            (Type::Primitive(PrimitiveType::Int), Type::Primitive(PrimitiveType::I32)) => true,
            (Type::Primitive(PrimitiveType::I32), Type::Primitive(PrimitiveType::Int)) => true,
            (Type::Primitive(PrimitiveType::Float), Type::Primitive(PrimitiveType::F64)) => true,
            (Type::Primitive(PrimitiveType::F64), Type::Primitive(PrimitiveType::Float)) => true,
            
            // Exact match for primitives
            (Type::Primitive(p1), Type::Primitive(p2)) => p1 == p2,
            (Type::Ident(i1), Type::Ident(i2)) => i1.name == i2.name,
            
            // Pointer compatibility
            (Type::Pointer { ty: ty1, mutable: m1 }, Type::Pointer { ty: ty2, mutable: m2 }) => {
                m1 == m2 && self.is_compatible(ty1, ty2)
            }
            
            // Reference compatibility
            (Type::Reference { ty: ty1, mutable: m1 }, Type::Reference { ty: ty2, mutable: m2 }) => {
                // Immutable reference can be created from mutable, but not vice versa
                (*m1 || !*m2) && self.is_compatible(ty1, ty2)
            }
            
            // Array compatibility
            (Type::Array { ty: ty1, size: s1 }, Type::Array { ty: ty2, size: s2 }) => {
                s1 == s2 && self.is_compatible(ty1, ty2)
            }
            
            // Slice compatibility
            (Type::Slice { ty: ty1 }, Type::Slice { ty: ty2 }) => {
                self.is_compatible(ty1, ty2)
            }
            
            // Tuple compatibility
            (Type::Tuple { types: types1 }, Type::Tuple { types: types2 }) => {
                types1.len() == types2.len()
                    && types1.iter().zip(types2.iter()).all(|(t1, t2)| self.is_compatible(t1, t2))
            }
            
            // Generic compatibility
            (Type::Generic { base: b1, args: a1 }, Type::Generic { base: b2, args: a2 }) => {
                self.is_compatible(b1, b2)
                    && a1.len() == a2.len()
                    && a1.iter().zip(a2.iter()).all(|(t1, t2)| self.is_compatible(t1, t2))
            }
            
            // Function compatibility
            (Type::Function { params: p1, return_type: r1 }, Type::Function { params: p2, return_type: r2 }) => {
                p1.len() == p2.len()
                    && p1.iter().zip(p2.iter()).all(|(t1, t2)| self.is_compatible(t1, t2))
                    && self.is_compatible(r1, r2)
            }
            
            // Fallible compatibility
            (Type::Fallible { ty: ty1 }, Type::Fallible { ty: ty2 }) => {
                self.is_compatible(ty1, ty2)
            }
            
            _ => false,
        }
    }
}

impl Default for TypeEnvironment {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::PrimitiveType;

    // Symbol Table Tests

    #[test]
    fn test_symbol_table_creation() {
        let table = SymbolTable::new();
        assert_eq!(table.scopes.len(), 1);
    }

    #[test]
    fn test_symbol_table_enter_exit_scope() {
        let mut table = SymbolTable::new();
        assert_eq!(table.scopes.len(), 1);
        
        table.enter_scope();
        assert_eq!(table.scopes.len(), 2);
        
        table.enter_scope();
        assert_eq!(table.scopes.len(), 3);
        
        table.exit_scope();
        assert_eq!(table.scopes.len(), 2);
        
        table.exit_scope();
        assert_eq!(table.scopes.len(), 1);
        
        // Should not go below 1 scope
        table.exit_scope();
        assert_eq!(table.scopes.len(), 1);
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
        assert!(matches!(found.unwrap().ty, Type::Primitive(PrimitiveType::I64)));
        assert!(found.unwrap().mutable);
        
        // Exit inner scope
        table.exit_scope();
        
        // Lookup should now find outer scope's 'x'
        let found = table.lookup("x");
        assert!(found.is_some());
        assert!(matches!(found.unwrap().ty, Type::Primitive(PrimitiveType::I32)));
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
            ("x", Type::Primitive(PrimitiveType::I32), SymbolKind::Variable),
            ("y", Type::Primitive(PrimitiveType::Bool), SymbolKind::Variable),
            ("foo", Type::Primitive(PrimitiveType::Void), SymbolKind::Function),
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
        let kinds = vec![
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
            },
        );
        
        env.register_type("Point".to_string(), type_info);
        
        let found = env.get_type("Point");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Point");
        
        match &found.unwrap().kind {
            TypeKind::Struct { fields } => {
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
            TypeKind::Struct { fields: vec![] },
        );
        assert!(matches!(struct_type.kind, TypeKind::Struct { .. }));
        
        let enum_type = TypeInfo::new(
            "Color".to_string(),
            TypeKind::Enum { variants: vec![] },
        );
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
}
