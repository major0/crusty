// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Semantic analysis module for type checking and validation.

use crate::ast::Type;
use crate::error::{SemanticError, SemanticErrorKind, Span};

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

/// Semantic analyzer for type checking and validation
#[derive(Debug, Clone)]
pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
    type_env: TypeEnvironment,
    errors: Vec<SemanticError>,
}

impl SemanticAnalyzer {
    /// Create a new semantic analyzer
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            type_env: TypeEnvironment::new(),
            errors: Vec::new(),
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
            Item::Namespace(_) | Item::Use(_) | Item::Extern(_) | Item::MacroDefinition(_) => {
                // These items don't require semantic analysis in this phase
            }
        }
    }

    /// Analyze a function declaration
    fn analyze_function(&mut self, func: &crate::ast::Function) {
        use crate::ast::Visibility;
        
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

            if let Err(msg) = self.symbol_table.insert(param.name.name.clone(), param_symbol) {
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

        // Analyze function body
        self.analyze_block(&func.body);

        // Exit function scope
        self.symbol_table.exit_scope();
    }

    /// Analyze a struct definition
    fn analyze_struct(&mut self, struct_def: &crate::ast::Struct) {
        // Register struct type in type environment
        let fields: Vec<(String, Type)> = struct_def
            .fields
            .iter()
            .map(|f| (f.name.name.clone(), f.ty.clone()))
            .collect();

        let type_info = TypeInfo::new(
            struct_def.name.name.clone(),
            TypeKind::Struct { fields },
        );

        self.type_env.register_type(struct_def.name.name.clone(), type_info);

        // Register struct as a type symbol
        let symbol = Symbol::new(
            struct_def.name.name.clone(),
            Type::Ident(struct_def.name.clone()),
            SymbolKind::Type,
            false,
        );

        if let Err(msg) = self.symbol_table.insert(struct_def.name.name.clone(), symbol) {
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

        let type_info = TypeInfo::new(
            enum_def.name.name.clone(),
            TypeKind::Enum { variants },
        );

        self.type_env.register_type(enum_def.name.name.clone(), type_info);

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
        // Register type alias in type environment
        let type_info = TypeInfo::new(
            typedef.name.name.clone(),
            TypeKind::Alias {
                target: typedef.target.clone(),
            },
        );

        self.type_env.register_type(typedef.name.name.clone(), type_info);

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

        if let Err(msg) = self.symbol_table.insert(const_def.name.name.clone(), symbol) {
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

        if let Err(msg) = self.symbol_table.insert(static_def.name.name.clone(), symbol) {
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
            Statement::Let { name, ty, init, mutable } => {
                // Analyze initialization expression if present
                let init_type = if let Some(ref init_expr) = init {
                    self.analyze_expression(init_expr)
                } else {
                    Type::Auto
                };

                // Determine the variable type
                let var_type = if let Some(ref declared_type) = ty {
                    // Check type compatibility if both type and init are present
                    if init.is_some() && !self.type_env.is_compatible(declared_type, &init_type) {
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
                    declared_type.clone()
                } else {
                    init_type
                };

                // Register variable in symbol table
                let symbol = Symbol::new(
                    name.name.clone(),
                    var_type,
                    SymbolKind::Variable,
                    *mutable,
                );

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
                    if init.is_some() && !self.type_env.is_compatible(declared_type, &init_type) {
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
                    declared_type.clone()
                } else {
                    init_type
                };

                // Register variable in symbol table (var is always mutable)
                let symbol = Symbol::new(
                    name.name.clone(),
                    var_type,
                    SymbolKind::Variable,
                    true,
                );

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
                let symbol = Symbol::new(
                    name.name.clone(),
                    ty.clone(),
                    SymbolKind::Const,
                    false,
                );

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
                // Analyze the return expression if present
                if let Some(ref return_expr) = expr {
                    self.analyze_expression(return_expr);
                }
            }

            Statement::If { condition, then_block, else_block } => {
                // Analyze condition
                let cond_type = self.analyze_expression(condition);
                
                // Condition should be boolean
                if !self.type_env.is_compatible(&Type::Primitive(crate::ast::PrimitiveType::Bool), &cond_type) {
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

            Statement::While { label: _, condition, body } => {
                // Analyze condition
                let cond_type = self.analyze_expression(condition);
                
                // Condition should be boolean
                if !self.type_env.is_compatible(&Type::Primitive(crate::ast::PrimitiveType::Bool), &cond_type) {
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

            Statement::For { label: _, init, condition, increment, body } => {
                // Enter scope for the for loop
                self.symbol_table.enter_scope();

                // Analyze initialization
                self.analyze_statement(init);

                // Analyze condition
                let cond_type = self.analyze_expression(condition);
                if !self.type_env.is_compatible(&Type::Primitive(crate::ast::PrimitiveType::Bool), &cond_type) {
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

            Statement::ForIn { label: _, var, iter, body } => {
                // Enter scope for the for-in loop
                self.symbol_table.enter_scope();

                // Analyze iterator expression
                let iter_type = self.analyze_expression(iter);

                // Register loop variable (type inference from iterator)
                let symbol = Symbol::new(
                    var.name.clone(),
                    iter_type,
                    SymbolKind::Variable,
                    false,
                );

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

            Statement::Switch { expr, cases, default } => {
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

            Statement::Break(_) | Statement::Continue(_) => {
                // No semantic analysis needed for break/continue
            }
        }
    }

    /// Analyze an expression and return its type (placeholder for sub-task 8.4)
    fn analyze_expression(&mut self, expr: &crate::ast::Expression) -> Type {
        use crate::ast::{Expression, BinaryOp, UnaryOp, PrimitiveType};
        
        match expr {
            Expression::Literal(lit) => {
                use crate::ast::Literal;
                match lit {
                    Literal::Int(_) => Type::Primitive(PrimitiveType::I32),
                    Literal::Float(_) => Type::Primitive(PrimitiveType::F64),
                    Literal::String(_) => Type::Reference {
                        ty: Box::new(Type::Primitive(PrimitiveType::Char)),
                        mutable: false,
                    },
                    Literal::Char(_) => Type::Primitive(PrimitiveType::Char),
                    Literal::Bool(_) => Type::Primitive(PrimitiveType::Bool),
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
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                        left_type
                    }
                    BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => {
                        Type::Primitive(PrimitiveType::Bool)
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        Type::Primitive(PrimitiveType::Bool)
                    }
                    BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor | BinaryOp::Shl | BinaryOp::Shr => {
                        left_type
                    }
                    BinaryOp::Assign | BinaryOp::AddAssign | BinaryOp::SubAssign | BinaryOp::MulAssign | BinaryOp::DivAssign => {
                        left_type
                    }
                }
            }

            Expression::Unary { op, expr: inner_expr } => {
                let expr_type = self.analyze_expression(inner_expr);

                match op {
                    UnaryOp::Not => Type::Primitive(PrimitiveType::Bool),
                    UnaryOp::Neg => expr_type,
                    UnaryOp::Ref => Type::Reference {
                        ty: Box::new(expr_type),
                        mutable: false,
                    },
                    UnaryOp::Deref => {
                        match expr_type {
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
                        }
                    }
                    UnaryOp::PreInc | UnaryOp::PreDec | UnaryOp::PostInc | UnaryOp::PostDec => expr_type,
                }
            }

            Expression::Call { func, args } => {
                let func_type = self.analyze_expression(func);

                // Analyze argument types
                let arg_types: Vec<Type> = args.iter().map(|arg| self.analyze_expression(arg)).collect();

                // Check if function type is valid
                match func_type {
                    Type::Function { params, return_type } => {
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
                            for (i, (param_type, arg_type)) in params.iter().zip(arg_types.iter()).enumerate() {
                                if !self.type_env.is_compatible(param_type, arg_type) {
                                    self.errors.push(SemanticError::new(
                                        Span::new(
                                            crate::error::Position::new(0, 0),
                                            crate::error::Position::new(0, 0),
                                        ),
                                        SemanticErrorKind::TypeMismatch,
                                        format!(
                                            "function call argument {} type mismatch: expected {:?}, found {:?}",
                                            i, param_type, arg_type
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

            Expression::FieldAccess { expr: obj_expr, field } => {
                let obj_type = self.analyze_expression(obj_expr);

                // Look up field in struct type
                match obj_type {
                    Type::Ident(ref type_ident) => {
                        if let Some(type_info) = self.type_env.get_type(&type_ident.name) {
                            match &type_info.kind {
                                TypeKind::Struct { fields } => {
                                    if let Some((_, field_type)) = fields.iter().find(|(name, _)| name == &field.name) {
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

            Expression::Index { expr: array_expr, index: index_expr } => {
                let array_type = self.analyze_expression(array_expr);
                let index_type = self.analyze_expression(index_expr);

                // Index should be an integer type
                match index_type {
                    Type::Primitive(PrimitiveType::I32)
                    | Type::Primitive(PrimitiveType::I64)
                    | Type::Primitive(PrimitiveType::U32)
                    | Type::Primitive(PrimitiveType::U64)
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

            Expression::Cast { expr: cast_expr, ty } => {
                // Analyze the expression being cast
                let expr_type = self.analyze_expression(cast_expr);

                // Basic cast validation (can be expanded)
                // For now, allow casts between numeric types
                match (&expr_type, ty) {
                    (Type::Primitive(_), Type::Primitive(_)) => {}
                    _ => {
                        self.errors.push(SemanticError::new(
                            Span::new(
                                crate::error::Position::new(0, 0),
                                crate::error::Position::new(0, 0),
                            ),
                            SemanticErrorKind::InvalidOperation,
                            format!("invalid cast from {:?} to {:?}", expr_type, ty),
                        ));
                    }
                }

                ty.clone()
            }

            Expression::Sizeof { ty } => {
                // sizeof returns usize (u64 in our case)
                Type::Primitive(PrimitiveType::U64)
            }

            Expression::Ternary { condition, then_expr, else_expr } => {
                let cond_type = self.analyze_expression(condition);
                let then_type = self.analyze_expression(then_expr);
                let else_type = self.analyze_expression(else_expr);

                // Condition should be boolean
                if !self.type_env.is_compatible(&Type::Primitive(PrimitiveType::Bool), &cond_type) {
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
                let types: Vec<Type> = elements.iter().map(|e| self.analyze_expression(e)).collect();
                Type::Tuple { types }
            }

            Expression::Range { start, end, inclusive: _ } => {
                // Analyze start and end expressions if present
                if let Some(ref start_expr) = start {
                    self.analyze_expression(start_expr);
                }
                if let Some(ref end_expr) = end {
                    self.analyze_expression(end_expr);
                }

                // Range type (simplified)
                Type::Auto
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
                            "error propagation operator (!) can only be used on fallible types".to_string(),
                        ));
                        Type::Auto
                    }
                }
            }

            Expression::MethodCall { receiver, method: _, args } => {
                let receiver_type = self.analyze_expression(receiver);
                
                // Analyze arguments
                for arg in args {
                    self.analyze_expression(arg);
                }

                // Method call type checking is simplified for now
                Type::Auto
            }

            Expression::TypeScopedCall { ty, method: _, args } => {
                // Analyze arguments
                for arg in args {
                    self.analyze_expression(arg);
                }

                // Type-scoped call returns the type (simplified)
                ty.clone()
            }

            Expression::ExplicitGenericCall { ty, generics: _, method: _, args } => {
                // Analyze arguments
                for arg in args {
                    self.analyze_expression(arg);
                }

                // Explicit generic call returns the type (simplified)
                ty.clone()
            }
        }
    }

    /// Get the symbol table (for testing)
    #[cfg(test)]
    pub fn symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    /// Get the type environment (for testing)
    #[cfg(test)]
    pub fn type_env(&self) -> &TypeEnvironment {
        &self.type_env
    }

    /// Get the errors (for testing)
    #[cfg(test)]
    pub fn errors(&self) -> &[SemanticError] {
        &self.errors
    }

    /// Check for unsupported C union feature
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
}

impl Default for SemanticAnalyzer {
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

    // Semantic Analyzer Tests

    #[test]
    fn test_semantic_analyzer_creation() {
        let analyzer = SemanticAnalyzer::new();
        assert_eq!(analyzer.errors().len(), 0);
        assert_eq!(analyzer.symbol_table().scopes.len(), 1);
    }

    #[test]
    fn test_semantic_analyzer_analyze_empty_file() {
        let mut analyzer = SemanticAnalyzer::new();
        let file = crate::ast::File {
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
        use crate::ast::{Function, Visibility, Block, PrimitiveType};
        
        let mut analyzer = SemanticAnalyzer::new();
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("test_func"),
            params: vec![],
            return_type: Some(Type::Primitive(PrimitiveType::I32)),
            body: Block::empty(),
            doc_comments: vec![],
        attributes: vec![],
        };
        
        analyzer.analyze_function(&func);
        
        // Function should be registered in symbol table
        let symbol = analyzer.symbol_table().lookup("test_func");
        assert!(symbol.is_some());
        assert_eq!(symbol.unwrap().kind, SymbolKind::Function);
    }

    #[test]
    fn test_semantic_analyzer_struct_registration() {
        use crate::ast::{Struct, Visibility, Field, PrimitiveType};
        
        let mut analyzer = SemanticAnalyzer::new();
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
        
        analyzer.analyze_struct(&struct_def);
        
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
        use crate::ast::{Enum, Visibility, EnumVariant};
        
        let mut analyzer = SemanticAnalyzer::new();
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
            ],
            doc_comments: vec![],
        attributes: vec![],
        };
        
        analyzer.analyze_enum(&enum_def);
        
        // Enum should be registered as a type
        let type_info = analyzer.type_env().get_type("Color");
        assert!(type_info.is_some());
        assert!(matches!(type_info.unwrap().kind, TypeKind::Enum { .. }));
    }

    #[test]
    fn test_semantic_analyzer_duplicate_function() {
        use crate::ast::{Function, Visibility, Block, PrimitiveType};
        
        let mut analyzer = SemanticAnalyzer::new();
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("duplicate"),
            params: vec![],
            return_type: Some(Type::Primitive(PrimitiveType::I32)),
            body: Block::empty(),
            doc_comments: vec![],
        attributes: vec![],
        };
        
        analyzer.analyze_function(&func);
        assert_eq!(analyzer.errors().len(), 0);
        
        // Try to register the same function again
        analyzer.analyze_function(&func);
        assert_eq!(analyzer.errors().len(), 1);
        assert_eq!(analyzer.errors()[0].kind, SemanticErrorKind::DuplicateDefinition);
    }

    #[test]
    fn test_semantic_analyzer_let_statement() {
        use crate::ast::{Statement, Expression, Literal, PrimitiveType};
        
        let mut analyzer = SemanticAnalyzer::new();
        let stmt = Statement::Let {
            name: Ident::new("x"),
            ty: Some(Type::Primitive(PrimitiveType::I32)),
            init: Some(Expression::Literal(Literal::Int(42))),
            mutable: false,
        };
        
        analyzer.analyze_statement(&stmt);
        
        // Variable should be registered
        let symbol = analyzer.symbol_table().lookup("x");
        assert!(symbol.is_some());
        assert_eq!(symbol.unwrap().kind, SymbolKind::Variable);
        assert!(!symbol.unwrap().mutable);
    }

    #[test]
    fn test_semantic_analyzer_var_statement() {
        use crate::ast::{Statement, Expression, Literal, PrimitiveType};
        
        let mut analyzer = SemanticAnalyzer::new();
        let stmt = Statement::Var {
            name: Ident::new("y"),
            ty: Some(Type::Primitive(PrimitiveType::I32)),
            init: Some(Expression::Literal(Literal::Int(10))),
        };
        
        analyzer.analyze_statement(&stmt);
        
        // Variable should be registered as mutable
        let symbol = analyzer.symbol_table().lookup("y");
        assert!(symbol.is_some());
        assert_eq!(symbol.unwrap().kind, SymbolKind::Variable);
        assert!(symbol.unwrap().mutable);
    }

    #[test]
    fn test_semantic_analyzer_undefined_variable() {
        use crate::ast::{Statement, Expression};
        
        let mut analyzer = SemanticAnalyzer::new();
        let stmt = Statement::Expr(Expression::Ident(Ident::new("undefined_var")));
        
        analyzer.analyze_statement(&stmt);
        
        // Should detect undefined variable
        assert_eq!(analyzer.errors().len(), 1);
        assert_eq!(analyzer.errors()[0].kind, SemanticErrorKind::UndefinedVariable);
    }

    #[test]
    fn test_semantic_analyzer_if_statement() {
        use crate::ast::{Statement, Expression, Literal, Block};
        
        let mut analyzer = SemanticAnalyzer::new();
        let stmt = Statement::If {
            condition: Expression::Literal(Literal::Bool(true)),
            then_block: Block::empty(),
            else_block: None,
        };
        
        analyzer.analyze_statement(&stmt);
        
        // Should have no errors for valid if statement
        assert_eq!(analyzer.errors().len(), 0);
    }

    #[test]
    fn test_semantic_analyzer_while_statement() {
        use crate::ast::{Statement, Expression, Literal, Block};
        
        let mut analyzer = SemanticAnalyzer::new();
        let stmt = Statement::While {
            label: None,
            condition: Expression::Literal(Literal::Bool(true)),
            body: Block::empty(),
        };
        
        analyzer.analyze_statement(&stmt);
        
        // Should have no errors for valid while statement
        assert_eq!(analyzer.errors().len(), 0);
    }

    #[test]
    fn test_semantic_analyzer_binary_expression() {
        use crate::ast::{Expression, BinaryOp, Literal};
        
        let mut analyzer = SemanticAnalyzer::new();
        let expr = Expression::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expression::Literal(Literal::Int(1))),
            right: Box::new(Expression::Literal(Literal::Int(2))),
        };
        
        let result_type = analyzer.analyze_expression(&expr);
        assert!(matches!(result_type, Type::Primitive(PrimitiveType::I32)));
        assert_eq!(analyzer.errors().len(), 0);
    }

    #[test]
    fn test_semantic_analyzer_comparison_expression() {
        use crate::ast::{Expression, BinaryOp, Literal};
        
        let mut analyzer = SemanticAnalyzer::new();
        let expr = Expression::Binary {
            op: BinaryOp::Lt,
            left: Box::new(Expression::Literal(Literal::Int(1))),
            right: Box::new(Expression::Literal(Literal::Int(2))),
        };
        
        let result_type = analyzer.analyze_expression(&expr);
        assert!(matches!(result_type, Type::Primitive(PrimitiveType::Bool)));
        assert_eq!(analyzer.errors().len(), 0);
    }

    #[test]
    fn test_semantic_analyzer_array_literal() {
        use crate::ast::{Expression, Literal};
        
        let mut analyzer = SemanticAnalyzer::new();
        let expr = Expression::ArrayLit {
            elements: vec![
                Expression::Literal(Literal::Int(1)),
                Expression::Literal(Literal::Int(2)),
                Expression::Literal(Literal::Int(3)),
            ],
        };
        
        let result_type = analyzer.analyze_expression(&expr);
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
        use crate::ast::{Expression, Literal};
        
        let mut analyzer = SemanticAnalyzer::new();
        let expr = Expression::TupleLit {
            elements: vec![
                Expression::Literal(Literal::Int(42)),
                Expression::Literal(Literal::Bool(true)),
            ],
        };
        
        let result_type = analyzer.analyze_expression(&expr);
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
        use crate::ast::{Expression, BinaryOp, Literal};
        
        let mut analyzer = SemanticAnalyzer::new();
        let expr = Expression::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expression::Literal(Literal::Int(1))),
            right: Box::new(Expression::Literal(Literal::Bool(true))),
        };
        
        analyzer.analyze_expression(&expr);
        
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
        assert_eq!(analyzer.errors()[0].kind, SemanticErrorKind::UnsupportedFeature);
        assert!(analyzer.errors()[0].message.contains("unions are not supported"));
        assert!(analyzer.errors()[0].message.contains("MyUnion"));
    }

    #[test]
    fn test_semantic_analyzer_unsupported_goto() {
        let mut analyzer = SemanticAnalyzer::new();
        
        analyzer.check_goto_usage("my_label");
        
        // Should detect unsupported goto
        assert_eq!(analyzer.errors().len(), 1);
        assert_eq!(analyzer.errors()[0].kind, SemanticErrorKind::UnsupportedFeature);
        assert!(analyzer.errors()[0].message.contains("goto statements are not supported"));
        assert!(analyzer.errors()[0].message.contains("my_label"));
    }

    #[test]
    fn test_semantic_analyzer_unsupported_include() {
        let mut analyzer = SemanticAnalyzer::new();
        
        analyzer.check_include_usage("stdio.h");
        
        // Should detect unsupported #include
        assert_eq!(analyzer.errors().len(), 1);
        assert_eq!(analyzer.errors()[0].kind, SemanticErrorKind::UnsupportedFeature);
        assert!(analyzer.errors()[0].message.contains("#include directives are not supported"));
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
        assert!(analyzer.errors().iter().all(|e| e.kind == SemanticErrorKind::UnsupportedFeature));
    }

    // Property-based tests
    
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;
        use crate::ast::{Expression, Literal, BinaryOp, PrimitiveType};

        // Property 28: Type checking matches Rust semantics
        // Validates: Requirements 18.9
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
                
                let result_type = analyzer.analyze_expression(&expr);
                
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
                
                let result_type = analyzer.analyze_expression(&expr);
                
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
                
                let result_type = analyzer.analyze_expression(&expr);
                
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
                
                let result_type = analyzer.analyze_expression(&expr);
                
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
}
