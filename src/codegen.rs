// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Code generation module for emitting Rust or Crusty source code.

use crate::ast::*;

/// Target language for code generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
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
            Item::Import(import_item) => self.generate_import(import_item),
            Item::Export(export_item) => self.generate_export(export_item),
            Item::Extern(extern_block) => self.generate_extern(extern_block),
            Item::Const(const_item) => self.generate_const(const_item),
            Item::Static(static_item) => self.generate_static(static_item),
            Item::MacroDefinition(macro_def) => self.generate_macro_definition(macro_def),
        }
    }

    fn generate_function(&mut self, func: &Function) {
        // Generate attributes
        for attr in &func.attributes {
            self.write_indent();
            self.write("#[");
            self.write(&attr.name.name);

            // Generate attribute arguments if present
            if !attr.args.is_empty() {
                self.write("(");
                for (i, arg) in attr.args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    match arg {
                        AttributeArg::Ident(ident) => {
                            self.write(&ident.name);
                        }
                        AttributeArg::Literal(lit) => {
                            self.write(&self.generate_literal_string(lit));
                        }
                        AttributeArg::NameValue { name, value } => {
                            self.write(&name.name);
                            self.write(" = ");
                            self.write(&self.generate_literal_string(value));
                        }
                    }
                }
                self.write(")");
            }

            self.write("]\n");
        }

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
                    Visibility::Private => {} // No keyword for private
                }

                self.write("fn ");
                self.write(&func.name.name);
                self.write("(");

                // Parameters
                for (i, param) in func.params.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }

                    // Special handling for self parameters to use idiomatic Rust syntax
                    if param.name.name == "self" {
                        match &param.ty {
                            Type::Reference { ty: _, mutable } => {
                                // &self or &mut self
                                if *mutable {
                                    self.write("&mut self");
                                } else {
                                    self.write("&self");
                                }
                            }
                            Type::Ident(ident) if ident.name == "Self" => {
                                // self (by value)
                                self.write("self");
                            }
                            _ => {
                                // Fallback to regular parameter syntax
                                self.write(&param.name.name);
                                self.write(": ");
                                self.write(&self.generate_type_string(&param.ty));
                            }
                        }
                    } else {
                        self.write(&param.name.name);
                        self.write(": ");
                        self.write(&self.generate_type_string(&param.ty));
                    }
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
        // Generate attributes
        for attr in &struct_def.attributes {
            self.write_indent();
            self.write("#[");
            self.write(&attr.name.name);

            if !attr.args.is_empty() {
                self.write("(");
                for (i, arg) in attr.args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    match arg {
                        AttributeArg::Ident(ident) => {
                            self.write(&ident.name);
                        }
                        AttributeArg::Literal(lit) => {
                            self.write(&self.generate_literal_string(lit));
                        }
                        AttributeArg::NameValue { name, value } => {
                            self.write(&name.name);
                            self.write(" = ");
                            self.write(&self.generate_literal_string(value));
                        }
                    }
                }
                self.write(")");
            }

            self.write("]\n");
        }

        // Generate doc comments
        for comment in &struct_def.doc_comments {
            self.write_line(&format!("/// {}", comment));
        }

        // Generate struct definition
        self.write_indent();
        match struct_def.visibility {
            Visibility::Public => self.write("pub "),
            Visibility::Private => {}
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
                Visibility::Private => {}
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
        // Generate attributes
        for attr in &enum_def.attributes {
            self.write_indent();
            self.write("#[");
            self.write(&attr.name.name);

            if !attr.args.is_empty() {
                self.write("(");
                for (i, arg) in attr.args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    match arg {
                        AttributeArg::Ident(ident) => {
                            self.write(&ident.name);
                        }
                        AttributeArg::Literal(lit) => {
                            self.write(&self.generate_literal_string(lit));
                        }
                        AttributeArg::NameValue { name, value } => {
                            self.write(&name.name);
                            self.write(" = ");
                            self.write(&self.generate_literal_string(value));
                        }
                    }
                }
                self.write(")");
            }

            self.write("]\n");
        }

        // Generate doc comments
        for comment in &enum_def.doc_comments {
            self.write_line(&format!("/// {}", comment));
        }

        // Generate enum definition
        self.write_indent();
        match enum_def.visibility {
            Visibility::Public => self.write("pub "),
            Visibility::Private => {}
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

    fn generate_typedef(&mut self, typedef: &Typedef) {
        // Generate doc comments
        for comment in &typedef.doc_comments {
            self.write_line(&format!("///{}", comment));
        }

        // Generate visibility
        let visibility = match typedef.visibility {
            Visibility::Public => "pub ",
            Visibility::Private => "",
        };

        // Generate type alias
        // typedef int MyInt; → pub type MyInt = i32;
        let type_str = self.generate_type_string(&typedef.target);
        self.write_line(&format!(
            "{}type {} = {};",
            visibility, typedef.name.name, type_str
        ));
    }

    fn generate_namespace(&mut self, _namespace: &Namespace) {
        // Placeholder
        self.write_line("// TODO: generate_namespace");
    }

    fn generate_import(&mut self, import_item: &Import) {
        // #import module.path → use module::path;
        self.write("use ");
        for (i, ident) in import_item.path.iter().enumerate() {
            if i > 0 {
                self.write("::");
            }
            self.write(&ident.name);
        }
        if let Some(alias) = &import_item.alias {
            self.write(" as ");
            self.write(&alias.name);
        }
        self.write_line(";");
    }

    fn generate_export(&mut self, export_item: &Export) {
        // #export module.path → pub use module::path;
        self.write("pub use ");
        for (i, ident) in export_item.path.iter().enumerate() {
            if i > 0 {
                self.write("::");
            }
            self.write(&ident.name);
        }
        if let Some(alias) = &export_item.alias {
            self.write(" as ");
            self.write(&alias.name);
        }
        self.write_line(";");
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

    fn generate_macro_definition(&mut self, macro_def: &MacroDefinition) {
        // Translate #define to Rust macro_rules!
        // Remove double-underscore prefix and suffix from macro name
        let rust_name = macro_def
            .name
            .name
            .trim_start_matches("__")
            .trim_end_matches("__")
            .to_lowercase();

        // Check if the converted name is a Rust keyword
        // If so, add a suffix to avoid conflicts
        let rust_name = if is_rust_keyword(&rust_name) {
            format!("{}_macro", rust_name)
        } else {
            rust_name
        };

        self.write_line(&format!("macro_rules! {} {{", rust_name));
        self.indent();

        // Generate macro pattern and body
        self.write_indent();
        self.write("(");

        // Generate parameter pattern
        if !macro_def.params.is_empty() {
            for (i, param) in macro_def.params.iter().enumerate() {
                if i > 0 {
                    self.write(", ");
                }
                self.write(&format!("${}:expr", param.name));
            }
        }

        self.write(") => {{\n");
        self.indent();

        // Generate macro body
        self.write_indent();
        for token in &macro_def.body {
            // Check if token is a parameter reference
            let is_param = macro_def.params.iter().any(|p| {
                if let crate::lexer::TokenKind::Ident(ref name) = token.kind {
                    name == &p.name
                } else {
                    false
                }
            });

            if is_param {
                // Replace parameter with $param
                if let crate::lexer::TokenKind::Ident(ref name) = token.kind {
                    self.write(&format!("${}", name));
                }
            } else {
                // Check if it's a macro invocation with double-underscores
                if let crate::lexer::TokenKind::Ident(ref name) = token.kind {
                    if name.starts_with("__") && name.ends_with("__") {
                        // Convert __macro_name__ to macro_name!
                        let rust_macro = name
                            .trim_start_matches("__")
                            .trim_end_matches("__")
                            .to_lowercase();
                        self.write(&format!("{}!", rust_macro));
                    } else {
                        self.write(&token.text);
                    }
                } else {
                    self.write(&token.text);
                }
            }
            self.write(" ");
        }
        self.write("\n");

        self.dedent();
        self.write_indent();
        self.write("}};\n");

        self.dedent();
        self.write_line("}");
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
            Statement::Let {
                name,
                ty,
                init,
                mutable,
            } => {
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
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
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
            Statement::While {
                label,
                condition,
                body,
            } => {
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
            Statement::For {
                label,
                init,
                condition,
                increment,
                body,
            } => {
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
            Statement::ForIn {
                label,
                var,
                iter,
                body,
            } => {
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
            Statement::Switch {
                expr,
                cases,
                default,
            } => {
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

            Statement::NestedFunction {
                name,
                params,
                return_type,
                body,
            } => {
                // TODO: Implement full closure generation with capture analysis in subtask 17.4
                // For now, generate a basic closure
                self.write_indent();
                self.write("let ");
                self.write(&name.name);
                self.write(" = |");

                // Generate parameters
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write(&param.name.name);
                    self.write(": ");
                    let param_type = self.generate_type_string(&param.ty);
                    self.write(&param_type);
                }

                self.write("|");

                // Generate return type if present
                if let Some(ref ret_ty) = return_type {
                    self.write(" -> ");
                    let return_type_str = self.generate_type_string(ret_ty);
                    self.write(&return_type_str);
                }

                self.write(" ");
                self.generate_block(body);
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
            Expression::Unary { op, expr } => self.generate_unary_expression_string(op, expr),
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
                format!("{}.{}", self.generate_expression_string(expr), field.name)
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
            Expression::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
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
            Expression::Range {
                start,
                end,
                inclusive,
            } => {
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
            Expression::MethodCall {
                receiver,
                method,
                args,
            } => {
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
            Expression::ExplicitGenericCall {
                ty,
                generics,
                method,
                args,
            } => {
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
            Literal::Null => match self.target {
                TargetLanguage::Rust => "Option::None".to_string(),
                TargetLanguage::Crusty => "NULL".to_string(),
            },
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
                format!(
                    "{{ let __tmp = &mut ({}); *__tmp += 1; *__tmp }}",
                    self.generate_expression_string(expr)
                )
            }
            UnaryOp::PreDec => {
                // --x translates to { x -= 1; x }
                format!(
                    "{{ let __tmp = &mut ({}); *__tmp -= 1; *__tmp }}",
                    self.generate_expression_string(expr)
                )
            }
            UnaryOp::PostInc => {
                // x++ translates to { let tmp = x; x += 1; tmp }
                format!(
                    "{{ let __old = ({}); let __tmp = &mut ({}); *__tmp += 1; __old }}",
                    self.generate_expression_string(expr),
                    self.generate_expression_string(expr)
                )
            }
            UnaryOp::PostDec => {
                // x-- translates to { let tmp = x; x -= 1; tmp }
                format!(
                    "{{ let __old = ({}); let __tmp = &mut ({}); *__tmp -= 1; __old }}",
                    self.generate_expression_string(expr),
                    self.generate_expression_string(expr)
                )
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
            Type::Function {
                params,
                return_type,
            } => {
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
                format!(
                    "Result<{}, Box<dyn std::error::Error>>",
                    self.generate_type_string(ty)
                )
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

/// Check if a string is a Rust keyword
fn is_rust_keyword(s: &str) -> bool {
    matches!(
        s,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
            | "try"
    )
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
            body: Block::empty(),
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
            body: Block::empty(),
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
            body: Block::empty(),
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
            body: Block::new(vec![stmt]),
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
            body: Block::new(vec![stmt]),
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
            body: Block::new(vec![stmt]),
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
            body: Block::new(vec![stmt]),
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
            body: Block::new(vec![stmt]),
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
            body: Block::new(vec![stmt]),
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
            body: Block::new(vec![stmt]),
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
            body: Block::empty(),
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
            body: Block::empty(),
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
            body: Block::empty(),
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
            body: Block::empty(),
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
            body: vec![crate::lexer::Token::new(
                crate::lexer::TokenKind::IntLiteral("100".to_string()),
                crate::error::Span::new(
                    crate::error::Position::new(1, 1),
                    crate::error::Position::new(1, 4),
                ),
                "100".to_string(),
            )],
            delimiter: MacroDelimiter::None,
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
            body: vec![
                crate::lexer::Token::new(
                    crate::lexer::TokenKind::LParen,
                    crate::error::Span::new(
                        crate::error::Position::new(1, 1),
                        crate::error::Position::new(1, 2),
                    ),
                    "(".to_string(),
                ),
                crate::lexer::Token::new(
                    crate::lexer::TokenKind::Ident("a".to_string()),
                    crate::error::Span::new(
                        crate::error::Position::new(1, 2),
                        crate::error::Position::new(1, 3),
                    ),
                    "a".to_string(),
                ),
                crate::lexer::Token::new(
                    crate::lexer::TokenKind::Plus,
                    crate::error::Span::new(
                        crate::error::Position::new(1, 4),
                        crate::error::Position::new(1, 5),
                    ),
                    "+".to_string(),
                ),
                crate::lexer::Token::new(
                    crate::lexer::TokenKind::Ident("b".to_string()),
                    crate::error::Span::new(
                        crate::error::Position::new(1, 6),
                        crate::error::Position::new(1, 7),
                    ),
                    "b".to_string(),
                ),
                crate::lexer::Token::new(
                    crate::lexer::TokenKind::RParen,
                    crate::error::Span::new(
                        crate::error::Position::new(1, 7),
                        crate::error::Position::new(1, 8),
                    ),
                    ")".to_string(),
                ),
            ],
            delimiter: MacroDelimiter::Parens,
        };

        let file = File {
            items: vec![Item::MacroDefinition(macro_def)],
            doc_comments: vec![],
        };

        let output = gen.generate(&file);
        assert!(output.contains("macro_rules! add"));
        assert!(output.contains("$a:expr"));
        assert!(output.contains("$b:expr"));
        assert!(output.contains("$a"));
        assert!(output.contains("$b"));
    }

    #[test]
    fn test_generate_macro_with_nested_macro_call() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let macro_def = MacroDefinition {
            name: Ident::new("__DEBUG__".to_string()),
            params: vec![Ident::new("msg".to_string())],
            body: vec![
                crate::lexer::Token::new(
                    crate::lexer::TokenKind::Ident("__println__".to_string()),
                    crate::error::Span::new(
                        crate::error::Position::new(1, 1),
                        crate::error::Position::new(1, 11),
                    ),
                    "__println__".to_string(),
                ),
                crate::lexer::Token::new(
                    crate::lexer::TokenKind::LParen,
                    crate::error::Span::new(
                        crate::error::Position::new(1, 11),
                        crate::error::Position::new(1, 12),
                    ),
                    "(".to_string(),
                ),
                crate::lexer::Token::new(
                    crate::lexer::TokenKind::Ident("msg".to_string()),
                    crate::error::Span::new(
                        crate::error::Position::new(1, 12),
                        crate::error::Position::new(1, 15),
                    ),
                    "msg".to_string(),
                ),
                crate::lexer::Token::new(
                    crate::lexer::TokenKind::RParen,
                    crate::error::Span::new(
                        crate::error::Position::new(1, 15),
                        crate::error::Position::new(1, 16),
                    ),
                    ")".to_string(),
                ),
            ],
            delimiter: MacroDelimiter::Parens,
        };

        let file = File {
            items: vec![Item::MacroDefinition(macro_def)],
            doc_comments: vec![],
        };

        let output = gen.generate(&file);
        assert!(output.contains("macro_rules! debug"));
        assert!(output.contains("println!")); // __println__ should become println!
        assert!(output.contains("$msg"));
    }

    #[test]
    fn test_generate_macro_with_ternary() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let macro_def = MacroDefinition {
            name: Ident::new("__MAX__".to_string()),
            params: vec![Ident::new("a".to_string()), Ident::new("b".to_string())],
            body: vec![
                crate::lexer::Token::new(
                    crate::lexer::TokenKind::Ident("a".to_string()),
                    crate::error::Span::new(
                        crate::error::Position::new(1, 1),
                        crate::error::Position::new(1, 2),
                    ),
                    "a".to_string(),
                ),
                crate::lexer::Token::new(
                    crate::lexer::TokenKind::Gt,
                    crate::error::Span::new(
                        crate::error::Position::new(1, 3),
                        crate::error::Position::new(1, 4),
                    ),
                    ">".to_string(),
                ),
                crate::lexer::Token::new(
                    crate::lexer::TokenKind::Ident("b".to_string()),
                    crate::error::Span::new(
                        crate::error::Position::new(1, 5),
                        crate::error::Position::new(1, 6),
                    ),
                    "b".to_string(),
                ),
                crate::lexer::Token::new(
                    crate::lexer::TokenKind::Question,
                    crate::error::Span::new(
                        crate::error::Position::new(1, 7),
                        crate::error::Position::new(1, 8),
                    ),
                    "?".to_string(),
                ),
                crate::lexer::Token::new(
                    crate::lexer::TokenKind::Ident("a".to_string()),
                    crate::error::Span::new(
                        crate::error::Position::new(1, 9),
                        crate::error::Position::new(1, 10),
                    ),
                    "a".to_string(),
                ),
                crate::lexer::Token::new(
                    crate::lexer::TokenKind::Colon,
                    crate::error::Span::new(
                        crate::error::Position::new(1, 11),
                        crate::error::Position::new(1, 12),
                    ),
                    ":".to_string(),
                ),
                crate::lexer::Token::new(
                    crate::lexer::TokenKind::Ident("b".to_string()),
                    crate::error::Span::new(
                        crate::error::Position::new(1, 13),
                        crate::error::Position::new(1, 14),
                    ),
                    "b".to_string(),
                ),
            ],
            delimiter: MacroDelimiter::Parens,
        };

        let file = File {
            items: vec![Item::MacroDefinition(macro_def)],
            doc_comments: vec![],
        };

        let output = gen.generate(&file);
        assert!(output.contains("macro_rules! max"));
        assert!(output.contains("$a:expr"));
        assert!(output.contains("$b:expr"));
        // Ternary should be translated to if-else
        assert!(output.contains("if") || output.contains("?"));
    }

    #[test]
    fn test_generate_macro_removes_double_underscores() {
        let mut gen = CodeGenerator::new(TargetLanguage::Rust);
        let macro_def = MacroDefinition {
            name: Ident::new("__MY_MACRO__".to_string()),
            params: vec![],
            body: vec![crate::lexer::Token::new(
                crate::lexer::TokenKind::IntLiteral("42".to_string()),
                crate::error::Span::new(
                    crate::error::Position::new(1, 1),
                    crate::error::Position::new(1, 3),
                ),
                "42".to_string(),
            )],
            delimiter: MacroDelimiter::None,
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
            body: vec![],
            delimiter: MacroDelimiter::None,
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
        let result = gen.generate_literal_string(&lit);
        assert_eq!(result, "Option::None");
    }

    #[test]
    fn test_generate_null_literal_to_crusty() {
        // Test: NULL → NULL (when generating Crusty)
        let gen = CodeGenerator::new(TargetLanguage::Crusty);
        let lit = Literal::Null;
        let result = gen.generate_literal_string(&lit);
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
            body: Block::new(vec![stmt]),
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
            body: Block::new(vec![stmt]),
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
            body: Block::new(vec![stmt]),
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
            body: Block::new(vec![stmt]),
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
            body: Block::new(vec![stmt]),
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
            body: Block::empty(),
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
    use super::*;

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
