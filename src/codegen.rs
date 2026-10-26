// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Code generation module for emitting Rust or Crusty source code.

use crate::ast::*;
use crate::semantic::Capture;
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
    /// When inside a `__scope__` body (ADR-0019), the name of the enclosing
    /// scope handle, so a `__thread__` emits a scoped borrowing spawn
    /// (`<handle>.spawn(|| …)`) instead of the top-level `std::thread::spawn(
    /// move || …)`. `None` at top level.
    scope_handle: Option<String>,
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
            scope_handle: None,
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

    /// Emit a list of outer attributes verbatim (ADR-0020), each on its own
    /// indented line. The attribute text already includes `#[ … ]`.
    fn write_attributes(&mut self, attributes: &[crate::ast::Attribute]) {
        for attr in attributes {
            self.write_indent();
            self.write(&attr.text);
            self.write("\n");
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
            Item::InnerAttribute { text } => {
                // Verbatim crate-level inner attribute (ADR-0020). Emitted as-is
                // at top level; rustc requires inner attributes before any item,
                // which source ordering preserves.
                self.write(text);
                self.write("\n");
            }
            Item::ImplBlock(impl_block) => self.generate_impl_block(impl_block),
            Item::Trait(trait_def) => self.generate_trait(trait_def),
        }
    }

    fn generate_function(&mut self, func: &Function) {
        // Generate attributes
        self.write_attributes(&func.attributes);

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

                // `unsafe` goes after `pub`, before `fn` (PS-09 Unit 1).
                if func.is_unsafe {
                    self.write("unsafe ");
                }

                self.write("fn ");
                self.write(&func.name.name);
                self.write(&self.generate_generic_params_string(&func.generic_params));
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
                // Crusty syntax: unsafe? static? return_type name(params) { }
                if func.is_unsafe {
                    self.write("unsafe ");
                }
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
                self.write(&self.generate_generic_params_string(&func.generic_params));
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

        // Where clause (verbatim, ADR-0011 Phase 2) sits between the
        // signature and the body for both targets.
        if let Some(ref where_clause) = func.where_clause {
            self.write(" where ");
            self.write(where_clause);
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
        self.write_attributes(&struct_def.attributes);

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
        self.write(&self.generate_generic_params_string(&struct_def.generic_params));

        // Body shape (ADR-0024): named `{ … }`, tuple `( … );`, or unit `;`.
        match &struct_def.body {
            StructBody::Named(fields) => {
                if let Some(ref where_clause) = struct_def.where_clause {
                    self.write(" where ");
                    self.write(where_clause);
                }
                self.write(" {\n");
                self.indent();
                for field in fields {
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
            }
            StructBody::Tuple(types) => {
                // `struct Name(T0, T1);` — positional fields, no names.
                self.write("(");
                let rendered: Vec<String> =
                    types.iter().map(|t| self.generate_type_string(t)).collect();
                self.write(&rendered.join(", "));
                self.write(")");
                if let Some(ref where_clause) = struct_def.where_clause {
                    self.write(" where ");
                    self.write(where_clause);
                }
                self.write(";\n");
            }
            StructBody::Unit => {
                // `struct Name;` — no fields.
                if let Some(ref where_clause) = struct_def.where_clause {
                    self.write(" where ");
                    self.write(where_clause);
                }
                self.write(";\n");
            }
        }

        // Generate impl block for methods if any
        if !struct_def.methods.is_empty() {
            self.write_line("");
            self.write_indent();
            self.write("impl");
            self.write(&self.generate_generic_params_string(&struct_def.generic_params));
            self.write(" ");
            self.write(&struct_def.name.name);
            self.write(&self.generate_generic_args_string(&struct_def.generic_params));
            if let Some(ref where_clause) = struct_def.where_clause {
                self.write(" where ");
                self.write(where_clause);
            }
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

    /// Generate a `typedef … @Type` implementation block (PS-09 Unit 2,
    /// ADR-0015). Rust: `impl [Trait for] Target { <methods> }`. Crusty:
    /// round-trips to `typedef struct|default { <methods> } @Target[.name];`.
    fn generate_impl_block(&mut self, impl_block: &ImplBlock) {
        self.write_attributes(&impl_block.attributes);
        self.write_indent();
        match self.target {
            TargetLanguage::Rust => {
                self.write("impl ");
                if let Some(ref trait_name) = impl_block.trait_name {
                    self.write(&trait_name.name);
                    self.write(" for ");
                }
                self.write(&impl_block.target.name);
                self.write(" {\n");
                self.indent();
                for (i, method) in impl_block.methods.iter().enumerate() {
                    if i > 0 {
                        self.write_line("");
                    }
                    if impl_block.trait_name.is_some() {
                        // Trait-impl methods inherit the trait's visibility —
                        // a `pub` qualifier is rejected by rustc (E0449). Emit
                        // them private so no `pub` is written.
                        let mut m = method.clone();
                        m.visibility = Visibility::Private;
                        self.generate_function(&m);
                    } else {
                        self.generate_function(method);
                    }
                }
                self.dedent();
                self.write_line("}");
            }
            TargetLanguage::Crusty => {
                self.write("typedef ");
                self.write(match impl_block.trait_name {
                    Some(_) => "default",
                    None => "struct",
                });
                self.write(" {\n");
                self.indent();
                for method in &impl_block.methods {
                    self.generate_function(method);
                }
                self.dedent();
                self.write_indent();
                self.write("} @");
                self.write(&impl_block.target.name);
                if let Some(ref name) = impl_block.name {
                    self.write(".");
                    self.write(&name.name);
                }
                self.write(";\n");
            }
        }
    }

    /// Emit a trait definition (PS-09 Unit 3 Part B, ADR-0016). A method with
    /// `body: None` emits a signature (`fn f(&self) -> R;`); a default method
    /// emits its body. Trait items inherit the trait's visibility, so no `pub`
    /// is written on them (rustc E0449) — methods are emitted private.
    fn generate_trait(&mut self, trait_def: &Trait) {
        self.write_attributes(&trait_def.attributes);
        self.write_indent();
        match self.target {
            TargetLanguage::Rust => {
                match trait_def.visibility {
                    Visibility::Public => self.write("pub "),
                    Visibility::Private => {}
                }
                self.write("trait ");
                self.write(&trait_def.name.name);
                self.write(&self.generate_generic_params_string(&trait_def.generic_params));
                self.write(" {\n");
                self.indent();
                for (i, method) in trait_def.methods.iter().enumerate() {
                    if i > 0 {
                        self.write_line("");
                    }
                    // Trait items share the trait's visibility — a `pub`
                    // qualifier is rejected by rustc (E0449).
                    let mut m = method.clone();
                    m.visibility = Visibility::Private;
                    self.generate_function(&m);
                }
                self.dedent();
                self.write_line("}");
            }
            TargetLanguage::Crusty => {
                if matches!(trait_def.visibility, Visibility::Private) {
                    self.write("static ");
                }
                self.write("typedef trait {\n");
                self.indent();
                for method in &trait_def.methods {
                    self.generate_function(method);
                }
                self.dedent();
                self.write_indent();
                self.write("} ");
                self.write(&trait_def.name.name);
                self.write(&self.generate_generic_params_string(&trait_def.generic_params));
                self.write(";\n");
            }
        }
    }

    fn generate_enum(&mut self, enum_def: &Enum) {
        // Generate attributes
        self.write_attributes(&enum_def.attributes);

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
        self.write(&self.generate_generic_params_string(&enum_def.generic_params));
        if let Some(ref where_clause) = enum_def.where_clause {
            self.write(" where ");
            self.write(where_clause);
        }
        self.write(" {\n");
        self.indent();

        // Generate variants (ADR-0023): unit / discriminant / tuple / struct.
        for variant in &enum_def.variants {
            self.write_indent();
            self.write(&variant.name.name);
            match &variant.body {
                crate::ast::VariantBody::Unit => {}
                crate::ast::VariantBody::Discriminant(value) => {
                    self.write(&format!(" = {}", value));
                }
                crate::ast::VariantBody::Tuple(types) => {
                    self.write("(");
                    let rendered: Vec<String> =
                        types.iter().map(|t| self.generate_type_string(t)).collect();
                    self.write(&rendered.join(", "));
                    self.write(")");
                }
                crate::ast::VariantBody::Struct(fields) => {
                    self.write(" { ");
                    let rendered: Vec<String> = fields
                        .iter()
                        .map(|f| format!("{}: {}", f.name.name, self.generate_type_string(&f.ty)))
                        .collect();
                    self.write(&rendered.join(", "));
                    self.write(" }");
                }
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
        // The C `#ifdef` family (ADR-0022) lowers to per-item `#[cfg(...)]`.
        // Rust has no block preprocessor and no cfg-level else-if, so an
        // else-if chain becomes a set of INDEPENDENT, mutually-exclusive cfg
        // attributes: arm k is active only when its own predicate holds AND
        // every earlier arm's predicate failed. So arm k emits under
        // `all(not(c_0), …, not(c_{k-1}), c_k)`, and the `#else` items under the
        // negation of every arm. A single arm with no `#else` collapses to a
        // bare `#[cfg(NAME)]` (byte-identical to the pre-chain output).
        //
        // `c_i` is the arm's own predicate; `not_c_i` is its negation. For a
        // negated arm (`#ifndef`/`#elifndef`) these swap.
        let predicate = |arm: &crate::ast::CondArm| -> String {
            if arm.negated {
                format!("not({})", arm.cfg.name)
            } else {
                arm.cfg.name.clone()
            }
        };
        let negation = |arm: &crate::ast::CondArm| -> String {
            if arm.negated {
                arm.cfg.name.clone()
            } else {
                format!("not({})", arm.cfg.name)
            }
        };

        // Prior-arm negations accumulate as we walk the chain.
        let mut prior_negations: Vec<String> = Vec::new();
        for arm in &cc.arms {
            // The cfg for this arm: all prior negations plus this arm's own
            // predicate. With no prior negations and a single term, emit the
            // bare predicate (no `all(...)` wrapper) so existing output is
            // unchanged.
            let own = predicate(arm);
            let cfg = if prior_negations.is_empty() {
                own
            } else {
                let mut terms = prior_negations.clone();
                terms.push(own);
                format!("all({})", terms.join(", "))
            };
            for item in &arm.items {
                self.write_line(&format!("#[cfg({})]", cfg));
                self.generate_item(item);
            }
            prior_negations.push(negation(arm));
        }

        // The `#else` items: active only when every arm failed.
        if !cc.else_items.is_empty() {
            let cfg = match prior_negations.len() {
                1 => prior_negations[0].clone(),
                _ => format!("all({})", prior_negations.join(", ")),
            };
            for item in &cc.else_items {
                self.write_line(&format!("#[cfg({})]", cfg));
                self.generate_item(item);
            }
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
            Statement::IfBind {
                init,
                condition,
                then_block,
                else_block,
            } => {
                // ADR-0026: wrap the init + plain `if` in a block so the binding
                // is scoped to the whole statement (C99 for-header semantics)
                // and dropped after.
                self.write_indent();
                self.write("{\n");
                self.indent();
                self.generate_statement(init);
                // Reuse the plain-If emitter for identical condition/else output.
                self.generate_statement(&Statement::If {
                    condition: condition.clone(),
                    then_block: then_block.clone(),
                    else_block: else_block.clone(),
                });
                self.dedent();
                self.write_indent();
                self.write("}\n");
            }
            Statement::WhileBind {
                label,
                init,
                condition,
                body,
            } => {
                // ADR-0026: init runs once before the loop, scoped to the block.
                self.write_indent();
                self.write("{\n");
                self.indent();
                self.generate_statement(init);
                self.generate_statement(&Statement::While {
                    label: label.clone(),
                    condition: condition.clone(),
                    body: body.clone(),
                });
                self.dedent();
                self.write_indent();
                self.write("}\n");
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
                // PS-07 (ADR-0014): a named nested function lowers to a Rust
                // `fn` ITEM inside the enclosing body — hoisted (callable before
                // its definition), recursive, and NON-capturing. (Capture lives
                // on `__func__` anonymous functions, which lower to closures.)
                self.write_indent();
                self.write("fn ");
                self.write(&name.name);
                self.write("(");

                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write(&param.name.name);
                    self.write(": ");
                    let param_type = self.generate_type_string(&param.ty);
                    self.write(&param_type);
                }

                self.write(")");

                if let Some(ref ret_ty) = return_type {
                    self.write(" -> ");
                    let return_type_str = self.generate_type_string(ret_ty);
                    self.write(&return_type_str);
                }

                self.write(" ");
                self.generate_block(body);
                self.write("\n");
            }
            Statement::Attributed { attributes, stmt } => {
                // Emit each attribute verbatim (ADR-0020) on its own indented
                // line, then the inner statement (which writes its own
                // indentation and newline).
                self.write_attributes(attributes);
                self.generate_statement(stmt);
            }
            Statement::Unsafe(block) => {
                // PS-09 Unit 1 (ADR-0015): `unsafe { … }` — emit the keyword
                // then the block verbatim; rustc owns unsafety (ADR-0001).
                self.write_indent();
                self.write("unsafe ");
                self.generate_block(block);
                self.write("\n");
            }
        }
    }
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
                    // CG-1: emit parentheses only where precedence/associativity
                    // require them, instead of wrapping every value binary — the
                    // blanket parens tripped rustc's `unused_parens` lint.
                    format!(
                        "{} {} {}",
                        self.emit_binary_operand(left, op, false),
                        self.generate_binary_op_string(op),
                        self.emit_binary_operand(right, op, true)
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
            Expression::StructInit { ty, fields, base } => {
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
                if let Some(base) = base {
                    if !fields.is_empty() {
                        result.push_str(", ");
                    }
                    result.push_str("..");
                    result.push_str(&self.generate_expression_string(base));
                }
                result.push_str(" }");
                result
            }
            Expression::StructVariantInit {
                ty,
                variant,
                fields,
                base,
            } => {
                // `@Type.Variant { … }` -> `Type::Variant { … }` (ADR-0023
                // follow-up). Same field + functional-update emission as
                // StructInit, but the head is a `Type::Variant` path.
                let mut result = self.generate_type_string(ty);
                result.push_str("::");
                result.push_str(&variant.name);
                result.push_str(" { ");
                for (i, (name, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&name.name);
                    result.push_str(": ");
                    result.push_str(&self.generate_expression_string(value));
                }
                if let Some(base) = base {
                    if !fields.is_empty() {
                        result.push_str(", ");
                    }
                    result.push_str("..");
                    result.push_str(&self.generate_expression_string(base));
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
            Expression::ArrayRepeat { value, count } => {
                format!(
                    "[{}; {}]",
                    self.generate_expression_string(value),
                    self.generate_expression_string(count)
                )
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
            Expression::Thread { body } => {
                // PS-09 Unit 4 (ADR-0018 / ADR-0019): `__thread__ { body }`.
                // Top-level → `std::thread::spawn(move || { body })` (`move`
                // mandatory, ADR-0017 E0373). Inside a `__scope__` → a scoped
                // borrowing spawn `<handle>.spawn(|| { body })` (NO `move`, so
                // it borrows; the scope auto-joins). A `__thread__` nested
                // inside this one's body is top-level again, so the body
                // sub-generator resets `scope_handle` to None.
                let mut sub = CodeGenerator::new(self.target);
                sub.indent_level = self.indent_level;
                sub.nested_function_captures = self.nested_function_captures.clone();
                sub.generate_block(body);
                let body_str = sub.output;
                match self.target {
                    TargetLanguage::Rust => match &self.scope_handle {
                        Some(handle) => format!("{}.spawn(|| {})", handle, body_str),
                        None => format!("std::thread::spawn(move || {})", body_str),
                    },
                    TargetLanguage::Crusty => {
                        format!("__thread__ {}", body_str)
                    }
                }
            }
            Expression::Scope { body } => {
                // ADR-0019: `__scope__ { body }` → `std::thread::scope(
                // |__crusty_scope| { body })`. The body sub-generator carries
                // the scope-handle name so a `__thread__` inside it emits a
                // scoped borrowing spawn. The handle name is generated and
                // reserved by convention, never written by the user.
                match self.target {
                    TargetLanguage::Rust => {
                        let handle = "__crusty_scope";
                        let mut sub = CodeGenerator::new(self.target);
                        sub.indent_level = self.indent_level;
                        sub.nested_function_captures = self.nested_function_captures.clone();
                        sub.scope_handle = Some(handle.to_string());
                        sub.generate_block(body);
                        format!("std::thread::scope(|{}| {})", handle, sub.output)
                    }
                    TargetLanguage::Crusty => {
                        let mut sub = CodeGenerator::new(self.target);
                        sub.indent_level = self.indent_level;
                        sub.nested_function_captures = self.nested_function_captures.clone();
                        sub.generate_block(body);
                        format!("__scope__ {}", sub.output)
                    }
                }
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
            Expression::TypeScopedPath { ty, member } => {
                // Translate @Type.CONST to Type::CONST (no call).
                format!("{}::{}", self.generate_type_string(ty), member.name)
            }
            Expression::TurbofishCall {
                ty,
                method,
                generics,
                args,
            } => {
                // Translate Type::method::<G>(args) verbatim — the turbofish
                // binds to the method, so emit Type::method::<G>(args).
                let mut result = self.generate_type_string(ty);
                result.push_str("::");
                result.push_str(&method.name);
                result.push_str("::<");
                for (i, gen) in generics.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&self.generate_type_string(gen));
                }
                result.push_str(">(");
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
            Expression::Match { scrutinee, arms } => {
                // PS-06 (ADR-0012): lower to a Rust `match`, emitted inline as
                // an expression so `let x = match … { … };` works. Each arm is
                // `<pattern> [if <guard>] => <body>,`.
                let mut result = String::from("match ");
                result.push_str(&self.generate_expression_string(scrutinee));
                result.push_str(" {\n");
                for arm in arms {
                    result.push_str(&self.generate_pattern_string(&arm.pattern));
                    if let Some(ref guard) = arm.guard {
                        result.push_str(" if ");
                        result.push_str(&self.generate_expression_string(guard));
                    }
                    result.push_str(" => ");
                    result.push_str(&self.generate_expression_string(&arm.body));
                    result.push_str(",\n");
                }
                result.push('}');
                result
            }
            Expression::AnonFunction {
                params,
                return_type,
                body,
            } => {
                // PS-07 (ADR-0013/0014): `__func__` lowers to a Rust closure
                // `|p: T, …| -> R { body }`. Params carry their written types;
                // the return type is emitted when present (omitted ⇒ unit).
                // The body is rendered with a sub-generator so the real
                // statement/block emitter (generate_block is &mut self) is
                // reused from this &self string context — same idiom as Loop.
                let mut result = String::from("|");
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&param.name.name);
                    result.push_str(": ");
                    result.push_str(&self.generate_type_string(&param.ty));
                }
                result.push('|');
                if let Some(ref ret) = return_type {
                    result.push_str(" -> ");
                    result.push_str(&self.generate_type_string(ret));
                }
                result.push(' ');

                let mut sub = CodeGenerator::new(self.target);
                sub.indent_level = self.indent_level;
                sub.nested_function_captures = self.nested_function_captures.clone();
                sub.generate_block(body);
                result.push_str(&sub.output);
                result
            }
        }
    }

    /// Generate a match-arm pattern as a Rust string (PS-06, ADR-0012).
    /// Structural patterns are emitted from their parts; `Pattern::Raw` is
    /// emitted verbatim (ADR-0001 — rustc arbitrates what Crusty does not
    /// model structurally).
    fn generate_pattern_string(&self, pattern: &crate::ast::Pattern) -> String {
        use crate::ast::Pattern;
        match pattern {
            Pattern::Wildcard => "_".to_string(),
            Pattern::Literal(lit) => self.generate_literal_string(lit),
            Pattern::Binding(name) => name.name.clone(),
            Pattern::Range { lo, hi, inclusive } => {
                let mut result = self.generate_expression_string(lo);
                result.push_str("..");
                if *inclusive {
                    result.push('=');
                }
                result.push_str(&self.generate_expression_string(hi));
                result
            }
            Pattern::Or(alts) => alts
                .iter()
                .map(|p| self.generate_pattern_string(p))
                .collect::<Vec<_>>()
                .join(" | "),
            Pattern::Path(segs) => segs
                .iter()
                .map(|s| s.name.clone())
                .collect::<Vec<_>>()
                .join("::"),
            Pattern::Raw(s) => s.clone(),
        }
    }

    /// Generate a literal value as string
    fn generate_literal_string(&self, lit: &Literal) -> String {
        match lit {
            Literal::Int(n) => n.to_string(),
            Literal::Float(f) => {
                // Rust's f64::to_string() drops the fraction for a whole-valued
                // float ("2.0" -> "2"), which rustc would then infer as an
                // integer in a float context. Keep it a float literal: when the
                // rendered form carries no decimal point or exponent (and is a
                // finite number), append ".0".
                let s = f.to_string();
                if s.contains('.') || s.contains('e') || s.contains('E') || !f.is_finite() {
                    s
                } else {
                    format!("{s}.0")
                }
            }
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

    /// Emit one operand of a value binary expression, parenthesising it only
    /// when the parent operator's precedence/associativity requires it (CG-1).
    /// `is_right` is true for the right-hand operand of the parent.
    ///
    /// Value binary operators in Crusty/Rust are left-associative, so:
    /// - a child binary binding looser than the parent always needs parens;
    /// - an equal-precedence child needs parens only on the RIGHT (left-assoc);
    /// - comparisons are non-associative, so a comparison child directly under
    ///   a comparison parent needs parens on either side.
    ///
    /// Assignment children never occur here (handled on the unparenthesised
    /// path), so their precedence 0 only ever forces parens, which is correct.
    fn emit_binary_operand(&self, operand: &Expression, parent: &BinaryOp, is_right: bool) -> String {
        let inner = self.generate_expression_string(operand);
        if let Expression::Binary { op: child, .. } = operand {
            if child.is_assignment() {
                // An assignment nested in a value position is unusual, but if it
                // appears it must be grouped to read as a value.
                return format!("({inner})");
            }
            let needs = child.precedence() < parent.precedence()
                || (child.precedence() == parent.precedence() && is_right)
                || (parent.is_comparison() && child.is_comparison());
            if needs {
                return format!("({inner})");
            }
        }
        inner
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

    /// Render a generic-parameter list as `<...>`, or `""` when there are
    /// none. Bounds and const-generic types are carried verbatim (ADR-0011
    /// Phase 2): Crusty does not model trait bounds, it hands them to rustc.
    pub fn generate_generic_params_string(&self, params: &[GenericParam]) -> String {
        if params.is_empty() {
            return String::new();
        }
        let rendered: Vec<String> = params
            .iter()
            .map(|param| match param {
                GenericParam::Lifetime { name } => format!("'{}", name.name),
                GenericParam::Const { name, ty } => {
                    format!("const {}: {}", name.name, self.generate_type_string(ty))
                }
                GenericParam::Type { name, bounds } => {
                    if bounds.is_empty() {
                        name.name.clone()
                    } else {
                        format!("{}: {}", name.name, bounds.join(" + "))
                    }
                }
            })
            .collect();
        format!("<{}>", rendered.join(", "))
    }

    /// Render the bare generic-ARGUMENT form `<T, 'a, N>` (names only, no
    /// bounds or `const`/type annotations), used where a generic type is
    /// *referenced* rather than declared — e.g. `impl<T> Name<T>`. Returns
    /// `""` when there are no params.
    pub fn generate_generic_args_string(&self, params: &[GenericParam]) -> String {
        if params.is_empty() {
            return String::new();
        }
        let rendered: Vec<String> = params
            .iter()
            .map(|param| match param {
                GenericParam::Lifetime { name } => format!("'{}", name.name),
                GenericParam::Const { name, .. } | GenericParam::Type { name, .. } => {
                    name.name.clone()
                }
            })
            .collect();
        format!("<{}>", rendered.join(", "))
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
            Type::Raw(s) => s.clone(),
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
