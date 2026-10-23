// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Code generation module for emitting Rust or Crusty source code.

use crate::ast::*;
use crate::semantic::{Capture, CaptureKind};
use std::collections::HashMap;

/// Target language for code generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TargetLanguage {
    Rust,
    Crusty,
}

/// C / POSIX type aliases recognised as identifier types and lowered to their
/// Rust equivalents (ADR-0003). These let idiomatic C source using the
/// `<stdint.h>` / POSIX spellings transpile unmodified. A matching user
/// `typedef` resolves earlier and so wins over this table. Single-token names
/// only. The alias maps to a `PrimitiveType`, so emission reuses the one
/// primitive-to-Rust map rather than duplicating the Rust spellings here.
fn c_type_alias_primitive(name: &str) -> Option<PrimitiveType> {
    Some(match name {
        "int8_t" => PrimitiveType::I8,
        "int16_t" => PrimitiveType::I16,
        "int32_t" => PrimitiveType::I32,
        "int64_t" => PrimitiveType::I64,
        "uint8_t" | "byte" => PrimitiveType::U8,
        "uint16_t" => PrimitiveType::U16,
        "uint32_t" => PrimitiveType::U32,
        "uint64_t" => PrimitiveType::U64,
        "size_t" | "uintptr_t" => PrimitiveType::Usize,
        "ssize_t" | "ptrdiff_t" | "intptr_t" => PrimitiveType::Isize,
        _ => return None,
    })
}

/// Code generator that produces source code from AST
pub struct CodeGenerator {
    target: TargetLanguage,
    indent_level: usize,
    output: String,
    /// Capture information for nested functions: function_name -> captures
    nested_function_captures: HashMap<String, Vec<Capture>>,
    /// Names the program defines via `typedef`. A C/POSIX alias (ADR-0003) is
    /// applied only when the name is NOT here, so a user typedef of the same
    /// name wins.
    user_type_names: std::collections::HashSet<String>,
}

impl CodeGenerator {
    /// Create a new code generator for the specified target language
    pub fn new(target: TargetLanguage) -> Self {
        Self {
            target,
            indent_level: 0,
            output: String::new(),
            nested_function_captures: HashMap::new(),
            user_type_names: std::collections::HashSet::new(),
        }
    }

    /// Set capture information for nested functions from semantic analysis
    #[allow(dead_code)] // Used in tests
    pub fn set_captures(&mut self, captures: HashMap<String, Vec<Capture>>) {
        self.nested_function_captures = captures;
    }

    // Test-only accessors for white-box unit tests in `codegen_unit_tests`.
    // `#[allow(dead_code)]`: the bin target compiles this module's `#[cfg(test)]`
    // code but not the sibling test module that uses these, so it sees them as
    // unused; the lib target does exercise them.

    /// Target language (for testing)
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn target(&self) -> TargetLanguage {
        self.target
    }

    /// Current indentation level (for testing)
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn indent_level(&self) -> usize {
        self.indent_level
    }

    /// Increase indentation (for testing)
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn indent_for_test(&mut self) {
        self.indent();
    }

    /// Decrease indentation (for testing)
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn dedent_for_test(&mut self) {
        self.dedent();
    }

    /// Generate a literal as a string (for testing)
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn generate_literal_string_for_test(&self, lit: &Literal) -> String {
        self.generate_literal_string(lit)
    }

    /// Generate source code from a File AST
    pub fn generate(&mut self, file: &File) -> String {
        self.output.clear();
        self.indent_level = 0;

        // Record user typedef names so built-in C/POSIX aliases defer to them
        // (ADR-0003). Top-level typedefs are the common case; a typedef shadowed
        // only inside a namespace still falls back to the alias, which is
        // acceptable for this compatibility feature.
        self.user_type_names.clear();
        for item in &file.items {
            if let Item::Typedef(td) = item {
                self.user_type_names.insert(td.name.name.clone());
            }
        }

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
            Item::ConditionalCompilation(cc) => self.generate_conditional_compilation(cc),
            Item::MacroInvocation {
                name,
                args,
                delimiter,
            } => self.generate_macro_invocation(name, args, delimiter),
            Item::RawRust { code } => self.generate_raw_rust(code),
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

                // C-style varargs: emit a trailing `...` (with a leading
                // comma when there are named parameters before it).
                if func.is_variadic {
                    if !func.params.is_empty() {
                        self.write(", ");
                    }
                    self.write("...");
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

                // C-style varargs marker.
                if func.is_variadic {
                    if !func.params.is_empty() {
                        self.write(", ");
                    }
                    self.write("...");
                }

                self.write(")");
            }
        }

        match &func.body {
            Some(block) => {
                self.write(" ");
                self.generate_block(block);
            }
            // Body-less declaration (extern fn): terminate with `;`.
            None => self.write(";"),
        }
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

    fn generate_namespace(&mut self, namespace: &Namespace) {
        // `namespace Name { … }` → `pub mod Name { … }`. Items nest via
        // generate_item, matching the file-level item loop (blank line between
        // items, current indentation respected).
        for comment in &namespace.doc_comments {
            self.write_line(&format!("///{}", comment));
        }
        self.write_line(&format!("pub mod {} {{", namespace.name.name));
        self.indent();
        for (i, item) in namespace.items.iter().enumerate() {
            if i > 0 {
                self.write_line("");
            }
            self.generate_item(item);
        }
        self.dedent();
        self.write_line("}");
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

    fn generate_extern(&mut self, extern_block: &Extern) {
        // `extern "ABI" { … }` → Rust extern block. A missing ABI defaults to
        // Rust's (bare `extern`). The contained items are body-less function
        // declarations (Function with body: None), emitted via generate_item,
        // which terminates each with a `;`.
        match &extern_block.abi {
            Some(abi) => self.write_line(&format!("extern \"{}\" {{", abi)),
            None => self.write_line("extern {"),
        }
        self.indent();
        for (i, item) in extern_block.items.iter().enumerate() {
            if i > 0 {
                self.write_line("");
            }
            self.generate_item(item);
        }
        self.dedent();
        self.write_line("}");
    }

    /// Emit a `__rust__{ … }` escape hatch: the captured raw Rust source,
    /// verbatim, one line per source line at the current indent level. Blank
    /// lines are preserved without indentation so trailing whitespace is not
    /// introduced. The code is passed through untouched — Crusty does not
    /// parse or validate it; rustc is the arbiter.
    fn generate_raw_rust(&mut self, code: &str) {
        for line in code.lines() {
            let trimmed = line.trim_end();
            if trimmed.is_empty() {
                self.write("\n");
            } else {
                self.write_line(trimmed.trim_start());
            }
        }
    }

    /// Emit an item-position macro invocation: `name! <delim> body <delim>`.
    /// The body span is passed through verbatim (ADR-0001) and the invocation
    /// delimiter is preserved, so `bitflags! { … }` stays brace-form and
    /// `my_macro!( … )` stays paren-form. A paren/bracket invocation is an item
    /// statement and gets a trailing `;`; a brace invocation is a Rust item and
    /// takes none.
    fn generate_macro_invocation(
        &mut self,
        name: &Ident,
        args: &[crate::ast::Token],
        delimiter: &crate::ast::MacroDelimiter,
    ) {
        let (open, close, semi) = match delimiter {
            crate::ast::MacroDelimiter::Parens => ('(', ')', ";"),
            crate::ast::MacroDelimiter::Brackets => ('[', ']', ";"),
            crate::ast::MacroDelimiter::Braces => ('{', '}', ""),
        };
        let body = args.first().map(|t| t.text.as_str()).unwrap_or("");
        self.write_line(&format!("{}!{}{}{}{}", name.name, open, body, close, semi));
    }

    fn generate_const(&mut self, const_item: &Const) {
        // A `#define __NAME__ value` lowers to a Rust `const` (ADR-0001).
        let ty = self.generate_type_string(&const_item.ty);
        let value = self.generate_expression_string(&const_item.value);
        self.write_line(&format!(
            "const {}: {} = {};",
            const_item.name.name, ty, value
        ));
    }

    fn generate_static(&mut self, static_item: &Static) {
        // `static [mut] Type NAME = value;` → `[pub] static [mut] NAME: Type = value;`
        for comment in &static_item.doc_comments {
            self.write_line(&format!("///{}", comment));
        }
        let visibility = match static_item.visibility {
            Visibility::Public => "pub ",
            Visibility::Private => "",
        };
        let mutability = if static_item.mutable { "mut " } else { "" };
        let ty = self.generate_type_string(&static_item.ty);
        let value = self.generate_expression_string(&static_item.value);
        self.write_line(&format!(
            "{}static {}{}: {} = {};",
            visibility, mutability, static_item.name.name, ty, value
        ));
    }

    fn generate_macro_definition(&mut self, macro_def: &MacroDefinition) {
        // Translate `#macro __NAME__(a, b) body` to a Rust `macro_rules!`.
        // The body is a raw source span (ADR-0001): Crusty does not tokenise it,
        // so parameter substitution is a plain textual `name` -> `$name` pass and
        // the rest is emitted verbatim for rustc to parse.
        let rust_name = macro_def
            .name
            .name
            .trim_start_matches("__")
            .trim_end_matches("__")
            .to_lowercase();

        // Avoid colliding with a Rust keyword.
        let rust_name = if is_rust_keyword(&rust_name) {
            format!("{}_macro", rust_name)
        } else {
            rust_name
        };

        self.write_line(&format!("macro_rules! {} {{", rust_name));
        self.indent();

        // Parameter pattern: ($a:expr, $b:expr)
        self.write_indent();
        self.write("(");
        for (i, param) in macro_def.params.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            self.write(&format!("${}:expr", param.name));
        }
        self.write(") => {{\n");
        self.indent();

        // Body: substitute each parameter reference with its `$name` form.
        self.write_indent();
        self.write(&substitute_macro_params(&macro_def.body, &macro_def.params));
        self.write("\n");

        self.dedent();
        self.write_indent();
        self.write("}};\n");

        self.dedent();
        self.write_line("}");
    }

    fn generate_conditional_compilation(&mut self, cc: &ConditionalCompilation) {
        // `#ifdef FOO` / `#ifndef FOO` lowers to `#[cfg(FOO)]` /
        // `#[cfg(not(FOO))]` on each contained item (ADR-0001).
        let predicate = if cc.negated {
            format!("not({})", cc.cfg.name)
        } else {
            cc.cfg.name.clone()
        };
        for item in &cc.items {
            self.write_line(&format!("#[cfg({})]", predicate));
            self.generate_item(item);
        }
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
                        // Crusty uses C-style syntax - no type annotations for let
                        // If there's a type but no cast in the init, we need to generate a cast
                        self.write("let ");
                        self.write(&name.name);
                        if let Some(ref init) = init {
                            self.write(" = ");
                            // If there's a type annotation in the AST but the init is not a cast,
                            // wrap it in a cast to preserve the type information
                            if let Some(ref ty) = ty {
                                if !matches!(init, Expression::Cast { .. }) {
                                    self.write("(");
                                    self.write(&self.generate_type_string(ty));
                                    self.write(")");
                                }
                            }
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
                        // Crusty uses C-style syntax - no type annotations for var
                        // If there's a type but no cast in the init, we need to generate a cast
                        self.write("var ");
                        self.write(&name.name);
                        if let Some(ref init) = init {
                            self.write(" = ");
                            // If there's a type annotation in the AST but the init is not a cast,
                            // wrap it in a cast to preserve the type information
                            if let Some(ref ty) = ty {
                                if !matches!(init, Expression::Cast { .. }) {
                                    self.write("(");
                                    self.write(&self.generate_type_string(ty));
                                    self.write(")");
                                }
                            }
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
                        // Crusty uses C-style syntax - no type annotations
                        // Type is specified via cast expression
                        self.write("const ");
                        self.write(&name.name);
                        self.write(" = ");
                        // If there's a type but the value is not a cast, wrap it
                        if !matches!(value, Expression::Cast { .. }) {
                            self.write("(");
                            self.write(&self.generate_type_string(ty));
                            self.write(")");
                        }
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
            Statement::Loop { label, body } => {
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
                // `loop` is spelled identically in Crusty and Rust.
                self.write("loop ");
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
            Statement::Break { label, value } => {
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
                if let Some(ref value) = value {
                    self.write(" ");
                    self.write(&self.generate_expression_string(value));
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
                // Generate closure with appropriate capture semantics
                self.write_indent();
                self.write("let ");

                // Determine if we need mut based on captures
                let captures = self.nested_function_captures.get(&name.name);
                let has_mutable_captures = captures
                    .map(|caps| caps.iter().any(|c| matches!(c.kind, CaptureKind::Mutable)))
                    .unwrap_or(false);

                if has_mutable_captures {
                    self.write("mut ");
                }

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
                // Assignment and compound-assignment (`=`, `+=`, …) are
                // statement-level place-expressions whose value is `()`. They
                // must NOT be wrapped in parens: `(i += 1);` is not a valid
                // statement/expression in Rust, and `x = (a = b)` would try to
                // use the `()` of the inner assignment. Every other binary op is
                // a value expression and keeps its parens.
                if op.is_assignment() {
                    format!(
                        "{} {} {}",
                        self.generate_expression_string(left),
                        self.generate_binary_op_string(op),
                        self.generate_expression_string(right)
                    )
                } else {
                    format!(
                        "({} {} {})",
                        self.generate_expression_string(left),
                        self.generate_binary_op_string(op),
                        self.generate_expression_string(right)
                    )
                }
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
            Expression::MacroCall {
                name,
                args,
                delimiter,
            } => {
                // Preserve the invocation delimiter (ADR-0001): `__vec__[1, 2]`
                // lowers to `vec![1, 2]`, not `vec!(1, 2)`. The argument span is
                // passed through verbatim.
                let (open, close) = match delimiter {
                    crate::ast::MacroDelimiter::Parens => ('(', ')'),
                    crate::ast::MacroDelimiter::Brackets => ('[', ']'),
                    crate::ast::MacroDelimiter::Braces => ('{', '}'),
                };
                let body = args.first().map(|t| t.text.as_str()).unwrap_or("");
                format!("{}!{}{}{}", name.name, open, body, close)
            }
            Expression::Loop { label, body } => {
                // Render the body with a sub-generator so we reuse the real
                // statement/block emitter (generate_block is &mut self; this
                // string context is &self). The sub-generator inherits the
                // target, indent level, and nested-function captures.
                let mut sub = CodeGenerator::new(self.target);
                sub.indent_level = self.indent_level;
                sub.nested_function_captures = self.nested_function_captures.clone();
                sub.generate_block(body);
                let body_str = sub.output;

                let mut result = String::new();
                if let Some(label) = label {
                    match self.target {
                        TargetLanguage::Rust => {
                            result.push('\'');
                            result.push_str(&label.name);
                            result.push_str(": ");
                        }
                        TargetLanguage::Crusty => {
                            result.push('.');
                            result.push_str(&label.name);
                            result.push_str(": ");
                        }
                    }
                }
                result.push_str("loop ");
                result.push_str(&body_str);
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
            Expression::Comma { left, right } => {
                // Comma expression: (left, right)
                // In Rust, we use a block with both expressions
                let mut result = String::from("{ ");
                result.push_str(&self.generate_expression_string(left));
                result.push_str("; ");
                result.push_str(&self.generate_expression_string(right));
                result.push_str(" }");
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
            // Verbatim passthrough: the source already is valid Rust (non-decimal
            // int, digit separators, numeric suffix, raw/byte string). ADR-0001.
            Literal::Raw(s) => s.clone(),
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
            BinaryOp::ModAssign => "%=",
            BinaryOp::BitAndAssign => "&=",
            BinaryOp::BitOrAssign => "|=",
            BinaryOp::BitXorAssign => "^=",
            BinaryOp::ShlAssign => "<<=",
            BinaryOp::ShrAssign => ">>=",
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
                // ++x: borrow the place once, increment, yield the new value.
                format!(
                    "{{ let __p = &mut ({}); *__p += 1; *__p }}",
                    self.generate_expression_string(expr)
                )
            }
            UnaryOp::PreDec => {
                // --x: borrow the place once, decrement, yield the new value.
                format!(
                    "{{ let __p = &mut ({}); *__p -= 1; *__p }}",
                    self.generate_expression_string(expr)
                )
            }
            UnaryOp::PostInc => {
                // x++: borrow the place ONCE (so the operand is evaluated once
                // and the increment targets the real place, not a copy), read
                // the old value, increment, yield the old value.
                format!(
                    "{{ let __p = &mut ({}); let __old = *__p; *__p += 1; __old }}",
                    self.generate_expression_string(expr)
                )
            }
            UnaryOp::PostDec => {
                // x--: borrow the place once, read old, decrement, yield old.
                format!(
                    "{{ let __p = &mut ({}); let __old = *__p; *__p -= 1; __old }}",
                    self.generate_expression_string(expr)
                )
            }
        }
    }

    /// Generate a type as string
    pub fn generate_type_string(&self, ty: &Type) -> String {
        match ty {
            Type::Primitive(prim) => self.generate_primitive_type_string(prim),
            Type::Ident(ident) => {
                // A C/POSIX alias (ADR-0003) lowers to its Rust type when
                // targeting Rust — unless the program defines a typedef of the
                // same name, which wins. Otherwise the identifier is verbatim.
                match self.target {
                    TargetLanguage::Rust if !self.user_type_names.contains(&ident.name) => {
                        match c_type_alias_primitive(&ident.name) {
                            Some(prim) => self.generate_primitive_type_string(&prim),
                            None => ident.name.clone(),
                        }
                    }
                    _ => ident.name.clone(),
                }
            }
            Type::Pointer { ty, mutable } => {
                // Translate pointers to raw pointers
                if *mutable {
                    format!("*mut {}", self.generate_type_string(ty))
                } else {
                    format!("*const {}", self.generate_type_string(ty))
                }
            }
            Type::Reference { ty, mutable } => {
                // & and var &/&mut translate to Rust references
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
                PrimitiveType::I8 => "i8".to_string(),
                PrimitiveType::I16 => "i16".to_string(),
                PrimitiveType::I32 => "i32".to_string(),
                PrimitiveType::I64 => "i64".to_string(),
                PrimitiveType::I128 => "i128".to_string(),
                PrimitiveType::U8 => "u8".to_string(),
                PrimitiveType::U16 => "u16".to_string(),
                PrimitiveType::U32 => "u32".to_string(),
                PrimitiveType::U64 => "u64".to_string(),
                PrimitiveType::U128 => "u128".to_string(),
                PrimitiveType::Usize => "usize".to_string(),
                PrimitiveType::Isize => "isize".to_string(),
                PrimitiveType::Float => "f64".to_string(),
                PrimitiveType::F32 => "f32".to_string(),
                PrimitiveType::F64 => "f64".to_string(),
                PrimitiveType::Bool => "bool".to_string(),
                PrimitiveType::Char => "char".to_string(),
                PrimitiveType::Void => "()".to_string(),
            },
            TargetLanguage::Crusty => match prim {
                PrimitiveType::Int => "int".to_string(),
                PrimitiveType::I8 => "i8".to_string(),
                PrimitiveType::I16 => "i16".to_string(),
                PrimitiveType::I32 => "i32".to_string(),
                PrimitiveType::I64 => "i64".to_string(),
                PrimitiveType::I128 => "i128".to_string(),
                PrimitiveType::U8 => "u8".to_string(),
                PrimitiveType::U16 => "u16".to_string(),
                PrimitiveType::U32 => "u32".to_string(),
                PrimitiveType::U64 => "u64".to_string(),
                PrimitiveType::U128 => "u128".to_string(),
                PrimitiveType::Usize => "usize".to_string(),
                PrimitiveType::Isize => "isize".to_string(),
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

/// Rewrite a `#macro` body so each parameter reference becomes its `$name`
/// form for `macro_rules!`. The body is raw source text (ADR-0001), so this is
/// a word-boundary textual substitution — no tokenisation. A character is part
/// of an identifier when it is alphanumeric or `_`; a `name` occurrence
/// flanked by non-identifier characters (or string ends) is a reference.
fn substitute_macro_params(body: &str, params: &[Ident]) -> String {
    if params.is_empty() {
        return body.to_string();
    }
    let is_ident_char = |c: char| c.is_alphanumeric() || c == '_';
    let chars: Vec<char> = body.chars().collect();
    let mut out = String::with_capacity(body.len());
    let mut i = 0;
    while i < chars.len() {
        if is_ident_char(chars[i]) {
            // Consume a whole identifier run.
            let start = i;
            while i < chars.len() && is_ident_char(chars[i]) {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            if params.iter().any(|p| p.name == word) {
                out.push('$');
            }
            out.push_str(&word);
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
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
