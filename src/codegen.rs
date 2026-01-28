// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Code generation module for emitting Rust or Crusty source code.

use crate::ast::*;

/// Target language for code generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetLanguage {
    Rust,
    Crusty,
}

/// Code generator that produces source code from AST
pub struct CodeGenerator {
    target: TargetLanguage,
    indent_level: usize,
    output: String,
}

impl CodeGenerator {
    /// Create a new code generator for the specified target language
    pub fn new(target: TargetLanguage) -> Self {
        Self {
            target,
            indent_level: 0,
            output: String::new(),
        }
    }

    /// Generate source code from a File AST
    pub fn generate(&mut self, file: &File) -> String {
        self.output.clear();
        self.indent_level = 0;

        // Generate doc comments for the file
        for comment in &file.doc_comments {
            self.write_line(&format!("//! {}", comment));
        }

        if !file.doc_comments.is_empty() && !file.items.is_empty() {
            self.write_line("");
        }

        // Generate all items
        for (i, item) in file.items.iter().enumerate() {
            if i > 0 {
                self.write_line("");
            }
            self.generate_item(item);
        }

        self.output.clone()
    }

    /// Write a line with current indentation
    fn write_line(&mut self, text: &str) {
        if !text.is_empty() {
            self.write_indent();
            self.output.push_str(text);
        }
        self.output.push('\n');
    }

    /// Write text without newline
    fn write(&mut self, text: &str) {
        self.output.push_str(text);
    }

    /// Write current indentation
    fn write_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }

    /// Increase indentation level
    fn indent(&mut self) {
        self.indent_level += 1;
    }

    /// Decrease indentation level
    fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    /// Generate code for a top-level item
    fn generate_item(&mut self, item: &Item) {
        match item {
            Item::Function(func) => self.generate_function(func),
            Item::Struct(struct_def) => self.generate_struct(struct_def),
            Item::Enum(enum_def) => self.generate_enum(enum_def),
            Item::Typedef(typedef) => self.generate_typedef(typedef),
            Item::Namespace(namespace) => self.generate_namespace(namespace),
            Item::Use(use_item) => self.generate_use(use_item),
            Item::Extern(extern_block) => self.generate_extern(extern_block),
            Item::Const(const_item) => self.generate_const(const_item),
            Item::Static(static_item) => self.generate_static(static_item),
            Item::MacroDefinition(macro_def) => self.generate_macro_definition(macro_def),
        }
    }

    fn generate_function(&mut self, func: &Function) {
        // Generate doc comments
        for comment in &func.doc_comments {
            self.write_line(&format!("/// {}", comment));
        }

        // Generate function signature
        self.write_indent();
        
        match self.target {
            TargetLanguage::Rust => {
                // Rust syntax: pub fn name(params) -> return_type { }
                match func.visibility {
                    Visibility::Public => self.write("pub "),
                    Visibility::Private => {}, // No keyword for private
                }

                self.write("fn ");
                self.write(&func.name.name);
                self.write("(");

                // Parameters
                for (i, param) in func.params.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write(&param.name.name);
                    self.write(": ");
                    self.write(&self.generate_type_string(&param.ty));
                }

                self.write(")");

                // Return type (void becomes no annotation)
                if let Some(ref return_type) = func.return_type {
                    if !matches!(return_type, Type::Primitive(PrimitiveType::Void)) {
                        self.write(" -> ");
                        self.write(&self.generate_type_string(return_type));
                    }
                }
            }
            TargetLanguage::Crusty => {
                // Crusty syntax: static? return_type name(params) { }
                if matches!(func.visibility, Visibility::Private) {
                    self.write("static ");
                }

                // Return type comes first in Crusty (C-style)
                if let Some(ref return_type) = func.return_type {
                    self.write(&self.generate_type_string(return_type));
                } else {
                    self.write("void");
                }
                self.write(" ");
                self.write(&func.name.name);
                self.write("(");

                // Parameters
                for (i, param) in func.params.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write(&self.generate_type_string(&param.ty));
                    self.write(" ");
                    self.write(&param.name.name);
                }

                self.write(")");
            }
        }

        self.write(" ");
        self.generate_block(&func.body);
        self.write("\n");
    }

    fn generate_struct(&mut self, struct_def: &Struct) {
        // Generate doc comments
        for comment in &struct_def.doc_comments {
            self.write_line(&format!("/// {}", comment));
        }

        // Generate struct definition
        self.write_indent();
        match struct_def.visibility {
            Visibility::Public => self.write("pub "),
            Visibility::Private => {},
        }
        self.write("struct ");
        self.write(&struct_def.name.name);
        self.write(" {\n");
        self.indent();

        // Generate fields
        for field in &struct_def.fields {
            for comment in &field.doc_comments {
                self.write_line(&format!("/// {}", comment));
            }
            self.write_indent();
            match field.visibility {
                Visibility::Public => self.write("pub "),
                Visibility::Private => {},
            }
            self.write(&field.name.name);
            self.write(": ");
            self.write(&self.generate_type_string(&field.ty));
            self.write(",\n");
        }

        self.dedent();
        self.write_line("}");

        // Generate impl block for methods if any
        if !struct_def.methods.is_empty() {
            self.write_line("");
            self.write_indent();
            self.write("impl ");
            self.write(&struct_def.name.name);
            self.write(" {\n");
            self.indent();

            for (i, method) in struct_def.methods.iter().enumerate() {
                if i > 0 {
                    self.write_line("");
                }
                self.generate_function(method);
            }

            self.dedent();
            self.write_line("}");
        }
    }

    fn generate_enum(&mut self, enum_def: &Enum) {
        // Generate doc comments
        for comment in &enum_def.doc_comments {
            self.write_line(&format!("/// {}", comment));
        }

        // Generate enum definition
        self.write_indent();
        match enum_def.visibility {
            Visibility::Public => self.write("pub "),
            Visibility::Private => {},
        }
        self.write("enum ");
        self.write(&enum_def.name.name);
        self.write(" {\n");
        self.indent();

        // Generate variants with discriminants
        for variant in &enum_def.variants {
            self.write_indent();
            self.write(&variant.name.name);
            if let Some(value) = variant.value {
                self.write(&format!(" = {}", value));
            }
            self.write(",\n");
        }

        self.dedent();
        self.write_line("}");
    }

    fn generate_typedef(&mut self, _typedef: &Typedef) {
        // Placeholder
        self.write_line("// TODO: generate_typedef");
    }

    fn generate_namespace(&mut self, _namespace: &Namespace) {
        // Placeholder
        self.write_line("// TODO: generate_namespace");
    }

    fn generate_use(&mut self, _use_item: &Use) {
        // Placeholder
        self.write_line("// TODO: generate_use");
    }

    fn generate_extern(&mut self, _extern_block: &Extern) {
        // Placeholder
        self.write_line("// TODO: generate_extern");
    }

    fn generate_const(&mut self, _const_item: &Const) {
        // Placeholder
        self.write_line("// TODO: generate_const");
    }

    fn generate_static(&mut self, _static_item: &Static) {
        // Placeholder
        self.write_line("// TODO: generate_static");
    }

    fn generate_macro_definition(&mut self, _macro_def: &MacroDefinition) {
        // Placeholder
        self.write_line("// TODO: generate_macro_definition");
    }

    /// Generate a block of statements
    fn generate_block(&mut self, block: &Block) {
        self.write("{\n");
        self.indent();
        
        for stmt in &block.statements {
            self.generate_statement(stmt);
        }
        
        self.dedent();
        self.write_indent();
        self.write("}");
    }

    /// Generate a statement
    fn generate_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Let { name, ty, init, mutable } => {
                self.write_indent();
                match self.target {
                    TargetLanguage::Rust => {
                        self.write("let ");
                        if *mutable {
                            self.write("mut ");
                        }
                        self.write(&name.name);
                        if let Some(ref ty) = ty {
                            self.write(": ");
                            self.write(&self.generate_type_string(ty));
                        }
                        if let Some(ref init) = init {
                            self.write(" = ");
                            self.write(&self.generate_expression_string(init));
                        }
                        self.write(";\n");
                    }
                    TargetLanguage::Crusty => {
                        // Crusty uses Rust-style let syntax (not C-style)
                        self.write("let ");
                        self.write(&name.name);
                        if let Some(ref ty) = ty {
                            self.write(": ");
                            self.write(&self.generate_type_string(ty));
                        }
                        if let Some(ref init) = init {
                            self.write(" = ");
                            self.write(&self.generate_expression_string(init));
                        }
                        self.write(";\n");
                    }
                }
            }
            Statement::Var { name, ty, init } => {
                self.write_indent();
                match self.target {
                    TargetLanguage::Rust => {
                        // var is translated to let mut
                        self.write("let mut ");
                        self.write(&name.name);
                        if let Some(ref ty) = ty {
                            self.write(": ");
                            self.write(&self.generate_type_string(ty));
                        }
                        if let Some(ref init) = init {
                            self.write(" = ");
                            self.write(&self.generate_expression_string(init));
                        }
                        self.write(";\n");
                    }
                    TargetLanguage::Crusty => {
                        // Crusty uses Rust-style var syntax
                        self.write("var ");
                        self.write(&name.name);
                        if let Some(ref ty) = ty {
                            self.write(": ");
                            self.write(&self.generate_type_string(ty));
                        }
                        if let Some(ref init) = init {
                            self.write(" = ");
                            self.write(&self.generate_expression_string(init));
                        }
                        self.write(";\n");
                    }
                }
            }
            Statement::Const { name, ty, value } => {
                self.write_indent();
                match self.target {
                    TargetLanguage::Rust => {
                        self.write("const ");
                        self.write(&name.name);
                        self.write(": ");
                        self.write(&self.generate_type_string(ty));
                        self.write(" = ");
                        self.write(&self.generate_expression_string(value));
                        self.write(";\n");
                    }
                    TargetLanguage::Crusty => {
                        // Crusty uses Rust-style const syntax
                        self.write("const ");
                        self.write(&name.name);
                        self.write(": ");
                        self.write(&self.generate_type_string(ty));
                        self.write(" = ");
                        self.write(&self.generate_expression_string(value));
                        self.write(";\n");
                    }
                }
            }
            Statement::Expr(expr) => {
                self.write_indent();
                self.write(&self.generate_expression_string(expr));
                self.write(";\n");
            }
            Statement::Return(expr) => {
                self.write_indent();
                self.write("return");
                if let Some(ref expr) = expr {
                    self.write(" ");
                    self.write(&self.generate_expression_string(expr));
                }
                self.write(";\n");
            }
            Statement::If { condition, then_block, else_block } => {
                self.write_indent();
                self.write("if ");
                match self.target {
                    TargetLanguage::Rust => {
                        self.write(&self.generate_expression_string(condition));
                    }
                    TargetLanguage::Crusty => {
                        self.write("(");
                        self.write(&self.generate_expression_string(condition));
                        self.write(")");
                    }
                }
                self.write(" ");
                self.generate_block(then_block);
                if let Some(ref else_block) = else_block {
                    self.write(" else ");
                    self.generate_block(else_block);
                }
                self.write("\n");
            }
            Statement::While { label, condition, body } => {
                self.write_indent();
                if let Some(ref label) = label {
                    match self.target {
                        TargetLanguage::Rust => {
                            // Translate .label: to 'label:
                            self.write("'");
                            self.write(&label.name);
                            self.write(": ");
                        }
                        TargetLanguage::Crusty => {
                            self.write(".");
                            self.write(&label.name);
                            self.write(": ");
                        }
                    }
                }
                self.write("while ");
                match self.target {
                    TargetLanguage::Rust => {
                        self.write(&self.generate_expression_string(condition));
                    }
                    TargetLanguage::Crusty => {
                        self.write("(");
                        self.write(&self.generate_expression_string(condition));
                        self.write(")");
                    }
                }
                self.write(" ");
                self.generate_block(body);
                self.write("\n");
            }
            Statement::For { label, init, condition, increment, body } => {
                // C-style for loop translates to Rust loop with break
                self.write_indent();
                if let Some(ref label) = label {
                    match self.target {
                        TargetLanguage::Rust => {
                            self.write("'");
                            self.write(&label.name);
                            self.write(": ");
                        }
                        TargetLanguage::Crusty => {
                            self.write(".");
                            self.write(&label.name);
                            self.write(": ");
                        }
                    }
                }
                
                match self.target {
                    TargetLanguage::Rust => {
                        self.write("{\n");
                        self.indent();
                        
                        // Init statement
                        self.generate_statement(init);
                        
                        // Loop
                        self.write_indent();
                        self.write("loop {\n");
                        self.indent();
                        
                        // Condition check
                        self.write_indent();
                        self.write("if !(");
                        self.write(&self.generate_expression_string(condition));
                        self.write(") { break; }\n");
                        
                        // Body
                        for stmt in &body.statements {
                            self.generate_statement(stmt);
                        }
                        
                        // Increment
                        self.write_indent();
                        self.write(&self.generate_expression_string(increment));
                        self.write(";\n");
                        
                        self.dedent();
                        self.write_indent();
                        self.write("}\n");
                        
                        self.dedent();
                        self.write_indent();
                        self.write("}\n");
                    }
                    TargetLanguage::Crusty => {
                        self.write("for (");
                        // Generate init inline (without newline)
                        match init.as_ref() {
                            Statement::Let { name, ty, init, .. } => {
                                self.write("let ");
                                if let Some(ref ty) = ty {
                                    self.write(&self.generate_type_string(ty));
                                    self.write(" ");
                                }
                                self.write(&name.name);
                                if let Some(ref init) = init {
                                    self.write(" = ");
                                    self.write(&self.generate_expression_string(init));
                                }
                            }
                            Statement::Var { name, ty, init } => {
                                self.write("var ");
                                if let Some(ref ty) = ty {
                                    self.write(&self.generate_type_string(ty));
                                    self.write(" ");
                                }
                                self.write(&name.name);
                                if let Some(ref init) = init {
                                    self.write(" = ");
                                    self.write(&self.generate_expression_string(init));
                                }
                            }
                            _ => {}
                        }
                        self.write("; ");
                        self.write(&self.generate_expression_string(condition));
                        self.write("; ");
                        self.write(&self.generate_expression_string(increment));
                        self.write(") ");
                        self.generate_block(body);
                        self.write("\n");
                    }
                }
            }
            Statement::ForIn { label, var, iter, body } => {
                self.write_indent();
                if let Some(ref label) = label {
                    match self.target {
                        TargetLanguage::Rust => {
                            self.write("'");
                            self.write(&label.name);
                            self.write(": ");
                        }
                        TargetLanguage::Crusty => {
                            self.write(".");
                            self.write(&label.name);
                            self.write(": ");
                        }
                    }
                }
                self.write("for ");
                self.write(&var.name);
                self.write(" in ");
                self.write(&self.generate_expression_string(iter));
                self.write(" ");
                self.generate_block(body);
                self.write("\n");
            }
            Statement::Switch { expr, cases, default } => {
                self.write_indent();
                match self.target {
                    TargetLanguage::Rust => {
                        self.write("match ");
                        self.write(&self.generate_expression_string(expr));
                        self.write(" {\n");
                        self.indent();
                        
                        for case in cases {
                            self.write_indent();
                            for (i, value) in case.values.iter().enumerate() {
                                if i > 0 {
                                    self.write(" | ");
                                }
                                self.write(&self.generate_expression_string(value));
                            }
                            self.write(" => ");
                            self.generate_block(&case.body);
                            self.write(",\n");
                        }
                        
                        if let Some(ref default) = default {
                            self.write_indent();
                            self.write("_ => ");
                            self.generate_block(default);
                            self.write(",\n");
                        }
                        
                        self.dedent();
                        self.write_indent();
                        self.write("}\n");
                    }
                    TargetLanguage::Crusty => {
                        self.write("switch (");
                        self.write(&self.generate_expression_string(expr));
                        self.write(") {\n");
                        self.indent();
                        
                        for case in cases {
                            self.write_indent();
                            self.write("case ");
                            for (i, value) in case.values.iter().enumerate() {
                                if i > 0 {
                                    self.write(", ");
                                }
                                self.write(&self.generate_expression_string(value));
                            }
                            self.write(": ");
                            self.generate_block(&case.body);
                            self.write("\n");
                        }
                        
                        if let Some(ref default) = default {
                            self.write_indent();
                            self.write("default: ");
                            self.generate_block(default);
                            self.write("\n");
                        }
                        
                        self.dedent();
                        self.write_indent();
                        self.write("}\n");
                    }
                }
            }
            Statement::Break(label) => {
                self.write_indent();
                self.write("break");
                if let Some(ref label) = label {
                    match self.target {
                        TargetLanguage::Rust => {
                            // Translate break .label to break 'label
                            self.write(" '");
                            self.write(&label.name);
                        }
                        TargetLanguage::Crusty => {
                            self.write(" .");
                            self.write(&label.name);
                        }
                    }
                }
                self.write(";\n");
            }
            Statement::Continue(label) => {
                self.write_indent();
                self.write("continue");
                if let Some(ref label) = label {
                    match self.target {
                        TargetLanguage::Rust => {
                            // Translate continue .label to continue 'label
                            self.write(" '");
                            self.write(&label.name);
                        }
                        TargetLanguage::Crusty => {
                            self.write(" .");
                            self.write(&label.name);
                        }
                    }
                }
                self.write(";\n");
            }
        }
    }

    /// Generate an expression and return as string
    pub fn generate_expression_string(&self, expr: &Expression) -> String {
        match expr {
            Expression::Literal(lit) => self.generate_literal_string(lit),
            Expression::Ident(ident) => ident.name.clone(),
            Expression::Binary { op, left, right } => {
                format!(
                    "({} {} {})",
                    self.generate_expression_string(left),
                    self.generate_binary_op_string(op),
                    self.generate_expression_string(right)
                )
            }
            Expression::Unary { op, expr } => {
                self.generate_unary_expression_string(op, expr)
            }
            Expression::Call { func, args } => {
                let mut result = self.generate_expression_string(func);
                result.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&self.generate_expression_string(arg));
                }
                result.push(')');
                result
            }
            Expression::FieldAccess { expr, field } => {
                format!(
                    "{}.{}",
                    self.generate_expression_string(expr),
                    field.name
                )
            }
            Expression::Index { expr, index } => {
                format!(
                    "{}[{}]",
                    self.generate_expression_string(expr),
                    self.generate_expression_string(index)
                )
            }
            Expression::Cast { expr, ty } => {
                format!(
                    "({} as {})",
                    self.generate_expression_string(expr),
                    self.generate_type_string(ty)
                )
            }
            Expression::Sizeof { ty } => {
                format!("std::mem::size_of::<{}>()", self.generate_type_string(ty))
            }
            Expression::Ternary { condition, then_expr, else_expr } => {
                format!(
                    "if {} {{ {} }} else {{ {} }}",
                    self.generate_expression_string(condition),
                    self.generate_expression_string(then_expr),
                    self.generate_expression_string(else_expr)
                )
            }
            Expression::StructInit { ty, fields } => {
                let mut result = self.generate_type_string(ty);
                result.push_str(" { ");
                for (i, (name, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&name.name);
                    result.push_str(": ");
                    result.push_str(&self.generate_expression_string(value));
                }
                result.push_str(" }");
                result
            }
            Expression::ArrayLit { elements } => {
                let mut result = String::from("[");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&self.generate_expression_string(elem));
                }
                result.push(']');
                result
            }
            Expression::TupleLit { elements } => {
                let mut result = String::from("(");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&self.generate_expression_string(elem));
                }
                result.push(')');
                result
            }
            Expression::Range { start, end, inclusive } => {
                let mut result = String::new();
                if let Some(ref start) = start {
                    result.push_str(&self.generate_expression_string(start));
                }
                result.push_str("..");
                if *inclusive {
                    result.push('=');
                }
                if let Some(ref end) = end {
                    result.push_str(&self.generate_expression_string(end));
                }
                result
            }
            Expression::MacroCall { name, args } => {
                let mut result = name.name.clone();
                result.push('!');
                result.push('(');
                for (i, token) in args.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&token.text);
                }
                result.push(')');
                result
            }
            Expression::RustBlock { tokens } => {
                let mut result = String::from("{ ");
                for token in tokens {
                    result.push_str(&token.text);
                    result.push(' ');
                }
                result.push('}');
                result
            }
            Expression::ErrorProp { expr } => {
                format!("{}?", self.generate_expression_string(expr))
            }
            Expression::MethodCall { receiver, method, args } => {
                let mut result = self.generate_expression_string(receiver);
                result.push('.');
                result.push_str(&method.name);
                result.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&self.generate_expression_string(arg));
                }
                result.push(')');
                result
            }
            Expression::TypeScopedCall { ty, method, args } => {
                // Translate @Type.method() to Type::method()
                let mut result = self.generate_type_string(ty);
                result.push_str("::");
                result.push_str(&method.name);
                result.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&self.generate_expression_string(arg));
                }
                result.push(')');
                result
            }
            Expression::ExplicitGenericCall { ty, generics, method, args } => {
                // Translate @Type(T).method() to Type::<T>::method()
                let mut result = self.generate_type_string(ty);
                result.push_str("::<");
                for (i, gen) in generics.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&self.generate_type_string(gen));
                }
                result.push_str(">::");
                result.push_str(&method.name);
                result.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&self.generate_expression_string(arg));
                }
                result.push(')');
                result
            }
        }
    }

    /// Generate a literal value as string
    fn generate_literal_string(&self, lit: &Literal) -> String {
        match lit {
            Literal::Int(n) => n.to_string(),
            Literal::Float(f) => f.to_string(),
            Literal::String(s) => format!("\"{}\"", s.escape_default()),
            Literal::Char(c) => format!("'{}'", c.escape_default()),
            Literal::Bool(b) => b.to_string(),
        }
    }

    /// Generate a binary operator as string
    fn generate_binary_op_string(&self, op: &BinaryOp) -> &'static str {
        match op {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Eq => "==",
            BinaryOp::Ne => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Gt => ">",
            BinaryOp::Le => "<=",
            BinaryOp::Ge => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
            BinaryOp::BitAnd => "&",
            BinaryOp::BitOr => "|",
            BinaryOp::BitXor => "^",
            BinaryOp::Shl => "<<",
            BinaryOp::Shr => ">>",
            BinaryOp::Assign => "=",
            BinaryOp::AddAssign => "+=",
            BinaryOp::SubAssign => "-=",
            BinaryOp::MulAssign => "*=",
            BinaryOp::DivAssign => "/=",
        }
    }

    /// Generate a unary expression as string
    fn generate_unary_expression_string(&self, op: &UnaryOp, expr: &Expression) -> String {
        match op {
            UnaryOp::Not => format!("!({})", self.generate_expression_string(expr)),
            UnaryOp::Neg => format!("-({})", self.generate_expression_string(expr)),
            UnaryOp::Ref => format!("&({})", self.generate_expression_string(expr)),
            UnaryOp::Deref => format!("*({})", self.generate_expression_string(expr)),
            UnaryOp::PreInc => {
                // ++x translates to { x += 1; x }
                format!("{{ let __tmp = &mut ({}); *__tmp += 1; *__tmp }}", 
                    self.generate_expression_string(expr))
            }
            UnaryOp::PreDec => {
                // --x translates to { x -= 1; x }
                format!("{{ let __tmp = &mut ({}); *__tmp -= 1; *__tmp }}", 
                    self.generate_expression_string(expr))
            }
            UnaryOp::PostInc => {
                // x++ translates to { let tmp = x; x += 1; tmp }
                format!("{{ let __old = ({}); let __tmp = &mut ({}); *__tmp += 1; __old }}", 
                    self.generate_expression_string(expr),
                    self.generate_expression_string(expr))
            }
            UnaryOp::PostDec => {
                // x-- translates to { let tmp = x; x -= 1; tmp }
                format!("{{ let __old = ({}); let __tmp = &mut ({}); *__tmp -= 1; __old }}", 
                    self.generate_expression_string(expr),
                    self.generate_expression_string(expr))
            }
        }
    }

    /// Generate a type as string
    pub fn generate_type_string(&self, ty: &Type) -> String {
        match ty {
            Type::Primitive(prim) => self.generate_primitive_type_string(prim),
            Type::Ident(ident) => ident.name.clone(),
            Type::Pointer { ty, mutable } => {
                // Translate pointers to raw pointers
                if *mutable {
                    format!("*mut {}", self.generate_type_string(ty))
                } else {
                    format!("*const {}", self.generate_type_string(ty))
                }
            }
            Type::Reference { ty, mutable } => {
                // & and &var/&mut translate to Rust references
                if *mutable {
                    format!("&mut {}", self.generate_type_string(ty))
                } else {
                    format!("&{}", self.generate_type_string(ty))
                }
            }
            Type::Array { ty, size } => {
                if let Some(size) = size {
                    format!("[{}; {}]", self.generate_type_string(ty), size)
                } else {
                    format!("[{}]", self.generate_type_string(ty))
                }
            }
            Type::Slice { ty } => {
                format!("[{}]", self.generate_type_string(ty))
            }
            Type::Tuple { types } => {
                let mut result = String::from("(");
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&self.generate_type_string(ty));
                }
                result.push(')');
                result
            }
            Type::Generic { base, args } => {
                let mut result = self.generate_type_string(base);
                result.push('<');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&self.generate_type_string(arg));
                }
                result.push('>');
                result
            }
            Type::Function { params, return_type } => {
                let mut result = String::from("fn(");
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&self.generate_type_string(param));
                }
                result.push_str(") -> ");
                result.push_str(&self.generate_type_string(return_type));
                result
            }
            Type::Fallible { ty } => {
                format!("Result<{}, Box<dyn std::error::Error>>", self.generate_type_string(ty))
            }
            Type::Auto => String::from("_"),
        }
    }

    /// Generate a primitive type as string
    fn generate_primitive_type_string(&self, prim: &PrimitiveType) -> String {
        match self.target {
            TargetLanguage::Rust => match prim {
                PrimitiveType::Int => "i32".to_string(),
                PrimitiveType::I32 => "i32".to_string(),
                PrimitiveType::I64 => "i64".to_string(),
                PrimitiveType::U32 => "u32".to_string(),
                PrimitiveType::U64 => "u64".to_string(),
                PrimitiveType::Float => "f64".to_string(),
                PrimitiveType::F32 => "f32".to_string(),
                PrimitiveType::F64 => "f64".to_string(),
                PrimitiveType::Bool => "bool".to_string(),
                PrimitiveType::Char => "char".to_string(),
                PrimitiveType::Void => "()".to_string(),
            },
            TargetLanguage::Crusty => match prim {
                PrimitiveType::Int => "int".to_string(),
                PrimitiveType::I32 => "i32".to_string(),
                PrimitiveType::I64 => "i64".to_string(),
                PrimitiveType::U32 => "u32".to_string(),
                PrimitiveType::U64 => "u64".to_string(),
                PrimitiveType::Float => "float".to_string(),
                PrimitiveType::F32 => "f32".to_string(),
                PrimitiveType::F64 => "f64".to_string(),
                PrimitiveType::Bool => "bool".to_string(),
                PrimitiveType::Char => "char".to_string(),
                PrimitiveType::Void => "void".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_code_generator() {
        let gen = CodeGenerator::new(TargetLanguage::Rust);
        assert_eq!(gen.target, TargetLanguage::Rust);
        assert_eq!(gen.indent_level, 0);
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
        assert_eq!(gen.indent_level, 0);
        
        gen.indent();
        assert_eq!(gen.indent_level, 1);
        
        gen.indent();
        assert_eq!(gen.indent_level, 2);
        
        gen.dedent();
        assert_eq!(gen.indent_level, 1);
        
        gen.dedent();
        assert_eq!(gen.indent_level, 0);
        
        // Should not go below 0
        gen.dedent();
        assert_eq!(gen.indent_level, 0);
    }

    #[test]
    fn test_generate_simple_function() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let func = Function {
            visibility: Visibility::Public,
            name: Ident::new("main"),
            params: vec![],
            return_type: None,
            body: Block::empty(),
            doc_comments: vec![],
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
            body: Block::empty(),
            doc_comments: vec![],
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
            body: Block::empty(),
            doc_comments: vec![],
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
            body: Block::empty(),
            doc_comments: vec![],
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
            body: Block::new(vec![stmt]),
            doc_comments: vec![],
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
            body: Block::new(vec![stmt]),
            doc_comments: vec![],
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
            body: Block::new(vec![stmt]),
            doc_comments: vec![],
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
            body: Block::new(vec![stmt]),
            doc_comments: vec![],
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
            body: Block::new(vec![stmt]),
            doc_comments: vec![],
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
            body: Block::new(vec![stmt]),
            doc_comments: vec![],
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
            body: Block::new(vec![stmt]),
            doc_comments: vec![],
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
                },
                Field {
                    visibility: Visibility::Public,
                    name: Ident::new("y"),
                    ty: Type::Primitive(PrimitiveType::I32),
                    doc_comments: vec![],
                },
            ],
            methods: vec![],
            doc_comments: vec![],
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
            body: Block::empty(),
            doc_comments: vec![],
        };
        let struct_def = Struct {
            visibility: Visibility::Public,
            name: Ident::new("Point"),
            fields: vec![],
            methods: vec![method],
            doc_comments: vec![],
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
        assert_eq!(gen.generate_type_string(&Type::Primitive(PrimitiveType::Int)), "i32");
        assert_eq!(gen.generate_type_string(&Type::Primitive(PrimitiveType::I32)), "i32");
        assert_eq!(gen.generate_type_string(&Type::Primitive(PrimitiveType::I64)), "i64");
        assert_eq!(gen.generate_type_string(&Type::Primitive(PrimitiveType::U32)), "u32");
        assert_eq!(gen.generate_type_string(&Type::Primitive(PrimitiveType::U64)), "u64");
        assert_eq!(gen.generate_type_string(&Type::Primitive(PrimitiveType::Float)), "f64");
        assert_eq!(gen.generate_type_string(&Type::Primitive(PrimitiveType::F32)), "f32");
        assert_eq!(gen.generate_type_string(&Type::Primitive(PrimitiveType::F64)), "f64");
        assert_eq!(gen.generate_type_string(&Type::Primitive(PrimitiveType::Bool)), "bool");
        assert_eq!(gen.generate_type_string(&Type::Primitive(PrimitiveType::Char)), "char");
        assert_eq!(gen.generate_type_string(&Type::Primitive(PrimitiveType::Void)), "()");
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
}

