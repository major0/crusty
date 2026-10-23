// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Semantic analysis module for type checking and validation.

use crate::ast::{Ident, Type};
use crate::error::{SemanticError, SemanticErrorKind, Span};

use std::collections::{HashMap, HashSet};

/// Symbol kind classification
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Variable,
    Function,
    Type,
    Const,
}

/// Capture kind for nested functions
#[derive(Debug, Clone, PartialEq)]
pub enum CaptureKind {
    Immutable, // Variable is only read
    Mutable,   // Variable is modified
}

/// Information about a captured variable in a nested function
#[derive(Debug, Clone, PartialEq)]
pub struct Capture {
    pub name: String,
    pub ty: Type,
    pub kind: CaptureKind,
}

impl Capture {
    pub fn new(name: String, ty: Type, kind: CaptureKind) -> Self {
        Self { name, ty, kind }
    }
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

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
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
    #[allow(dead_code)]
    pub fn lookup_in_current_scope(&self, name: &str) -> Option<&Symbol> {
        self.scopes.last().and_then(|scope| scope.lookup(name))
    }

    /// Number of active scopes (for testing)
    #[cfg(test)]
    #[allow(dead_code)] // used by the lib-target `semantic_unit_tests` module
    pub fn scopes_len(&self) -> usize {
        self.scopes.len()
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

    /// Resolve a type by following type aliases
    /// Returns the resolved type, or the original type if it's not an alias
    pub fn resolve_type(&self, ty: &Type) -> Type {
        match ty {
            Type::Ident(ident) => {
                // Look up the type in the type environment
                if let Some(type_info) = self.types.get(&ident.name) {
                    match &type_info.kind {
                        TypeKind::Alias { target } => {
                            // Recursively resolve the target type
                            // This handles chains like: typedef A B; typedef B C;
                            self.resolve_type(target)
                        }
                        _ => ty.clone(),
                    }
                } else if let Some(prim) = Self::c_alias_to_primitive(&ident.name) {
                    // A built-in C/POSIX alias (ADR-0003), only when the program
                    // did not define a typedef of the same name above.
                    Type::Primitive(prim)
                } else {
                    ty.clone()
                }
            }
            // For complex types, recursively resolve inner types
            Type::Pointer { ty: inner, mutable } => Type::Pointer {
                ty: Box::new(self.resolve_type(inner)),
                mutable: *mutable,
            },
            Type::Reference { ty: inner, mutable } => Type::Reference {
                ty: Box::new(self.resolve_type(inner)),
                mutable: *mutable,
            },
            Type::Array { ty: inner, size } => Type::Array {
                ty: Box::new(self.resolve_type(inner)),
                size: *size,
            },
            Type::Slice { ty: inner } => Type::Slice {
                ty: Box::new(self.resolve_type(inner)),
            },
            Type::Generic { base, args } => Type::Generic {
                base: Box::new(self.resolve_type(base)),
                args: args.iter().map(|t| self.resolve_type(t)).collect(),
            },
            Type::Tuple { types } => Type::Tuple {
                types: types.iter().map(|t| self.resolve_type(t)).collect(),
            },
            Type::Function {
                params,
                return_type,
            } => Type::Function {
                params: params.iter().map(|t| self.resolve_type(t)).collect(),
                return_type: Box::new(self.resolve_type(return_type)),
            },
            Type::Fallible { ty: inner } => Type::Fallible {
                ty: Box::new(self.resolve_type(inner)),
            },
            // Primitives and Auto don't need resolution
            _ => ty.clone(),
        }
    }

    /// Check if a type alias chain contains a circular reference
    pub fn has_circular_reference(&self, ty: &Type, visited: &mut HashSet<String>) -> bool {
        match ty {
            Type::Ident(ident) => {
                if visited.contains(&ident.name) {
                    return true; // Circular reference detected
                }

                if let Some(type_info) = self.types.get(&ident.name) {
                    if let TypeKind::Alias { target } = &type_info.kind {
                        visited.insert(ident.name.clone());
                        let result = self.has_circular_reference(target, visited);
                        visited.remove(&ident.name);
                        return result;
                    }
                }
                false
            }
            // Check inner types for complex types
            Type::Pointer { ty, .. }
            | Type::Reference { ty, .. }
            | Type::Array { ty, .. }
            | Type::Slice { ty }
            | Type::Fallible { ty } => self.has_circular_reference(ty, visited),

            Type::Generic { base, args } => {
                self.has_circular_reference(base, visited)
                    || args.iter().any(|t| self.has_circular_reference(t, visited))
            }

            Type::Tuple { types } => types
                .iter()
                .any(|t| self.has_circular_reference(t, visited)),

            Type::Function {
                params,
                return_type,
            } => {
                params
                    .iter()
                    .any(|t| self.has_circular_reference(t, visited))
                    || self.has_circular_reference(return_type, visited)
            }

            _ => false,
        }
    }

    /// Map a built-in C/POSIX type alias (ADR-0003) to its primitive type, or
    /// `None` if the name is not an alias. Used by `resolve_type` so aliases
    /// type-check as their underlying integer; a user typedef of the same name
    /// is consulted first and wins.
    fn c_alias_to_primitive(name: &str) -> Option<crate::ast::PrimitiveType> {
        use crate::ast::PrimitiveType::*;
        Some(match name {
            "int8_t" => I8,
            "int16_t" => I16,
            "int32_t" => I32,
            "int64_t" => I64,
            "uint8_t" | "byte" => U8,
            "uint16_t" => U16,
            "uint32_t" => U32,
            "uint64_t" => U64,
            "size_t" | "uintptr_t" => Usize,
            "ssize_t" | "ptrdiff_t" | "intptr_t" => Isize,
            _ => return None,
        })
    }

    /// Whether a primitive is one of the integer types (the C-style `int` plus
    /// every fixed-width signed/unsigned integer). Used for lenient integer
    /// width compatibility.
    fn is_integer_primitive(p: &crate::ast::PrimitiveType) -> bool {
        use crate::ast::PrimitiveType::*;
        matches!(
            p,
            Int | I8 | I16 | I32 | I64 | I128 | U8 | U16 | U32 | U64 | U128 | Usize | Isize
        )
    }

    /// Check if two types are compatible
    pub fn is_compatible(&self, t1: &Type, t2: &Type) -> bool {
        use crate::ast::PrimitiveType;

        // Resolve type aliases first
        let resolved_t1 = self.resolve_type(t1);
        let resolved_t2 = self.resolve_type(t2);

        match (&resolved_t1, &resolved_t2) {
            // Auto type is compatible with anything
            (Type::Auto, _) | (_, Type::Auto) => true,

            // Numeric type compatibility. Crusty is lenient about integer
            // width here — an integer literal is typed I32 but legitimately
            // fits any integer-typed binding (rustc performs the real
            // width/overflow checking). So any integer primitive unifies with
            // any other, and likewise the two float types.
            (Type::Primitive(p1), Type::Primitive(p2))
                if Self::is_integer_primitive(p1) && Self::is_integer_primitive(p2) =>
            {
                true
            }
            (Type::Primitive(PrimitiveType::Float), Type::Primitive(PrimitiveType::F64)) => true,
            (Type::Primitive(PrimitiveType::F64), Type::Primitive(PrimitiveType::Float)) => true,
            (Type::Primitive(PrimitiveType::Float), Type::Primitive(PrimitiveType::F32)) => true,
            (Type::Primitive(PrimitiveType::F32), Type::Primitive(PrimitiveType::Float)) => true,

            // Exact match for primitives
            (Type::Primitive(p1), Type::Primitive(p2)) => p1 == p2,
            (Type::Ident(i1), Type::Ident(i2)) => i1.name == i2.name,

            // Pointer compatibility
            (
                Type::Pointer {
                    ty: ty1,
                    mutable: m1,
                },
                Type::Pointer {
                    ty: ty2,
                    mutable: m2,
                },
            ) => m1 == m2 && self.is_compatible(ty1, ty2),

            // Reference compatibility
            (
                Type::Reference {
                    ty: ty1,
                    mutable: m1,
                },
                Type::Reference {
                    ty: ty2,
                    mutable: m2,
                },
            ) => {
                // Immutable reference can be created from mutable, but not vice versa
                (*m1 || !*m2) && self.is_compatible(ty1, ty2)
            }

            // Array compatibility
            (Type::Array { ty: ty1, size: s1 }, Type::Array { ty: ty2, size: s2 }) => {
                s1 == s2 && self.is_compatible(ty1, ty2)
            }

            // Slice compatibility
            (Type::Slice { ty: ty1 }, Type::Slice { ty: ty2 }) => self.is_compatible(ty1, ty2),

            // Tuple compatibility
            (Type::Tuple { types: types1 }, Type::Tuple { types: types2 }) => {
                types1.len() == types2.len()
                    && types1
                        .iter()
                        .zip(types2.iter())
                        .all(|(t1, t2)| self.is_compatible(t1, t2))
            }

            // Generic compatibility
            (Type::Generic { base: b1, args: a1 }, Type::Generic { base: b2, args: a2 }) => {
                self.is_compatible(b1, b2)
                    && a1.len() == a2.len()
                    && a1
                        .iter()
                        .zip(a2.iter())
                        .all(|(t1, t2)| self.is_compatible(t1, t2))
            }

            // Function compatibility
            (
                Type::Function {
                    params: p1,
                    return_type: r1,
                },
                Type::Function {
                    params: p2,
                    return_type: r2,
                },
            ) => {
                p1.len() == p2.len()
                    && p1
                        .iter()
                        .zip(p2.iter())
                        .all(|(t1, t2)| self.is_compatible(t1, t2))
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

/// Semantic analyzer for type checking and validation
#[derive(Debug, Clone)]
pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
    type_env: TypeEnvironment,
    errors: Vec<SemanticError>,
    /// Track captures for nested functions: function_name -> list of captures
    nested_function_captures: HashMap<String, Vec<Capture>>,
    /// Track variables that are modified (for determining mutable captures)
    #[allow(dead_code)]
    modified_variables: std::collections::HashSet<String>,
    /// Track the expected return type for the current function being analyzed
    expected_return_type: Option<Type>,
    /// Track if we're currently inside a nested function (for multi-level nesting detection)
    /// This flag is set to true when analyzing a nested function body and is used to reject
    /// nested functions within nested functions (Requirement 59.19: no multi-level nesting).
    inside_nested_function: bool,
}

impl SemanticAnalyzer {
    /// Create a new semantic analyzer
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            type_env: TypeEnvironment::new(),
            errors: Vec::new(),
            nested_function_captures: HashMap::new(),
            modified_variables: std::collections::HashSet::new(),
            expected_return_type: None,
            inside_nested_function: false,
        }
    }

    /// Analyze a complete file AST
    pub fn analyze(&mut self, file: &crate::ast::File) -> Result<(), Vec<SemanticError>> {
        // Clear previous errors
        self.errors.clear();

        // Analyze all items in the file
        for item in &file.items {
            self.analyze_item(item);
        }

        // Return errors if any were found
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    /// Analyze a single item
    fn analyze_item(&mut self, item: &crate::ast::Item) {
        use crate::ast::Item;

        match item {
            Item::Function(func) => self.analyze_function(func),
            Item::Struct(struct_def) => self.analyze_struct(struct_def),
            Item::Enum(enum_def) => self.analyze_enum(enum_def),
            Item::Typedef(typedef) => self.analyze_typedef(typedef),
            Item::Const(const_def) => self.analyze_const(const_def),
            Item::Static(static_def) => self.analyze_static(static_def),
            Item::MacroDefinition(macro_def) => self.analyze_macro_definition(macro_def),
            Item::ConditionalCompilation(cc) => {
                // Analyze the conditionally-compiled items; the cfg predicate
                // itself is passed through to rustc (ADR-0001).
                for inner in &cc.items {
                    self.analyze_item(inner);
                }
            }
            Item::Namespace(_) | Item::Import(_) | Item::Export(_) | Item::Extern(_) => {
                // These items don't require semantic analysis in this phase
            }
            Item::RawRust { .. } => {
                // Raw Rust passthrough (`__rust__{ … }`) is opaque to Crusty's
                // analyzer — rustc validates it.
            }
        }
    }

    /// Analyze a function declaration
    fn analyze_function(&mut self, func: &crate::ast::Function) {
        // Validate that function names don't use double-underscore pattern (reserved for macros)
        if func.name.name.starts_with("__") && func.name.name.ends_with("__") {
            self.errors.push(SemanticError::new(
                Span::new(
                    crate::error::Position::new(0, 0),
                    crate::error::Position::new(0, 0),
                ),
                SemanticErrorKind::InvalidOperation,
                format!(
                    "Function names cannot use double-underscore pattern (reserved for macros): '{}'",
                    func.name.name
                ),
            ));
            return;
        }

        // Register function in symbol table
        let func_type = if let Some(ref return_type) = func.return_type {
            Type::Function {
                params: func.params.iter().map(|p| p.ty.clone()).collect(),
                return_type: Box::new(return_type.clone()),
            }
        } else {
            Type::Function {
                params: func.params.iter().map(|p| p.ty.clone()).collect(),
                return_type: Box::new(Type::Primitive(crate::ast::PrimitiveType::Void)),
            }
        };

        let symbol = Symbol::new(
            func.name.name.clone(),
            func_type,
            SymbolKind::Function,
            false,
        );

        if let Err(msg) = self.symbol_table.insert(func.name.name.clone(), symbol) {
            self.errors.push(SemanticError::new(
                Span::new(
                    crate::error::Position::new(0, 0),
                    crate::error::Position::new(0, 0),
                ),
                SemanticErrorKind::DuplicateDefinition,
                msg,
            ));
            return;
        }

        // Set expected return type for this function
        let old_return_type = self.expected_return_type.clone();
        self.expected_return_type = if let Some(ref return_type) = func.return_type {
            Some(return_type.clone())
        } else {
            Some(Type::Primitive(crate::ast::PrimitiveType::Void))
        };

        // Enter function scope
        self.symbol_table.enter_scope();

        // Register parameters in function scope
        for param in &func.params {
            let param_symbol = Symbol::new(
                param.name.name.clone(),
                param.ty.clone(),
                SymbolKind::Variable,
                false,
            );

            if let Err(msg) = self
                .symbol_table
                .insert(param.name.name.clone(), param_symbol)
            {
                self.errors.push(SemanticError::new(
                    Span::new(
                        crate::error::Position::new(0, 0),
                        crate::error::Position::new(0, 0),
                    ),
                    SemanticErrorKind::DuplicateDefinition,
                    msg,
                ));
            }
        }

        // Analyze function body (definitions only; a body-less extern
        // declaration has nothing to check).
        if let Some(ref body) = func.body {
            self.analyze_block(body);
        }

        // Exit function scope
        self.symbol_table.exit_scope();

        // Restore previous return type
        self.expected_return_type = old_return_type;
    }

    /// Analyze a struct definition
    fn analyze_struct(&mut self, struct_def: &crate::ast::Struct) {
        // Register struct type in type environment
        let fields: Vec<(String, Type)> = struct_def
            .fields
            .iter()
            .map(|f| (f.name.name.clone(), f.ty.clone()))
            .collect();

        let type_info = TypeInfo::new(struct_def.name.name.clone(), TypeKind::Struct { fields });

        self.type_env
            .register_type(struct_def.name.name.clone(), type_info);

        // Register struct as a type symbol
        let symbol = Symbol::new(
            struct_def.name.name.clone(),
            Type::Ident(struct_def.name.clone()),
            SymbolKind::Type,
            false,
        );

        if let Err(msg) = self
            .symbol_table
            .insert(struct_def.name.name.clone(), symbol)
        {
            self.errors.push(SemanticError::new(
                Span::new(
                    crate::error::Position::new(0, 0),
                    crate::error::Position::new(0, 0),
                ),
                SemanticErrorKind::DuplicateDefinition,
                msg,
            ));
        }

        // Analyze struct methods
        for method in &struct_def.methods {
            self.analyze_function(method);
        }
    }

    /// Analyze an enum definition
    fn analyze_enum(&mut self, enum_def: &crate::ast::Enum) {
        // Register enum type in type environment
        let variants: Vec<String> = enum_def
            .variants
            .iter()
            .map(|v| v.name.name.clone())
            .collect();

        let type_info = TypeInfo::new(enum_def.name.name.clone(), TypeKind::Enum { variants });

        self.type_env
            .register_type(enum_def.name.name.clone(), type_info);

        // Register enum as a type symbol
        let symbol = Symbol::new(
            enum_def.name.name.clone(),
            Type::Ident(enum_def.name.clone()),
            SymbolKind::Type,
            false,
        );

        if let Err(msg) = self.symbol_table.insert(enum_def.name.name.clone(), symbol) {
            self.errors.push(SemanticError::new(
                Span::new(
                    crate::error::Position::new(0, 0),
                    crate::error::Position::new(0, 0),
                ),
                SemanticErrorKind::DuplicateDefinition,
                msg,
            ));
        }
    }

    /// Analyze a typedef
    fn analyze_typedef(&mut self, typedef: &crate::ast::Typedef) {
        // Check for circular references
        let mut visited = HashSet::new();
        if self
            .type_env
            .has_circular_reference(&typedef.target, &mut visited)
        {
            self.errors.push(SemanticError::new(
                Span::new(
                    crate::error::Position::new(0, 0),
                    crate::error::Position::new(0, 0),
                ),
                SemanticErrorKind::TypeMismatch,
                format!("circular type alias definition for '{}'", typedef.name.name),
            ));
            return;
        }

        // Register type alias in type environment
        let type_info = TypeInfo::new(
            typedef.name.name.clone(),
            TypeKind::Alias {
                target: typedef.target.clone(),
            },
        );

        self.type_env
            .register_type(typedef.name.name.clone(), type_info);

        // Register typedef as a type symbol
        let symbol = Symbol::new(
            typedef.name.name.clone(),
            typedef.target.clone(),
            SymbolKind::Type,
            false,
        );

        if let Err(msg) = self.symbol_table.insert(typedef.name.name.clone(), symbol) {
            self.errors.push(SemanticError::new(
                Span::new(
                    crate::error::Position::new(0, 0),
                    crate::error::Position::new(0, 0),
                ),
                SemanticErrorKind::DuplicateDefinition,
                msg,
            ));
        }
    }

    /// Analyze a const declaration
    fn analyze_const(&mut self, const_def: &crate::ast::Const) {
        // Analyze the constant value expression
        let value_type = self.analyze_expression(&const_def.value);

        // Check type compatibility
        if !self.type_env.is_compatible(&const_def.ty, &value_type) {
            self.errors.push(SemanticError::new(
                Span::new(
                    crate::error::Position::new(0, 0),
                    crate::error::Position::new(0, 0),
                ),
                SemanticErrorKind::TypeMismatch,
                format!(
                    "const '{}' type mismatch: expected {:?}, found {:?}",
                    const_def.name.name, const_def.ty, value_type
                ),
            ));
        }

        // Register const in symbol table
        let symbol = Symbol::new(
            const_def.name.name.clone(),
            const_def.ty.clone(),
            SymbolKind::Const,
            false,
        );

        if let Err(msg) = self
            .symbol_table
            .insert(const_def.name.name.clone(), symbol)
        {
            self.errors.push(SemanticError::new(
                Span::new(
                    crate::error::Position::new(0, 0),
                    crate::error::Position::new(0, 0),
                ),
                SemanticErrorKind::DuplicateDefinition,
                msg,
            ));
        }
    }

    /// Analyze a static declaration
    fn analyze_static(&mut self, static_def: &crate::ast::Static) {
        // Analyze the static value expression
        let value_type = self.analyze_expression(&static_def.value);

        // Check type compatibility
        if !self.type_env.is_compatible(&static_def.ty, &value_type) {
            self.errors.push(SemanticError::new(
                Span::new(
                    crate::error::Position::new(0, 0),
                    crate::error::Position::new(0, 0),
                ),
                SemanticErrorKind::TypeMismatch,
                format!(
                    "static '{}' type mismatch: expected {:?}, found {:?}",
                    static_def.name.name, static_def.ty, value_type
                ),
            ));
        }

        // Register static in symbol table
        let symbol = Symbol::new(
            static_def.name.name.clone(),
            static_def.ty.clone(),
            SymbolKind::Variable,
            static_def.mutable,
        );

        if let Err(msg) = self
            .symbol_table
            .insert(static_def.name.name.clone(), symbol)
        {
            self.errors.push(SemanticError::new(
                Span::new(
                    crate::error::Position::new(0, 0),
                    crate::error::Position::new(0, 0),
                ),
                SemanticErrorKind::DuplicateDefinition,
                msg,
            ));
        }
    }

    /// Analyze a macro definition
    fn analyze_macro_definition(&mut self, macro_def: &crate::ast::MacroDefinition) {
        // Validate macro name has double-underscore prefix and suffix
        if !macro_def.name.name.starts_with("__") || !macro_def.name.name.ends_with("__") {
            self.errors.push(SemanticError::new(
                Span::new(
                    crate::error::Position::new(0, 0),
                    crate::error::Position::new(0, 0),
                ),
                SemanticErrorKind::InvalidOperation,
                format!(
                    "macro name '{}' must have double-underscore prefix and suffix",
                    macro_def.name.name
                ),
            ));
        }

        // Macro bodies are opaque raw source spans (ADR-0001): Crusty does not
        // tokenise or inspect them. Any error inside the body surfaces from
        // rustc on the generated `macro_rules!`. Only the macro NAME is
        // validated here.
    }

    /// The element type yielded by iterating a value of type `iter_type` in a
    /// for-in loop. A slice or array of `T` yields `T`; a reference to either
    /// (`&[T]`, `&[T; N]`) yields `T`, since iterating a borrowed collection is
    /// common. A range already carries its element type (see the `Range` arm of
    /// `analyze_expression`), so it passes through.
    ///
    /// Generic collections are modelled by their base name: a sequence or set
    /// of one element type (`Vec<T>`, `VecDeque<T>`, `HashSet<T>`,
    /// `BTreeSet<T>`, `Option<T>`, …) yields `T`; a map of two type arguments
    /// (`HashMap<K, V>`, `BTreeMap<K, V>`, `IndexMap<K, V>`, …) yields the
    /// `(K, V)` pair, matching Rust's `for (k, v) in map`. An unrecognised
    /// generic, or anything else, falls back to `Auto`, which keeps the loop
    /// variable lenient rather than binding it to the wrong (collection) type.
    fn iterable_element_type(iter_type: &Type) -> Type {
        match iter_type {
            Type::Slice { ty } | Type::Array { ty, .. } => (**ty).clone(),
            Type::Reference { ty, .. } => Self::iterable_element_type(ty),
            Type::Generic { base, args } => Self::generic_element_type(base, args),
            // A range is already typed as its element type upstream.
            Type::Primitive(_) => iter_type.clone(),
            _ => Type::Auto,
        }
    }

    /// Element type yielded by iterating a generic collection `Base<args...>`.
    /// Sequence/set-like bases with one argument yield that argument; map-like
    /// bases with two arguments yield the `(K, V)` pair. Anything else — an
    /// unknown base, or an unexpected argument count — is `Auto`.
    fn generic_element_type(base: &Type, args: &[Type]) -> Type {
        let Type::Ident(name) = base else {
            return Type::Auto;
        };
        match (name.name.as_str(), args) {
            // Map-like: iterating yields a (key, value) pair.
            ("HashMap" | "BTreeMap" | "IndexMap", [k, v]) => Type::Tuple {
                types: vec![k.clone(), v.clone()],
            },
            // Sequence/set/option-like: iterating yields the single element.
            (
                "Vec" | "VecDeque" | "HashSet" | "BTreeSet" | "IndexSet" | "BinaryHeap"
                | "LinkedList" | "Option",
                [elem],
            ) => elem.clone(),
            _ => Type::Auto,
        }
    }

    /// Analyze a block of statements
    fn analyze_block(&mut self, block: &crate::ast::Block) {
        for statement in &block.statements {
            self.analyze_statement(statement);
        }
    }

    /// Analyze a statement (placeholder for sub-task 8.3)
    fn analyze_statement(&mut self, statement: &crate::ast::Statement) {
        use crate::ast::Statement;

        match statement {
            Statement::Let {
                name,
                ty,
                init,
                mutable,
            } => {
                // Analyze initialization expression if present
                let init_type = if let Some(ref init_expr) = init {
                    self.analyze_expression(init_expr)
                } else {
                    Type::Auto
                };

                // Determine the variable type
                let var_type = if let Some(ref declared_type) = ty {
                    // Check type compatibility if both type and init are present
                    if init.is_some() {
                        // Special handling for function types (nested functions)
                        let compatible = match (&init_type, declared_type) {
                            (Type::Function { .. }, Type::Function { .. }) => {
                                self.check_function_type_compatibility(&init_type, declared_type)
                            }
                            _ => self.type_env.is_compatible(declared_type, &init_type),
                        };

                        if !compatible {
                            self.errors.push(SemanticError::new(
                                Span::new(
                                    crate::error::Position::new(0, 0),
                                    crate::error::Position::new(0, 0),
                                ),
                                SemanticErrorKind::TypeMismatch,
                                format!(
                                    "variable '{}' type mismatch: expected {:?}, found {:?}",
                                    name.name, declared_type, init_type
                                ),
                            ));
                        }
                    }
                    declared_type.clone()
                } else {
                    init_type
                };

                // Register variable in symbol table
                let symbol =
                    Symbol::new(name.name.clone(), var_type, SymbolKind::Variable, *mutable);

                if let Err(msg) = self.symbol_table.insert(name.name.clone(), symbol) {
                    self.errors.push(SemanticError::new(
                        Span::new(
                            crate::error::Position::new(0, 0),
                            crate::error::Position::new(0, 0),
                        ),
                        SemanticErrorKind::DuplicateDefinition,
                        msg,
                    ));
                }
            }

            Statement::Var { name, ty, init } => {
                // Analyze initialization expression if present
                let init_type = if let Some(ref init_expr) = init {
                    self.analyze_expression(init_expr)
                } else {
                    Type::Auto
                };

                // Determine the variable type
                let var_type = if let Some(ref declared_type) = ty {
                    // Check type compatibility if both type and init are present
                    if init.is_some() {
                        // Special handling for function types (nested functions)
                        let compatible = match (&init_type, declared_type) {
                            (Type::Function { .. }, Type::Function { .. }) => {
                                self.check_function_type_compatibility(&init_type, declared_type)
                            }
                            _ => self.type_env.is_compatible(declared_type, &init_type),
                        };

                        if !compatible {
                            self.errors.push(SemanticError::new(
                                Span::new(
                                    crate::error::Position::new(0, 0),
                                    crate::error::Position::new(0, 0),
                                ),
                                SemanticErrorKind::TypeMismatch,
                                format!(
                                    "variable '{}' type mismatch: expected {:?}, found {:?}",
                                    name.name, declared_type, init_type
                                ),
                            ));
                        }
                    }
                    declared_type.clone()
                } else {
                    init_type
                };

                // Register variable in symbol table (var is always mutable)
                let symbol = Symbol::new(name.name.clone(), var_type, SymbolKind::Variable, true);

                if let Err(msg) = self.symbol_table.insert(name.name.clone(), symbol) {
                    self.errors.push(SemanticError::new(
                        Span::new(
                            crate::error::Position::new(0, 0),
                            crate::error::Position::new(0, 0),
                        ),
                        SemanticErrorKind::DuplicateDefinition,
                        msg,
                    ));
                }
            }

            Statement::Const { name, ty, value } => {
                // Analyze the constant value expression
                let value_type = self.analyze_expression(value);

                // Check type compatibility
                if !self.type_env.is_compatible(ty, &value_type) {
                    self.errors.push(SemanticError::new(
                        Span::new(
                            crate::error::Position::new(0, 0),
                            crate::error::Position::new(0, 0),
                        ),
                        SemanticErrorKind::TypeMismatch,
                        format!(
                            "const '{}' type mismatch: expected {:?}, found {:?}",
                            name.name, ty, value_type
                        ),
                    ));
                }

                // Register const in symbol table
                let symbol = Symbol::new(name.name.clone(), ty.clone(), SymbolKind::Const, false);

                if let Err(msg) = self.symbol_table.insert(name.name.clone(), symbol) {
                    self.errors.push(SemanticError::new(
                        Span::new(
                            crate::error::Position::new(0, 0),
                            crate::error::Position::new(0, 0),
                        ),
                        SemanticErrorKind::DuplicateDefinition,
                        msg,
                    ));
                }
            }

            Statement::Expr(expr) => {
                // Analyze the expression
                self.analyze_expression(expr);
            }

            Statement::Return(expr) => {
                // Analyze the return expression if present and check type
                if let Some(ref return_expr) = expr {
                    let return_type = self.analyze_expression(return_expr);

                    // Check if return type matches expected return type
                    if let Some(ref expected_type) = self.expected_return_type {
                        // Special handling for function types (nested functions)
                        let compatible = match (&return_type, expected_type) {
                            (Type::Function { .. }, Type::Function { .. }) => {
                                self.check_function_type_compatibility(&return_type, expected_type)
                            }
                            _ => self.type_env.is_compatible(expected_type, &return_type),
                        };

                        if !compatible {
                            self.errors.push(SemanticError::new(
                                Span::new(
                                    crate::error::Position::new(0, 0),
                                    crate::error::Position::new(0, 0),
                                ),
                                SemanticErrorKind::TypeMismatch,
                                format!(
                                    "return type mismatch: expected {:?}, found {:?}",
                                    expected_type, return_type
                                ),
                            ));
                        }
                    }
                } else {
                    // Returning void - check if function expects void
                    if let Some(ref expected_type) = self.expected_return_type {
                        let void_type = Type::Primitive(crate::ast::PrimitiveType::Void);
                        if !self.type_env.is_compatible(expected_type, &void_type) {
                            self.errors.push(SemanticError::new(
                                Span::new(
                                    crate::error::Position::new(0, 0),
                                    crate::error::Position::new(0, 0),
                                ),
                                SemanticErrorKind::TypeMismatch,
                                format!(
                                    "return type mismatch: expected {:?}, found void",
                                    expected_type
                                ),
                            ));
                        }
                    }
                }
            }

            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                // Analyze condition
                let cond_type = self.analyze_expression(condition);

                // Condition should be boolean
                if !self.type_env.is_compatible(
                    &Type::Primitive(crate::ast::PrimitiveType::Bool),
                    &cond_type,
                ) {
                    self.errors.push(SemanticError::new(
                        Span::new(
                            crate::error::Position::new(0, 0),
                            crate::error::Position::new(0, 0),
                        ),
                        SemanticErrorKind::TypeMismatch,
                        format!("if condition must be boolean, found {:?}", cond_type),
                    ));
                }

                // Analyze then block
                self.symbol_table.enter_scope();
                self.analyze_block(then_block);
                self.symbol_table.exit_scope();

                // Analyze else block if present
                if let Some(ref else_blk) = else_block {
                    self.symbol_table.enter_scope();
                    self.analyze_block(else_blk);
                    self.symbol_table.exit_scope();
                }
            }

            Statement::While {
                label: _,
                condition,
                body,
            } => {
                // Analyze condition
                let cond_type = self.analyze_expression(condition);

                // Condition should be boolean
                if !self.type_env.is_compatible(
                    &Type::Primitive(crate::ast::PrimitiveType::Bool),
                    &cond_type,
                ) {
                    self.errors.push(SemanticError::new(
                        Span::new(
                            crate::error::Position::new(0, 0),
                            crate::error::Position::new(0, 0),
                        ),
                        SemanticErrorKind::TypeMismatch,
                        format!("while condition must be boolean, found {:?}", cond_type),
                    ));
                }

                // Analyze body
                self.symbol_table.enter_scope();
                self.analyze_block(body);
                self.symbol_table.exit_scope();
            }

            Statement::Loop { label: _, body } => {
                // An unconditional loop has no condition to type-check; just
                // analyze the body in a fresh scope.
                self.symbol_table.enter_scope();
                self.analyze_block(body);
                self.symbol_table.exit_scope();
            }

            Statement::For {
                label: _,
                init,
                condition,
                increment,
                body,
            } => {
                // Enter scope for the for loop
                self.symbol_table.enter_scope();

                // Analyze initialization
                self.analyze_statement(init);

                // Analyze condition
                let cond_type = self.analyze_expression(condition);
                if !self.type_env.is_compatible(
                    &Type::Primitive(crate::ast::PrimitiveType::Bool),
                    &cond_type,
                ) {
                    self.errors.push(SemanticError::new(
                        Span::new(
                            crate::error::Position::new(0, 0),
                            crate::error::Position::new(0, 0),
                        ),
                        SemanticErrorKind::TypeMismatch,
                        format!("for condition must be boolean, found {:?}", cond_type),
                    ));
                }

                // Analyze increment
                self.analyze_expression(increment);

                // Analyze body
                self.analyze_block(body);

                // Exit scope
                self.symbol_table.exit_scope();
            }

            Statement::ForIn {
                label: _,
                var,
                iter,
                body,
            } => {
                // Enter scope for the for-in loop
                self.symbol_table.enter_scope();

                // Analyze iterator expression
                let iter_type = self.analyze_expression(iter);

                // The loop variable binds to the ITERABLE's element type, not the
                // iterable itself: iterating a slice/array of T yields T, and a
                // range of T yields T. Anything we don't model (generic
                // collections like Vec/IndexMap) falls back to Auto, preserving
                // the lenient behaviour rather than binding a wrong type.
                let element_type = Self::iterable_element_type(&iter_type);

                // Register loop variable
                let symbol =
                    Symbol::new(var.name.clone(), element_type, SymbolKind::Variable, false);

                if let Err(msg) = self.symbol_table.insert(var.name.clone(), symbol) {
                    self.errors.push(SemanticError::new(
                        Span::new(
                            crate::error::Position::new(0, 0),
                            crate::error::Position::new(0, 0),
                        ),
                        SemanticErrorKind::DuplicateDefinition,
                        msg,
                    ));
                }

                // Analyze body
                self.analyze_block(body);

                // Exit scope
                self.symbol_table.exit_scope();
            }

            Statement::Switch {
                expr,
                cases,
                default,
            } => {
                // Analyze switch expression
                let switch_type = self.analyze_expression(expr);

                // Analyze each case
                for case in cases {
                    for value in &case.values {
                        let value_type = self.analyze_expression(value);
                        if !self.type_env.is_compatible(&switch_type, &value_type) {
                            self.errors.push(SemanticError::new(
                                Span::new(
                                    crate::error::Position::new(0, 0),
                                    crate::error::Position::new(0, 0),
                                ),
                                SemanticErrorKind::TypeMismatch,
                                format!(
                                    "switch case value type mismatch: expected {:?}, found {:?}",
                                    switch_type, value_type
                                ),
                            ));
                        }
                    }

                    self.symbol_table.enter_scope();
                    self.analyze_block(&case.body);
                    self.symbol_table.exit_scope();
                }

                // Analyze default case if present
                if let Some(ref default_block) = default {
                    self.symbol_table.enter_scope();
                    self.analyze_block(default_block);
                    self.symbol_table.exit_scope();
                }
            }

            Statement::Break { value, .. } => {
                // The label (if any) is a control-flow target, not a symbol.
                // A break value is an ordinary expression to type-check.
                if let Some(expr) = value {
                    self.analyze_expression(expr);
                }
            }
            Statement::Continue(_) => {
                // No semantic analysis needed for continue
            }

            Statement::NestedFunction {
                name,
                params,
                return_type,
                body,
            } => {
                // ============================================================================
                // NESTED FUNCTION VALIDATION (Task 17.5)
                // ============================================================================
                // This section implements three key validation rules for nested functions:
                // 1. Reject multi-level nesting (Requirement 59.19)
                // 2. Prevent static nested functions (Requirement 59.18 - enforced by AST)
                // 3. Ensure type compatibility for function pointers (Requirement 59.17)

                // Requirement 59.19: Verify nested functions cannot contain nested functions (no multi-level nesting)
                // We track whether we're currently inside a nested function using the `inside_nested_function` flag.
                // If this flag is true when we encounter another nested function, we reject it.
                if self.inside_nested_function {
                    self.errors.push(SemanticError::new(
                        Span::new(
                            crate::error::Position::new(0, 0),
                            crate::error::Position::new(0, 0),
                        ),
                        SemanticErrorKind::UnsupportedFeature,
                        format!(
                            "nested function '{}' cannot contain nested functions (multi-level nesting not supported)",
                            name.name
                        ),
                    ));
                    return;
                }

                // Verify function name doesn't use double-underscore pattern (reserved for macros)
                if name.name.starts_with("__") && name.name.ends_with("__") {
                    self.errors.push(SemanticError::new(
                        Span::new(
                            crate::error::Position::new(0, 0),
                            crate::error::Position::new(0, 0),
                        ),
                        SemanticErrorKind::UnsupportedFeature,
                        format!(
                            "function name '{}' uses double-underscore pattern reserved for macros",
                            name.name
                        ),
                    ));
                }

                // Requirement 59.17: Register the nested function with proper type for type compatibility checking
                // This allows the nested function to be assigned to variables and passed as parameters
                // with proper type checking based on its signature (parameter types and return type).
                let func_type = if let Some(ref ret_type) = return_type {
                    Type::Function {
                        params: params.iter().map(|p| p.ty.clone()).collect(),
                        return_type: Box::new(ret_type.clone()),
                    }
                } else {
                    Type::Function {
                        params: params.iter().map(|p| p.ty.clone()).collect(),
                        return_type: Box::new(Type::Primitive(crate::ast::PrimitiveType::Void)),
                    }
                };

                let func_symbol =
                    Symbol::new(name.name.clone(), func_type, SymbolKind::Function, false);

                if let Err(msg) = self.symbol_table.insert(name.name.clone(), func_symbol) {
                    self.errors.push(SemanticError::new(
                        Span::new(
                            crate::error::Position::new(0, 0),
                            crate::error::Position::new(0, 0),
                        ),
                        SemanticErrorKind::DuplicateDefinition,
                        msg,
                    ));
                }

                // Collect variables in scope before entering nested function scope
                let variables_in_scope = self.collect_variables_in_scope();

                // Collect variables used in the nested function body
                let mut used_variables = std::collections::HashSet::new();
                self.collect_used_variables_in_block(body, &mut used_variables);

                // Collect variables modified in the nested function body
                let mut modified_variables = std::collections::HashSet::new();
                self.collect_modified_variables_in_block(body, &mut modified_variables);

                // Build capture list
                let mut captures = Vec::new();
                for var_name in &used_variables {
                    // Check if this variable is from an outer scope (not a parameter)
                    let is_parameter = params.iter().any(|p| p.name.name == *var_name);

                    if !is_parameter {
                        if let Some((var_type, _)) = variables_in_scope.get(var_name) {
                            // Determine if capture is mutable or immutable
                            let capture_kind = if modified_variables.contains(var_name) {
                                CaptureKind::Mutable
                            } else {
                                CaptureKind::Immutable
                            };

                            captures.push(Capture::new(
                                var_name.clone(),
                                var_type.clone(),
                                capture_kind,
                            ));
                        }
                    }
                }

                // Store captures for this nested function
                self.nested_function_captures
                    .insert(name.name.clone(), captures);

                // Set expected return type for this nested function
                let old_return_type = self.expected_return_type.clone();
                self.expected_return_type = if let Some(ref ret_type) = return_type {
                    Some(ret_type.clone())
                } else {
                    Some(Type::Primitive(crate::ast::PrimitiveType::Void))
                };

                // Mark that we're inside a nested function (for multi-level nesting detection)
                // This flag is checked at the start of this function to reject nested functions
                // within nested functions (Requirement 59.19).
                let was_inside_nested = self.inside_nested_function;
                self.inside_nested_function = true;

                // Enter new scope for nested function
                self.symbol_table.enter_scope();

                // Register parameters in nested function scope
                for param in params {
                    let symbol = Symbol::new(
                        param.name.name.clone(),
                        param.ty.clone(),
                        SymbolKind::Variable,
                        false,
                    );
                    if let Err(msg) = self.symbol_table.insert(param.name.name.clone(), symbol) {
                        self.errors.push(SemanticError::new(
                            Span::new(
                                crate::error::Position::new(0, 0),
                                crate::error::Position::new(0, 0),
                            ),
                            SemanticErrorKind::DuplicateDefinition,
                            msg,
                        ));
                    }
                }

                // Analyze nested function body
                self.analyze_block(body);

                // Exit nested function scope
                self.symbol_table.exit_scope();

                // Restore previous nested function state
                self.inside_nested_function = was_inside_nested;

                // Restore previous return type
                self.expected_return_type = old_return_type;
            }
        }
    }

    /// Check if two function types are compatible
    /// This is used when assigning nested functions to variables or passing them as arguments
    fn check_function_type_compatibility(&self, actual: &Type, expected: &Type) -> bool {
        match (actual, expected) {
            (
                Type::Function {
                    params: actual_params,
                    return_type: actual_ret,
                },
                Type::Function {
                    params: expected_params,
                    return_type: expected_ret,
                },
            ) => {
                // Check parameter count
                if actual_params.len() != expected_params.len() {
                    return false;
                }

                // Check each parameter type
                for (actual_param, expected_param) in
                    actual_params.iter().zip(expected_params.iter())
                {
                    if !self.type_env.is_compatible(actual_param, expected_param) {
                        return false;
                    }
                }

                // Check return type
                self.type_env.is_compatible(actual_ret, expected_ret)
            }
            _ => false,
        }
    }

    /// Analyze an expression and return its type (placeholder for sub-task 8.4)
    fn analyze_expression(&mut self, expr: &crate::ast::Expression) -> Type {
        use crate::ast::{BinaryOp, Expression, PrimitiveType, UnaryOp};

        match expr {
            Expression::Literal(lit) => {
                use crate::ast::Literal;
                match lit {
                    Literal::Int(_) => Type::Primitive(PrimitiveType::I32),
                    Literal::Float(_) => Type::Primitive(PrimitiveType::F64),
                    // A Rust string literal is `&str`.
                    Literal::String(_) => Type::Reference {
                        ty: Box::new(Type::Ident(Ident::new("str"))),
                        mutable: false,
                    },
                    Literal::Char(_) => Type::Primitive(PrimitiveType::Char),
                    Literal::Bool(_) => Type::Primitive(PrimitiveType::Bool),
                    Literal::Null => {
                        // NULL is typed as Option<_> where _ is inferred from context
                        // For now, we'll use a generic Option type
                        Type::Generic {
                            base: Box::new(Type::Ident(Ident::new("Option"))),
                            args: vec![Type::Auto],
                        }
                    }
                }
            }

            Expression::Ident(ident) => {
                // Look up the identifier in the symbol table
                if let Some(symbol) = self.symbol_table.lookup(&ident.name) {
                    symbol.ty.clone()
                } else {
                    self.errors.push(SemanticError::new(
                        Span::new(
                            crate::error::Position::new(0, 0),
                            crate::error::Position::new(0, 0),
                        ),
                        SemanticErrorKind::UndefinedVariable,
                        format!("undefined variable '{}'", ident.name),
                    ));
                    Type::Auto
                }
            }

            Expression::Binary { op, left, right } => {
                let left_type = self.analyze_expression(left);
                let right_type = self.analyze_expression(right);

                // Check type compatibility
                if !self.type_env.is_compatible(&left_type, &right_type) {
                    self.errors.push(SemanticError::new(
                        Span::new(
                            crate::error::Position::new(0, 0),
                            crate::error::Position::new(0, 0),
                        ),
                        SemanticErrorKind::TypeMismatch,
                        format!(
                            "binary operation type mismatch: {:?} and {:?}",
                            left_type, right_type
                        ),
                    ));
                }

                // Determine result type based on operator
                match op {
                    BinaryOp::Add
                    | BinaryOp::Sub
                    | BinaryOp::Mul
                    | BinaryOp::Div
                    | BinaryOp::Mod => left_type,
                    BinaryOp::Eq
                    | BinaryOp::Ne
                    | BinaryOp::Lt
                    | BinaryOp::Gt
                    | BinaryOp::Le
                    | BinaryOp::Ge => Type::Primitive(PrimitiveType::Bool),
                    BinaryOp::And | BinaryOp::Or => Type::Primitive(PrimitiveType::Bool),
                    BinaryOp::BitAnd
                    | BinaryOp::BitOr
                    | BinaryOp::BitXor
                    | BinaryOp::Shl
                    | BinaryOp::Shr => left_type,
                    BinaryOp::Assign
                    | BinaryOp::AddAssign
                    | BinaryOp::SubAssign
                    | BinaryOp::MulAssign
                    | BinaryOp::DivAssign
                    | BinaryOp::ModAssign
                    | BinaryOp::BitAndAssign
                    | BinaryOp::BitOrAssign
                    | BinaryOp::BitXorAssign
                    | BinaryOp::ShlAssign
                    | BinaryOp::ShrAssign => left_type,
                }
            }

            Expression::Unary {
                op,
                expr: inner_expr,
            } => {
                let expr_type = self.analyze_expression(inner_expr);

                match op {
                    UnaryOp::Not => Type::Primitive(PrimitiveType::Bool),
                    UnaryOp::Neg => expr_type,
                    UnaryOp::Ref => Type::Reference {
                        ty: Box::new(expr_type),
                        mutable: false,
                    },
                    UnaryOp::Deref => match expr_type {
                        Type::Pointer { ty, .. } | Type::Reference { ty, .. } => *ty,
                        _ => {
                            self.errors.push(SemanticError::new(
                                Span::new(
                                    crate::error::Position::new(0, 0),
                                    crate::error::Position::new(0, 0),
                                ),
                                SemanticErrorKind::InvalidOperation,
                                "cannot dereference non-pointer type".to_string(),
                            ));
                            Type::Auto
                        }
                    },
                    UnaryOp::PreInc | UnaryOp::PreDec | UnaryOp::PostInc | UnaryOp::PostDec => {
                        expr_type
                    }
                }
            }

            Expression::Call { func, args } => {
                let func_type = self.analyze_expression(func);

                // Analyze argument types
                let arg_types: Vec<Type> = args
                    .iter()
                    .map(|arg| self.analyze_expression(arg))
                    .collect();

                // Check if function type is valid
                match func_type {
                    Type::Function {
                        params,
                        return_type,
                    } => {
                        // Check argument count
                        if params.len() != arg_types.len() {
                            self.errors.push(SemanticError::new(
                                Span::new(
                                    crate::error::Position::new(0, 0),
                                    crate::error::Position::new(0, 0),
                                ),
                                SemanticErrorKind::TypeMismatch,
                                format!(
                                    "function call argument count mismatch: expected {}, found {}",
                                    params.len(),
                                    arg_types.len()
                                ),
                            ));
                        } else {
                            // Check argument types
                            for (i, (param_type, arg_type)) in
                                params.iter().zip(arg_types.iter()).enumerate()
                            {
                                // Special handling for function types (nested functions as arguments)
                                let compatible = match (arg_type, param_type) {
                                    (Type::Function { .. }, Type::Function { .. }) => {
                                        self.check_function_type_compatibility(arg_type, param_type)
                                    }
                                    _ => self.type_env.is_compatible(param_type, arg_type),
                                };

                                if !compatible {
                                    self.errors.push(SemanticError::new(
                                        Span::new(
                                            crate::error::Position::new(0, 0),
                                            crate::error::Position::new(0, 0),
                                        ),
                                        SemanticErrorKind::TypeMismatch,
                                        format!(
                                            "function call argument {} type mismatch: expected {:?}, found {:?}",
                                            i + 1, param_type, arg_type
                                        ),
                                    ));
                                }
                            }
                        }

                        *return_type
                    }
                    _ => {
                        self.errors.push(SemanticError::new(
                            Span::new(
                                crate::error::Position::new(0, 0),
                                crate::error::Position::new(0, 0),
                            ),
                            SemanticErrorKind::InvalidOperation,
                            "cannot call non-function type".to_string(),
                        ));
                        Type::Auto
                    }
                }
            }

            Expression::FieldAccess {
                expr: obj_expr,
                field,
            } => {
                let obj_type = self.analyze_expression(obj_expr);

                // Dereference if it's a reference type
                let actual_type = match &obj_type {
                    Type::Reference { ty, .. } => ty.as_ref().clone(),
                    Type::Pointer { ty, .. } => ty.as_ref().clone(),
                    other => other.clone(),
                };

                // Look up field in struct type
                match actual_type {
                    Type::Ident(ref type_ident) => {
                        if let Some(type_info) = self.type_env.get_type(&type_ident.name) {
                            match &type_info.kind {
                                TypeKind::Struct { fields } => {
                                    if let Some((_, field_type)) =
                                        fields.iter().find(|(name, _)| name == &field.name)
                                    {
                                        field_type.clone()
                                    } else {
                                        self.errors.push(SemanticError::new(
                                            Span::new(
                                                crate::error::Position::new(0, 0),
                                                crate::error::Position::new(0, 0),
                                            ),
                                            SemanticErrorKind::InvalidOperation,
                                            format!("field '{}' not found in struct", field.name),
                                        ));
                                        Type::Auto
                                    }
                                }
                                _ => {
                                    self.errors.push(SemanticError::new(
                                        Span::new(
                                            crate::error::Position::new(0, 0),
                                            crate::error::Position::new(0, 0),
                                        ),
                                        SemanticErrorKind::InvalidOperation,
                                        "field access on non-struct type".to_string(),
                                    ));
                                    Type::Auto
                                }
                            }
                        } else {
                            Type::Auto
                        }
                    }
                    _ => {
                        self.errors.push(SemanticError::new(
                            Span::new(
                                crate::error::Position::new(0, 0),
                                crate::error::Position::new(0, 0),
                            ),
                            SemanticErrorKind::InvalidOperation,
                            "field access on non-struct type".to_string(),
                        ));
                        Type::Auto
                    }
                }
            }

            Expression::Index {
                expr: array_expr,
                index: index_expr,
            } => {
                let array_type = self.analyze_expression(array_expr);
                let index_type = self.analyze_expression(index_expr);

                // Index should be an integer type
                match index_type {
                    Type::Primitive(PrimitiveType::I8)
                    | Type::Primitive(PrimitiveType::I16)
                    | Type::Primitive(PrimitiveType::I32)
                    | Type::Primitive(PrimitiveType::I64)
                    | Type::Primitive(PrimitiveType::I128)
                    | Type::Primitive(PrimitiveType::U8)
                    | Type::Primitive(PrimitiveType::U16)
                    | Type::Primitive(PrimitiveType::U32)
                    | Type::Primitive(PrimitiveType::U64)
                    | Type::Primitive(PrimitiveType::U128)
                    | Type::Primitive(PrimitiveType::Int) => {}
                    _ => {
                        self.errors.push(SemanticError::new(
                            Span::new(
                                crate::error::Position::new(0, 0),
                                crate::error::Position::new(0, 0),
                            ),
                            SemanticErrorKind::TypeMismatch,
                            format!("array index must be integer type, found {:?}", index_type),
                        ));
                    }
                }

                // Get element type from array/slice
                match array_type {
                    Type::Array { ty, .. } | Type::Slice { ty } => *ty,
                    _ => {
                        self.errors.push(SemanticError::new(
                            Span::new(
                                crate::error::Position::new(0, 0),
                                crate::error::Position::new(0, 0),
                            ),
                            SemanticErrorKind::InvalidOperation,
                            "cannot index non-array type".to_string(),
                        ));
                        Type::Auto
                    }
                }
            }

            Expression::Cast {
                expr: cast_expr,
                ty,
            } => {
                // Analyze the expression being cast
                let expr_type = self.analyze_expression(cast_expr);

                // Resolve both types to handle typedef aliases
                let resolved_expr_type = self.type_env.resolve_type(&expr_type);
                let resolved_target_type = self.type_env.resolve_type(ty);

                // Check if cast is valid
                // Allow casts between:
                // 1. Compatible types (including through typedef)
                // 2. Numeric types
                // 3. Pointer types
                let is_valid_cast = self
                    .type_env
                    .is_compatible(&resolved_expr_type, &resolved_target_type)
                    || matches!(
                        (&resolved_expr_type, &resolved_target_type),
                        (Type::Primitive(_), Type::Primitive(_))
                            | (Type::Pointer { .. }, Type::Pointer { .. })
                            | (Type::Primitive(_), Type::Pointer { .. })
                            | (Type::Pointer { .. }, Type::Primitive(_))
                    );

                if !is_valid_cast {
                    self.errors.push(SemanticError::new(
                        Span::new(
                            crate::error::Position::new(0, 0),
                            crate::error::Position::new(0, 0),
                        ),
                        SemanticErrorKind::InvalidOperation,
                        format!("invalid cast from {:?} to {:?}", expr_type, ty),
                    ));
                }

                ty.clone()
            }

            Expression::Sizeof { ty: _ } => {
                // sizeof returns usize (u64 in our case)
                Type::Primitive(PrimitiveType::U64)
            }

            Expression::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                let cond_type = self.analyze_expression(condition);
                let then_type = self.analyze_expression(then_expr);
                let else_type = self.analyze_expression(else_expr);

                // Condition should be boolean
                if !self
                    .type_env
                    .is_compatible(&Type::Primitive(PrimitiveType::Bool), &cond_type)
                {
                    self.errors.push(SemanticError::new(
                        Span::new(
                            crate::error::Position::new(0, 0),
                            crate::error::Position::new(0, 0),
                        ),
                        SemanticErrorKind::TypeMismatch,
                        format!("ternary condition must be boolean, found {:?}", cond_type),
                    ));
                }

                // Both branches should have compatible types
                if !self.type_env.is_compatible(&then_type, &else_type) {
                    self.errors.push(SemanticError::new(
                        Span::new(
                            crate::error::Position::new(0, 0),
                            crate::error::Position::new(0, 0),
                        ),
                        SemanticErrorKind::TypeMismatch,
                        format!(
                            "ternary branches have incompatible types: {:?} and {:?}",
                            then_type, else_type
                        ),
                    ));
                }

                then_type
            }

            Expression::StructInit { ty, fields } => {
                // Analyze field initializers
                for (_, field_expr) in fields {
                    self.analyze_expression(field_expr);
                }

                // If type is Auto, try to infer from context
                // For now, just return the type as-is
                // TODO: Implement type inference from context
                ty.clone()
            }

            Expression::ArrayLit { elements } => {
                if elements.is_empty() {
                    Type::Array {
                        ty: Box::new(Type::Auto),
                        size: Some(0),
                    }
                } else {
                    let first_type = self.analyze_expression(&elements[0]);

                    // Check all elements have the same type
                    for elem in &elements[1..] {
                        let elem_type = self.analyze_expression(elem);
                        if !self.type_env.is_compatible(&first_type, &elem_type) {
                            self.errors.push(SemanticError::new(
                                Span::new(
                                    crate::error::Position::new(0, 0),
                                    crate::error::Position::new(0, 0),
                                ),
                                SemanticErrorKind::TypeMismatch,
                                format!(
                                    "array elements have incompatible types: {:?} and {:?}",
                                    first_type, elem_type
                                ),
                            ));
                        }
                    }

                    Type::Array {
                        ty: Box::new(first_type),
                        size: Some(elements.len()),
                    }
                }
            }

            Expression::TupleLit { elements } => {
                let types: Vec<Type> = elements
                    .iter()
                    .map(|e| self.analyze_expression(e))
                    .collect();
                Type::Tuple { types }
            }

            Expression::Range {
                start,
                end,
                inclusive: _,
            } => {
                // A range's element type is the type of its endpoints. Analyze
                // both ends; prefer a concrete endpoint type over Auto so a
                // for-in over `0..n` binds the loop variable to the integer
                // element type rather than Auto.
                let start_ty = start.as_ref().map(|e| self.analyze_expression(e));
                let end_ty = end.as_ref().map(|e| self.analyze_expression(e));
                match (start_ty, end_ty) {
                    (Some(t), _) if t != Type::Auto => t,
                    (_, Some(t)) if t != Type::Auto => t,
                    (Some(t), _) => t,
                    (_, Some(t)) => t,
                    (None, None) => Type::Auto,
                }
            }

            Expression::MacroCall { .. } => {
                // Macro calls are not type-checked at this stage
                Type::Auto
            }

            Expression::RustBlock { .. } => {
                // Rust blocks are not type-checked at this stage
                Type::Auto
            }

            Expression::ErrorProp { expr: inner_expr } => {
                let expr_type = self.analyze_expression(inner_expr);

                // Error propagation should be on fallible types
                match expr_type {
                    Type::Fallible { ty } => *ty,
                    _ => {
                        self.errors.push(SemanticError::new(
                            Span::new(
                                crate::error::Position::new(0, 0),
                                crate::error::Position::new(0, 0),
                            ),
                            SemanticErrorKind::InvalidOperation,
                            "error propagation operator (!) can only be used on fallible types"
                                .to_string(),
                        ));
                        Type::Auto
                    }
                }
            }

            Expression::MethodCall {
                receiver,
                method: _,
                args,
            } => {
                let _receiver_type = self.analyze_expression(receiver);

                // Analyze arguments
                for arg in args {
                    self.analyze_expression(arg);
                }

                // Method call type checking is simplified for now
                Type::Auto
            }

            Expression::TypeScopedCall {
                ty,
                method: _,
                args,
            } => {
                // Analyze arguments
                for arg in args {
                    self.analyze_expression(arg);
                }

                // Type-scoped call returns the type (simplified)
                ty.clone()
            }

            Expression::ExplicitGenericCall {
                ty,
                generics: _,
                method: _,
                args,
            } => {
                // Analyze arguments
                for arg in args {
                    self.analyze_expression(arg);
                }

                // Explicit generic call returns the type (simplified)
                ty.clone()
            }

            Expression::Comma { left, right } => {
                // Analyze both expressions
                self.analyze_expression(left);
                // Comma expression returns the type of the right expression
                self.analyze_expression(right)
            }

            Expression::Loop { body, .. } => {
                // Analyze the body so errors inside it (e.g. undefined
                // variables) are reported.
                self.symbol_table.enter_scope();
                self.analyze_block(body);
                self.symbol_table.exit_scope();
                // A loop expression yields the type of its first
                // `break <value>`; with no value-carrying break it yields unit.
                match Self::first_break_value(body) {
                    Some(value) => self.analyze_expression(value),
                    None => Type::Primitive(PrimitiveType::Void),
                }
            }
        }
    }

    /// Find the first `break <value>` directly within a loop body (not
    /// descending into nested loops, whose breaks bind to the inner loop).
    /// Returns the break's value expression, if any.
    fn first_break_value(block: &crate::ast::Block) -> Option<&crate::ast::Expression> {
        for stmt in &block.statements {
            if let Some(v) = Self::first_break_value_in_statement(stmt) {
                return Some(v);
            }
        }
        None
    }

    fn first_break_value_in_statement(
        stmt: &crate::ast::Statement,
    ) -> Option<&crate::ast::Expression> {
        use crate::ast::Statement;
        match stmt {
            Statement::Break { value, .. } => value.as_ref(),
            // Descend into conditional/branch bodies, which belong to THIS loop.
            Statement::If { then_block, else_block, .. } => Self::first_break_value(then_block)
                .or_else(|| else_block.as_ref().and_then(Self::first_break_value)),
            // Do NOT descend into nested loops/while/for: their breaks bind
            // to the inner loop, not this one.
            _ => None,
        }
    }

    // Test-only accessors for white-box unit tests in `semantic_unit_tests`.
    // `#[allow(dead_code)]`: the bin target compiles this module's `#[cfg(test)]`
    // code but not the sibling test module that uses these, so it sees them as
    // unused; the lib target does exercise them.

    /// Get the symbol table (for testing)
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    /// Get the type environment (for testing)
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn type_env(&self) -> &TypeEnvironment {
        &self.type_env
    }

    /// Get the errors (for testing)
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn errors(&self) -> &[SemanticError] {
        &self.errors
    }

    /// Analyze an expression (for testing)
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn analyze_expression_test(&mut self, expr: &crate::ast::Expression) -> Type {
        self.analyze_expression(expr)
    }

    /// Analyze a statement (for testing)
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn analyze_statement_test(&mut self, statement: &crate::ast::Statement) {
        self.analyze_statement(statement);
    }

    /// Analyze a function (for testing)
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn analyze_function_test(&mut self, func: &crate::ast::Function) {
        self.analyze_function(func);
    }

    /// Analyze a struct (for testing)
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn analyze_struct_test(&mut self, struct_def: &crate::ast::Struct) {
        self.analyze_struct(struct_def);
    }

    /// Analyze an enum (for testing)
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn analyze_enum_test(&mut self, enum_def: &crate::ast::Enum) {
        self.analyze_enum(enum_def);
    }

    /// Analyze a typedef (for testing)
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn analyze_typedef_test(&mut self, typedef: &crate::ast::Typedef) {
        self.analyze_typedef(typedef);
    }

    /// Analyze a macro definition (for testing)
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn analyze_macro_definition_test(&mut self, macro_def: &crate::ast::MacroDefinition) {
        self.analyze_macro_definition(macro_def);
    }

    /// Clear accumulated errors (for testing)
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn clear_errors_for_test(&mut self) {
        self.errors.clear();
    }

    /// Check for unsupported C union feature
    #[allow(dead_code)]
    pub fn check_union_usage(&mut self, name: &str) {
        self.errors.push(SemanticError::new(
            Span::new(
                crate::error::Position::new(0, 0),
                crate::error::Position::new(0, 0),
            ),
            SemanticErrorKind::UnsupportedFeature,
            format!(
                "C unions are not supported in Crusty. Union '{}' cannot be used because Rust does not have direct union support with the same semantics as C. Consider using an enum with variants instead.",
                name
            ),
        ));
    }

    /// Check for unsupported goto statement
    #[allow(dead_code)]
    pub fn check_goto_usage(&mut self, label: &str) {
        self.errors.push(SemanticError::new(
            Span::new(
                crate::error::Position::new(0, 0),
                crate::error::Position::new(0, 0),
            ),
            SemanticErrorKind::UnsupportedFeature,
            format!(
                "goto statements are not supported in Crusty. goto to label '{}' cannot be used because Rust does not support goto. Use structured control flow (loops, if/else, match) instead.",
                label
            ),
        ));
    }

    /// Check for unsupported #include directive
    #[allow(dead_code)]
    pub fn check_include_usage(&mut self, path: &str) {
        self.errors.push(SemanticError::new(
            Span::new(
                crate::error::Position::new(0, 0),
                crate::error::Position::new(0, 0),
            ),
            SemanticErrorKind::UnsupportedFeature,
            format!(
                "#include directives are not supported in Crusty. #include '{}' cannot be used because Crusty uses a module system similar to Rust. Use #use directives to import modules instead.",
                path
            ),
        ));
    }

    /// Collect all variables currently in scope (for capture analysis)
    fn collect_variables_in_scope(&self) -> HashMap<String, (Type, bool)> {
        let mut variables = HashMap::new();

        // Iterate through all scopes from outermost to innermost
        for scope in &self.symbol_table.scopes {
            for (name, symbol) in &scope.symbols {
                if symbol.kind == SymbolKind::Variable {
                    variables.insert(name.clone(), (symbol.ty.clone(), symbol.mutable));
                }
            }
        }

        variables
    }

    /// Analyze which variables are used in an expression
    fn collect_used_variables(
        &self,
        expr: &crate::ast::Expression,
        used: &mut std::collections::HashSet<String>,
    ) {
        use crate::ast::Expression;

        match expr {
            Expression::Ident(ident) => {
                used.insert(ident.name.clone());
            }
            Expression::Binary { left, right, .. } => {
                self.collect_used_variables(left, used);
                self.collect_used_variables(right, used);
            }
            Expression::Unary { expr, .. } => {
                self.collect_used_variables(expr, used);
            }
            Expression::Call { func, args } => {
                self.collect_used_variables(func, used);
                for arg in args {
                    self.collect_used_variables(arg, used);
                }
            }
            Expression::Index { expr, index } => {
                self.collect_used_variables(expr, used);
                self.collect_used_variables(index, used);
            }
            Expression::FieldAccess { expr, .. } => {
                self.collect_used_variables(expr, used);
            }
            Expression::Cast { expr, .. } => {
                self.collect_used_variables(expr, used);
            }
            Expression::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                self.collect_used_variables(condition, used);
                self.collect_used_variables(then_expr, used);
                self.collect_used_variables(else_expr, used);
            }
            Expression::ArrayLit { elements } => {
                for elem in elements {
                    self.collect_used_variables(elem, used);
                }
            }
            Expression::TupleLit { elements } => {
                for elem in elements {
                    self.collect_used_variables(elem, used);
                }
            }
            Expression::StructInit { fields, .. } => {
                for (_, field_expr) in fields {
                    self.collect_used_variables(field_expr, used);
                }
            }
            Expression::Range { start, end, .. } => {
                if let Some(s) = start {
                    self.collect_used_variables(s, used);
                }
                if let Some(e) = end {
                    self.collect_used_variables(e, used);
                }
            }
            Expression::Sizeof { .. } => {
                // Sizeof only uses types, not variables
            }
            Expression::MacroCall { .. } => {
                // Macro calls are opaque - we can't analyze them
            }
            Expression::RustBlock { .. } => {
                // Rust blocks are opaque - we can't analyze them
            }
            Expression::ErrorProp { expr } => {
                self.collect_used_variables(expr, used);
            }
            Expression::MethodCall { receiver, args, .. } => {
                self.collect_used_variables(receiver, used);
                for arg in args {
                    self.collect_used_variables(arg, used);
                }
            }
            Expression::TypeScopedCall { args, .. } => {
                for arg in args {
                    self.collect_used_variables(arg, used);
                }
            }
            Expression::ExplicitGenericCall { args, .. } => {
                for arg in args {
                    self.collect_used_variables(arg, used);
                }
            }
            Expression::Comma { left, right } => {
                self.collect_used_variables(left, used);
                self.collect_used_variables(right, used);
            }
            Expression::Loop { body, .. } => {
                self.collect_used_variables_in_block(body, used);
            }
            Expression::Literal(_) => {
                // Literals don't use variables
            }
        }
    }

    /// Analyze which variables are used in a statement
    fn collect_used_variables_in_statement(
        &self,
        stmt: &crate::ast::Statement,
        used: &mut std::collections::HashSet<String>,
    ) {
        use crate::ast::Statement;

        match stmt {
            Statement::Let { init, .. } | Statement::Var { init, .. } => {
                if let Some(expr) = init {
                    self.collect_used_variables(expr, used);
                }
            }
            Statement::Const { value, .. } => {
                self.collect_used_variables(value, used);
            }
            Statement::Expr(expr) => {
                self.collect_used_variables(expr, used);
            }
            Statement::Return(Some(expr)) => {
                self.collect_used_variables(expr, used);
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.collect_used_variables(condition, used);
                self.collect_used_variables_in_block(then_block, used);
                if let Some(else_blk) = else_block {
                    self.collect_used_variables_in_block(else_blk, used);
                }
            }
            Statement::While {
                condition, body, ..
            } => {
                self.collect_used_variables(condition, used);
                self.collect_used_variables_in_block(body, used);
            }
            Statement::Loop { body, .. } => {
                self.collect_used_variables_in_block(body, used);
            }
            Statement::For {
                init,
                condition,
                increment,
                body,
                ..
            } => {
                self.collect_used_variables_in_statement(init, used);
                self.collect_used_variables(condition, used);
                self.collect_used_variables(increment, used);
                self.collect_used_variables_in_block(body, used);
            }
            Statement::ForIn { iter, body, .. } => {
                self.collect_used_variables(iter, used);
                self.collect_used_variables_in_block(body, used);
            }
            Statement::Switch {
                expr,
                cases,
                default,
            } => {
                self.collect_used_variables(expr, used);
                for case in cases {
                    for value in &case.values {
                        self.collect_used_variables(value, used);
                    }
                    self.collect_used_variables_in_block(&case.body, used);
                }
                if let Some(default_block) = default {
                    self.collect_used_variables_in_block(default_block, used);
                }
            }
            Statement::NestedFunction { body, .. } => {
                // Don't traverse into nested functions - they have their own scope
                self.collect_used_variables_in_block(body, used);
            }
            Statement::Break { value, .. } => {
                if let Some(expr) = value {
                    self.collect_used_variables(expr, used);
                }
            }
            Statement::Return(None) | Statement::Continue(_) => {
                // No variables used
            }
        }
    }

    /// Analyze which variables are used in a block
    fn collect_used_variables_in_block(
        &self,
        block: &crate::ast::Block,
        used: &mut std::collections::HashSet<String>,
    ) {
        for stmt in &block.statements {
            self.collect_used_variables_in_statement(stmt, used);
        }
    }

    /// Analyze which variables are modified in an expression (for mutable capture detection)
    fn collect_modified_variables(
        &self,
        expr: &crate::ast::Expression,
        modified: &mut std::collections::HashSet<String>,
    ) {
        use crate::ast::{BinaryOp, Expression};

        match expr {
            Expression::Binary {
                op:
                    _op @ (BinaryOp::Assign
                    | BinaryOp::AddAssign
                    | BinaryOp::SubAssign
                    | BinaryOp::MulAssign
                    | BinaryOp::DivAssign),
                left,
                ..
            } => {
                // Mark the left-hand side as modified
                if let Expression::Ident(ident) = &**left {
                    modified.insert(ident.name.clone());
                }
            }
            Expression::Unary { op, expr } => {
                // Increment/decrement operators modify the operand
                use crate::ast::UnaryOp;
                match op {
                    UnaryOp::PreInc | UnaryOp::PostInc | UnaryOp::PreDec | UnaryOp::PostDec => {
                        if let Expression::Ident(ident) = &**expr {
                            modified.insert(ident.name.clone());
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    /// Analyze which variables are modified in a statement
    fn collect_modified_variables_in_statement(
        &self,
        stmt: &crate::ast::Statement,
        modified: &mut std::collections::HashSet<String>,
    ) {
        use crate::ast::Statement;

        match stmt {
            Statement::Let { init, .. } | Statement::Var { init, .. } => {
                if let Some(expr) = init {
                    self.collect_modified_variables(expr, modified);
                }
            }
            Statement::Const { value, .. } => {
                self.collect_modified_variables(value, modified);
            }
            Statement::Expr(expr) => {
                self.collect_modified_variables(expr, modified);
            }
            Statement::Return(Some(expr)) => {
                self.collect_modified_variables(expr, modified);
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.collect_modified_variables(condition, modified);
                self.collect_modified_variables_in_block(then_block, modified);
                if let Some(else_blk) = else_block {
                    self.collect_modified_variables_in_block(else_blk, modified);
                }
            }
            Statement::While {
                condition, body, ..
            } => {
                self.collect_modified_variables(condition, modified);
                self.collect_modified_variables_in_block(body, modified);
            }
            Statement::Loop { body, .. } => {
                self.collect_modified_variables_in_block(body, modified);
            }
            Statement::For {
                init,
                condition,
                increment,
                body,
                ..
            } => {
                self.collect_modified_variables_in_statement(init, modified);
                self.collect_modified_variables(condition, modified);
                self.collect_modified_variables(increment, modified);
                self.collect_modified_variables_in_block(body, modified);
            }
            Statement::ForIn { iter, body, .. } => {
                self.collect_modified_variables(iter, modified);
                self.collect_modified_variables_in_block(body, modified);
            }
            Statement::Switch {
                expr,
                cases,
                default,
            } => {
                self.collect_modified_variables(expr, modified);
                for case in cases {
                    for value in &case.values {
                        self.collect_modified_variables(value, modified);
                    }
                    self.collect_modified_variables_in_block(&case.body, modified);
                }
                if let Some(default_block) = default {
                    self.collect_modified_variables_in_block(default_block, modified);
                }
            }
            Statement::NestedFunction { body, .. } => {
                self.collect_modified_variables_in_block(body, modified);
            }
            Statement::Return(None) | Statement::Break { .. } | Statement::Continue(_) => {
                // No variables modified
            }
        }
    }

    /// Analyze which variables are modified in a block
    fn collect_modified_variables_in_block(
        &self,
        block: &crate::ast::Block,
        modified: &mut std::collections::HashSet<String>,
    ) {
        for stmt in &block.statements {
            self.collect_modified_variables_in_statement(stmt, modified);
        }
    }

    /// Get the captures for a nested function
    #[allow(dead_code)]
    pub fn get_captures(&self, function_name: &str) -> Option<&Vec<Capture>> {
        self.nested_function_captures.get(function_name)
    }

    /// Get all nested function captures
    #[allow(dead_code)] // Used in tests
    pub fn get_all_captures(&self) -> &HashMap<String, Vec<Capture>> {
        &self.nested_function_captures
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
