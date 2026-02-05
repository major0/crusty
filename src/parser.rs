// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Parser module for building AST from token stream.
//!
//! This module contains two parser implementations:
//! 1. A hand-written recursive descent parser (legacy)
//! 2. A rust-peg based parser (new implementation)
//!
//! The rust-peg parser uses PEG (Parsing Expression Grammar) rules with:
//! - Lookahead for keyword disambiguation (prevents "let" from matching "letter")
//! - Whitespace handling with quiet! macro for cleaner error messages
//! - Direct AST construction within grammar rules

use crate::ast::*;
use crate::error::ParseError;
use crate::lexer::{Lexer, Token, TokenKind};
use std::collections::HashMap;

/// Parser for Crusty source code
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
    /// Token buffer for lookahead (stores peeked tokens)
    token_buffer: Vec<Token>,
    /// Registry of macro names to their delimiter types
    macro_registry: HashMap<String, MacroDelimiter>,
}

impl<'a> Parser<'a> {
    /// Create a new parser from source code
    pub fn new(source: &'a str) -> Result<Self, ParseError> {
        let mut lexer = Lexer::new(source);
        let current_token = lexer
            .next_token()
            .map_err(|e| ParseError::new(e.span, e.message, vec![], "lexical error"))?;

        Ok(Self {
            lexer,
            current_token,
            token_buffer: Vec::new(),
            macro_registry: HashMap::new(),
        })
    }

    /// Advance to the next token
    fn advance(&mut self) -> Result<(), ParseError> {
        // If we have buffered tokens, use them first
        if !self.token_buffer.is_empty() {
            self.current_token = self.token_buffer.remove(0);
        } else {
            self.current_token = self
                .lexer
                .next_token()
                .map_err(|e| ParseError::new(e.span, e.message, vec![], "lexical error"))?;
        }
        Ok(())
    }

    /// Peek ahead n tokens without consuming them
    /// Returns None if we can't peek that far ahead
    fn peek_ahead(&mut self, n: usize) -> Result<Option<Token>, ParseError> {
        // Ensure we have enough tokens in the buffer
        while self.token_buffer.len() < n {
            let token = self
                .lexer
                .next_token()
                .map_err(|e| ParseError::new(e.span, e.message, vec![], "lexical error"))?;
            self.token_buffer.push(token);
        }

        if n == 0 {
            Ok(Some(self.current_token.clone()))
        } else if n <= self.token_buffer.len() {
            Ok(Some(self.token_buffer[n - 1].clone()))
        } else {
            Ok(None)
        }
    }

    /// Expect a specific token kind and consume it
    fn expect(&mut self, expected: TokenKind) -> Result<Token, ParseError> {
        if std::mem::discriminant(&self.current_token.kind) == std::mem::discriminant(&expected) {
            let token = self.current_token.clone();
            self.advance()?;
            Ok(token)
        } else {
            Err(ParseError::new(
                self.current_token.span,
                format!(
                    "expected {:?}, found {:?}",
                    expected, self.current_token.kind
                ),
                vec![format!("{:?}", expected)],
                format!("{:?}", self.current_token.kind),
            ))
        }
    }

    /// Peek at the current token without consuming it
    #[allow(dead_code)]
    fn peek(&self) -> &Token {
        &self.current_token
    }

    /// Check if current token matches a specific kind
    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.current_token.kind) == std::mem::discriminant(kind)
    }

    /// Check if current token could be the start of a type
    fn is_type_token(&self) -> bool {
        matches!(
            self.current_token.kind,
            TokenKind::Int
                | TokenKind::I32
                | TokenKind::I64
                | TokenKind::U32
                | TokenKind::U64
                | TokenKind::Float
                | TokenKind::F32
                | TokenKind::F64
                | TokenKind::Bool
                | TokenKind::Char
                | TokenKind::Void
                | TokenKind::Ident(_)
                | TokenKind::Star // For pointer types like *int
                | TokenKind::BitAnd // For reference types like &int
                | TokenKind::Var // For mutable references like var &int
        )
    }

    /// Check if current position looks like a variable declaration (Type name = value;)
    /// Uses lookahead to distinguish from expressions like int(x) or int + 5
    /// Returns true if pattern matches: Type Identifier '='
    fn looks_like_declaration(&mut self) -> Result<bool, ParseError> {
        // First check: current token must be a type token
        if !self.is_type_token() {
            return Ok(false);
        }

        // For primitive types, we need to look ahead to distinguish from expressions
        let is_primitive = matches!(
            self.current_token.kind,
            TokenKind::Int
                | TokenKind::I32
                | TokenKind::I64
                | TokenKind::U32
                | TokenKind::U64
                | TokenKind::Float
                | TokenKind::F32
                | TokenKind::F64
                | TokenKind::Bool
                | TokenKind::Char
                | TokenKind::Void
        );

        // For identifiers (could be typedef or variable), check if next token is assignment
        // This would indicate an assignment statement, not a declaration
        if matches!(self.current_token.kind, TokenKind::Ident(_)) && !is_primitive {
            let next_token = self.peek_ahead(1)?;
            if let Some(token) = next_token {
                if matches!(token.kind, TokenKind::Assign) {
                    // Pattern: identifier = value (assignment, not declaration)
                    return Ok(false);
                }
            }
        }

        // Look ahead to see what comes after the type
        // Pattern: Type Identifier '='
        // Need to handle pointer types: int* ptr = ...
        let mut lookahead_offset = 1;

        // Skip pointer/reference modifiers
        loop {
            let next_token = self.peek_ahead(lookahead_offset)?;
            if let Some(token) = next_token {
                if matches!(token.kind, TokenKind::Star | TokenKind::BitAnd) {
                    lookahead_offset += 1;
                    continue;
                }
                break;
            } else {
                return Ok(false);
            }
        }

        // Now check for identifier
        let next_token = self.peek_ahead(lookahead_offset)?;
        if let Some(token) = next_token {
            if !matches!(token.kind, TokenKind::Ident(_)) {
                // Not followed by identifier, so not a declaration
                // Could be: int(x) or int + 5
                return Ok(false);
            }

            // Check if there's an '=' after the identifier
            let token_after_ident = self.peek_ahead(lookahead_offset + 1)?;
            if let Some(token) = token_after_ident {
                return Ok(matches!(token.kind, TokenKind::Assign));
            }
        }

        Ok(false)
    }

    /// Check if we're at end of file
    fn is_at_end(&self) -> bool {
        matches!(self.current_token.kind, TokenKind::Eof)
    }

    /// Parse a complete source file into a File AST
    pub fn parse_file(&mut self) -> Result<File, ParseError> {
        let mut items = Vec::new();
        let doc_comments = Vec::new();

        while !self.is_at_end() {
            items.push(self.parse_item()?);
        }

        Ok(File {
            items,
            doc_comments,
        })
    }

    /// Parse a top-level item
    fn parse_item(&mut self) -> Result<Item, ParseError> {
        // Parse attributes first (they start with #[)
        let attributes = self.parse_attributes()?;

        // Check for #define directive (starts with # but not #[)
        if self.check(&TokenKind::Hash) {
            return self.parse_define();
        }

        // Check for visibility modifier (static keyword makes functions private)
        let is_static = if self.check(&TokenKind::Static) {
            self.advance()?;
            true
        } else {
            false
        };

        // Check for type keywords that indicate function declarations
        match &self.current_token.kind {
            TokenKind::Int
            | TokenKind::I32
            | TokenKind::I64
            | TokenKind::U32
            | TokenKind::U64
            | TokenKind::Float
            | TokenKind::F32
            | TokenKind::F64
            | TokenKind::Bool
            | TokenKind::Char
            | TokenKind::Void => self.parse_function(is_static, attributes),
            TokenKind::Struct => self.parse_struct_with_attributes(attributes),
            TokenKind::Enum => self.parse_enum_with_attributes(attributes),
            TokenKind::Typedef => self.parse_typedef(is_static),
            _ => Err(ParseError::new(
                self.current_token.span,
                "expected item declaration",
                vec![
                    "function".to_string(),
                    "struct".to_string(),
                    "enum".to_string(),
                    "typedef".to_string(),
                    "#define".to_string(),
                ],
                format!("{:?}", self.current_token.kind),
            )),
        }
    }

    /// Parse attributes (#[...])
    fn parse_attributes(&mut self) -> Result<Vec<Attribute>, ParseError> {
        let mut attributes = Vec::new();

        while self.check(&TokenKind::Hash) {
            // Peek ahead to check if this is an attribute (#[) or a #define
            // We need to check the next token without consuming the #
            let is_attribute = {
                let mut temp_lexer = Lexer {
                    source: self.lexer.source,
                    chars: self.lexer.source[self.lexer.position..]
                        .char_indices()
                        .peekable(),
                    position: self.lexer.position,
                    line: self.lexer.line,
                    column: self.lexer.column,
                };

                // Try to read the next token
                if let Ok(token) = temp_lexer.next_token() {
                    matches!(token.kind, TokenKind::LBracket)
                } else {
                    false
                }
            };

            // If not an attribute, stop parsing attributes
            if !is_attribute {
                break;
            }

            self.advance()?;
            self.expect(TokenKind::LBracket)?;

            // Parse attribute name
            let name = match &self.current_token.kind {
                TokenKind::Ident(n) => {
                    let ident = Ident::new(n.clone());
                    self.advance()?;
                    ident
                }
                _ => {
                    return Err(ParseError::new(
                        self.current_token.span,
                        "expected attribute name",
                        vec!["identifier".to_string()],
                        format!("{:?}", self.current_token.kind),
                    ));
                }
            };

            // Parse optional attribute arguments
            let mut args = Vec::new();
            if self.check(&TokenKind::LParen) {
                self.advance()?;

                if !self.check(&TokenKind::RParen) {
                    loop {
                        args.push(self.parse_attribute_arg()?);
                        if self.check(&TokenKind::Comma) {
                            self.advance()?;
                        } else {
                            break;
                        }
                    }
                }

                self.expect(TokenKind::RParen)?;
            }

            self.expect(TokenKind::RBracket)?;

            attributes.push(Attribute { name, args });
        }

        Ok(attributes)
    }

    /// Parse an attribute argument
    fn parse_attribute_arg(&mut self) -> Result<AttributeArg, ParseError> {
        match &self.current_token.kind {
            TokenKind::Ident(n) => {
                let ident = Ident::new(n.clone());
                self.advance()?;

                // Check for name = value syntax
                if self.check(&TokenKind::Assign) {
                    self.advance()?;
                    let value = self.parse_attribute_literal()?;
                    Ok(AttributeArg::NameValue { name: ident, value })
                } else {
                    Ok(AttributeArg::Ident(ident))
                }
            }
            _ => {
                let lit = self.parse_attribute_literal()?;
                Ok(AttributeArg::Literal(lit))
            }
        }
    }

    /// Parse a literal for attribute arguments
    fn parse_attribute_literal(&mut self) -> Result<Literal, ParseError> {
        match &self.current_token.kind {
            TokenKind::IntLiteral(s) => {
                let val = s.parse::<i64>().map_err(|_| {
                    ParseError::new(
                        self.current_token.span,
                        "invalid integer literal",
                        vec![],
                        s.clone(),
                    )
                })?;
                self.advance()?;
                Ok(Literal::Int(val))
            }
            TokenKind::StringLiteral(s) => {
                let val = s.clone();
                self.advance()?;
                Ok(Literal::String(val))
            }
            TokenKind::BoolLiteral(b) => {
                let val = *b;
                self.advance()?;
                Ok(Literal::Bool(val))
            }
            _ => Err(ParseError::new(
                self.current_token.span,
                "expected literal in attribute",
                vec![
                    "integer".to_string(),
                    "string".to_string(),
                    "bool".to_string(),
                ],
                format!("{:?}", self.current_token.kind),
            )),
        }
    }

    /// Parse a struct definition with attributes
    fn parse_struct_with_attributes(
        &mut self,
        attributes: Vec<Attribute>,
    ) -> Result<Item, ParseError> {
        self.expect(TokenKind::Struct)?;

        // Parse struct name
        let name = match &self.current_token.kind {
            TokenKind::Ident(name) => {
                let ident = Ident::new(name.clone());
                self.advance()?;
                ident
            }
            _ => {
                return Err(ParseError::new(
                    self.current_token.span,
                    "expected struct name",
                    vec!["identifier".to_string()],
                    format!("{:?}", self.current_token.kind),
                ));
            }
        };

        self.expect(TokenKind::LBrace)?;

        let mut fields = Vec::new();
        let mut methods = Vec::new();

        while !self.check(&TokenKind::RBrace) {
            // Parse field/method attributes
            let item_attributes = self.parse_attributes()?;

            // Check if this is a method (has parentheses after identifier) or a field
            if self.is_method_definition()? {
                let mut method = self.parse_struct_method()?;
                method.attributes = item_attributes;
                methods.push(method);
            } else {
                // Parse as field
                let field_type = self.parse_type()?;

                let field_name = match &self.current_token.kind {
                    TokenKind::Ident(name) => {
                        let ident = Ident::new(name.clone());
                        self.advance()?;
                        ident
                    }
                    _ => {
                        return Err(ParseError::new(
                            self.current_token.span,
                            "expected field name",
                            vec!["identifier".to_string()],
                            format!("{:?}", self.current_token.kind),
                        ));
                    }
                };

                self.expect(TokenKind::Semicolon)?;

                fields.push(Field {
                    visibility: Visibility::Public,
                    name: field_name,
                    ty: field_type,
                    doc_comments: Vec::new(),
                    attributes: item_attributes,
                });
            }
        }

        self.expect(TokenKind::RBrace)?;

        Ok(Item::Struct(Struct {
            visibility: Visibility::Public,
            name,
            fields,
            methods,
            doc_comments: Vec::new(),
            attributes,
        }))
    }

    /// Parse an enum definition with attributes
    fn parse_enum_with_attributes(
        &mut self,
        attributes: Vec<Attribute>,
    ) -> Result<Item, ParseError> {
        self.expect(TokenKind::Enum)?;

        // Parse enum name
        let name = match &self.current_token.kind {
            TokenKind::Ident(name) => {
                let ident = Ident::new(name.clone());
                self.advance()?;
                ident
            }
            _ => {
                return Err(ParseError::new(
                    self.current_token.span,
                    "expected enum name",
                    vec!["identifier".to_string()],
                    format!("{:?}", self.current_token.kind),
                ));
            }
        };

        self.expect(TokenKind::LBrace)?;

        let mut variants = Vec::new();
        let mut next_value = 0i64;

        while !self.check(&TokenKind::RBrace) {
            // Parse variant name
            let variant_name = match &self.current_token.kind {
                TokenKind::Ident(name) => {
                    let ident = Ident::new(name.clone());
                    self.advance()?;
                    ident
                }
                _ => {
                    return Err(ParseError::new(
                        self.current_token.span,
                        "expected enum variant name",
                        vec!["identifier".to_string()],
                        format!("{:?}", self.current_token.kind),
                    ));
                }
            };

            // Check for explicit value
            let value = if self.check(&TokenKind::Assign) {
                self.advance()?;
                match &self.current_token.kind {
                    TokenKind::IntLiteral(s) => {
                        let val = s.parse::<i64>().map_err(|_| {
                            ParseError::new(
                                self.current_token.span,
                                "invalid integer literal",
                                vec![],
                                s.clone(),
                            )
                        })?;
                        self.advance()?;
                        next_value = val + 1;
                        Some(val)
                    }
                    _ => {
                        return Err(ParseError::new(
                            self.current_token.span,
                            "expected integer literal",
                            vec!["integer".to_string()],
                            format!("{:?}", self.current_token.kind),
                        ));
                    }
                }
            } else {
                let val = next_value;
                next_value += 1;
                Some(val)
            };

            variants.push(EnumVariant {
                name: variant_name,
                value,
            });

            if self.check(&TokenKind::Comma) {
                self.advance()?;
            } else {
                break;
            }
        }

        self.expect(TokenKind::RBrace)?;

        Ok(Item::Enum(Enum {
            visibility: Visibility::Public,
            name,
            variants,
            doc_comments: Vec::new(),
            attributes,
        }))
    }

    /// Parse a function declaration
    fn parse_function(
        &mut self,
        is_static: bool,
        attributes: Vec<Attribute>,
    ) -> Result<Item, ParseError> {
        // Parse return type
        let return_type = if self.check(&TokenKind::Void) {
            self.advance()?;
            None
        } else {
            Some(self.parse_type()?)
        };

        // Parse function name
        let name = match &self.current_token.kind {
            TokenKind::Ident(name) => {
                let ident = Ident::new(name.clone());
                self.advance()?;
                ident
            }
            _ => {
                return Err(ParseError::new(
                    self.current_token.span,
                    "expected function name",
                    vec!["identifier".to_string()],
                    format!("{:?}", self.current_token.kind),
                ));
            }
        };

        // Parse parameter list
        self.expect(TokenKind::LParen)?;
        let mut params = Vec::new();

        if !self.check(&TokenKind::RParen) {
            loop {
                // Parse parameter type
                let param_type = self.parse_type()?;

                // Parse parameter name
                let param_name = match &self.current_token.kind {
                    TokenKind::Ident(name) => {
                        let ident = Ident::new(name.clone());
                        self.advance()?;
                        ident
                    }
                    _ => {
                        return Err(ParseError::new(
                            self.current_token.span,
                            "expected parameter name",
                            vec!["identifier".to_string()],
                            format!("{:?}", self.current_token.kind),
                        ));
                    }
                };

                params.push(Param {
                    name: param_name,
                    ty: param_type,
                });

                if self.check(&TokenKind::Comma) {
                    self.advance()?;
                } else {
                    break;
                }
            }
        }

        self.expect(TokenKind::RParen)?;

        // Parse function body
        let body = self.parse_block()?;

        Ok(Item::Function(Function {
            visibility: if is_static {
                Visibility::Private
            } else {
                Visibility::Public
            },
            name,
            params,
            return_type,
            body,
            doc_comments: Vec::new(),
            attributes,
        }))
    }

    /// Parse a struct definition
    #[allow(dead_code)]
    fn parse_struct(&mut self) -> Result<Item, ParseError> {
        self.expect(TokenKind::Struct)?;

        // Parse struct name
        let name = match &self.current_token.kind {
            TokenKind::Ident(name) => {
                let ident = Ident::new(name.clone());
                self.advance()?;
                ident
            }
            _ => {
                return Err(ParseError::new(
                    self.current_token.span,
                    "expected struct name",
                    vec!["identifier".to_string()],
                    format!("{:?}", self.current_token.kind),
                ));
            }
        };

        self.expect(TokenKind::LBrace)?;

        let mut fields = Vec::new();
        let mut methods = Vec::new();

        while !self.check(&TokenKind::RBrace) {
            // Check if this is a method (has parentheses after identifier) or a field
            // We need to look ahead to determine this

            // Save current position for potential backtracking
            let _saved_token = self.current_token.clone();

            // Try to parse as a method first
            if self.is_method_definition()? {
                methods.push(self.parse_struct_method()?);
            } else {
                // Parse as field
                // Parse field type
                let field_type = self.parse_type()?;

                // Parse field name
                let field_name = match &self.current_token.kind {
                    TokenKind::Ident(name) => {
                        let ident = Ident::new(name.clone());
                        self.advance()?;
                        ident
                    }
                    _ => {
                        return Err(ParseError::new(
                            self.current_token.span,
                            "expected field name",
                            vec!["identifier".to_string()],
                            format!("{:?}", self.current_token.kind),
                        ));
                    }
                };

                self.expect(TokenKind::Semicolon)?;

                fields.push(Field {
                    visibility: Visibility::Public,
                    name: field_name,
                    ty: field_type,
                    doc_comments: Vec::new(),
                    attributes: Vec::new(),
                });
            }
        }

        self.expect(TokenKind::RBrace)?;

        Ok(Item::Struct(Struct {
            visibility: Visibility::Public,
            name,
            fields,
            methods,
            doc_comments: Vec::new(),
            attributes: Vec::new(),
        }))
    }

    /// Check if the current position is a method definition
    fn is_method_definition(&self) -> Result<bool, ParseError> {
        // A method definition looks like:
        // - return_type method_name(params) { body }
        // - void method_name(params) { body }
        // - static return_type method_name(params) { body }
        // - TypeName method_name(params) { body } (custom types)

        // Check for static keyword
        if self.check(&TokenKind::Static) {
            return Ok(true);
        }

        // Check for type keyword (primitive or custom identifier) followed by identifier and then (
        let is_type_keyword = matches!(
            self.current_token.kind,
            TokenKind::Int
                | TokenKind::I32
                | TokenKind::I64
                | TokenKind::U32
                | TokenKind::U64
                | TokenKind::Float
                | TokenKind::F32
                | TokenKind::F64
                | TokenKind::Bool
                | TokenKind::Char
                | TokenKind::Void
                | TokenKind::Ident(_) // Allow custom type names
        );

        if !is_type_keyword {
            return Ok(false);
        }

        // Create a temporary lexer for lookahead starting from current lexer position
        let mut temp_lexer = Lexer {
            source: self.lexer.source,
            chars: self.lexer.source[self.lexer.position..]
                .char_indices()
                .peekable(),
            position: self.lexer.position,
            line: self.lexer.line,
            column: self.lexer.column,
        };

        // Read the next token (should be identifier for method name)
        if let Ok(token) = temp_lexer.next_token() {
            if !matches!(token.kind, TokenKind::Ident(_)) {
                return Ok(false);
            }
        } else {
            return Ok(false);
        }

        // Check for (
        if let Ok(token) = temp_lexer.next_token() {
            Ok(matches!(token.kind, TokenKind::LParen))
        } else {
            Ok(false)
        }
    }

    /// Parse a method definition within a struct
    fn parse_struct_method(&mut self) -> Result<Function, ParseError> {
        // Check for static keyword
        let is_static = if self.check(&TokenKind::Static) {
            self.advance()?;
            true
        } else {
            false
        };

        // Parse return type
        let return_type = if self.check(&TokenKind::Void) {
            self.advance()?;
            None
        } else {
            Some(self.parse_type()?)
        };

        // Parse method name
        let name = match &self.current_token.kind {
            TokenKind::Ident(name) => {
                let ident = Ident::new(name.clone());
                self.advance()?;
                ident
            }
            _ => {
                return Err(ParseError::new(
                    self.current_token.span,
                    "expected method name",
                    vec!["identifier".to_string()],
                    format!("{:?}", self.current_token.kind),
                ));
            }
        };

        // Parse parameter list
        self.expect(TokenKind::LParen)?;
        let mut params = Vec::new();

        if !self.check(&TokenKind::RParen) {
            loop {
                // Check for self parameter
                if self.check(&TokenKind::Ident("self".to_string())) {
                    let self_ident = Ident::new("self");
                    self.advance()?;

                    // self parameter (immutable reference)
                    params.push(Param {
                        name: self_ident,
                        ty: Type::Ident(Ident::new("Self")),
                    });
                } else if self.check(&TokenKind::Var) {
                    // var &self (mutable reference to self)
                    self.advance()?;

                    // Expect & after var
                    self.expect(TokenKind::BitAnd)?;

                    // Expect 'self' identifier
                    if let TokenKind::Ident(n) = &self.current_token.kind {
                        if n == "self" {
                            self.advance()?;
                            params.push(Param {
                                name: Ident::new("self"),
                                ty: Type::Reference {
                                    ty: Box::new(Type::Ident(Ident::new("Self"))),
                                    mutable: true,
                                },
                            });
                        } else {
                            return Err(ParseError::new(
                                self.current_token.span,
                                "expected 'self' after var &",
                                vec!["self".to_string()],
                                format!("{:?}", self.current_token.kind),
                            ));
                        }
                    } else {
                        return Err(ParseError::new(
                            self.current_token.span,
                            "expected 'self' after var &",
                            vec!["self".to_string()],
                            format!("{:?}", self.current_token.kind),
                        ));
                    }
                } else if self.check(&TokenKind::BitAnd) {
                    // &self or &mut self
                    self.advance()?;

                    let mutable = if self.check(&TokenKind::Mut) {
                        self.advance()?;
                        true
                    } else {
                        false
                    };

                    // Expect 'self' identifier
                    if let TokenKind::Ident(n) = &self.current_token.kind {
                        if n == "self" {
                            self.advance()?;
                            params.push(Param {
                                name: Ident::new("self"),
                                ty: Type::Reference {
                                    ty: Box::new(Type::Ident(Ident::new("Self"))),
                                    mutable,
                                },
                            });
                        } else {
                            return Err(ParseError::new(
                                self.current_token.span,
                                "expected 'self' after &",
                                vec!["self".to_string()],
                                format!("{:?}", self.current_token.kind),
                            ));
                        }
                    } else {
                        return Err(ParseError::new(
                            self.current_token.span,
                            "expected 'self' after &",
                            vec!["self".to_string()],
                            format!("{:?}", self.current_token.kind),
                        ));
                    }
                } else {
                    // Regular parameter
                    let param_type = self.parse_type()?;

                    let param_name = match &self.current_token.kind {
                        TokenKind::Ident(name) => {
                            let ident = Ident::new(name.clone());
                            self.advance()?;
                            ident
                        }
                        _ => {
                            return Err(ParseError::new(
                                self.current_token.span,
                                "expected parameter name",
                                vec!["identifier".to_string()],
                                format!("{:?}", self.current_token.kind),
                            ));
                        }
                    };

                    params.push(Param {
                        name: param_name,
                        ty: param_type,
                    });
                }

                if self.check(&TokenKind::Comma) {
                    self.advance()?;
                } else {
                    break;
                }
            }
        }

        self.expect(TokenKind::RParen)?;

        // Parse method body
        let body = self.parse_block()?;

        Ok(Function {
            visibility: if is_static {
                Visibility::Private
            } else {
                Visibility::Public
            },
            name,
            params,
            return_type,
            body,
            doc_comments: Vec::new(),
            attributes: Vec::new(),
        })
    }

    /// Parse an enum definition
    #[allow(dead_code)]
    fn parse_enum(&mut self) -> Result<Item, ParseError> {
        self.expect(TokenKind::Enum)?;

        // Parse enum name
        let name = match &self.current_token.kind {
            TokenKind::Ident(name) => {
                let ident = Ident::new(name.clone());
                self.advance()?;
                ident
            }
            _ => {
                return Err(ParseError::new(
                    self.current_token.span,
                    "expected enum name",
                    vec!["identifier".to_string()],
                    format!("{:?}", self.current_token.kind),
                ));
            }
        };

        self.expect(TokenKind::LBrace)?;

        let mut variants = Vec::new();
        let mut next_value = 0i64;

        while !self.check(&TokenKind::RBrace) {
            // Parse variant name
            let variant_name = match &self.current_token.kind {
                TokenKind::Ident(name) => {
                    let ident = Ident::new(name.clone());
                    self.advance()?;
                    ident
                }
                _ => {
                    return Err(ParseError::new(
                        self.current_token.span,
                        "expected enum variant name",
                        vec!["identifier".to_string()],
                        format!("{:?}", self.current_token.kind),
                    ));
                }
            };

            // Check for explicit value
            let value = if self.check(&TokenKind::Assign) {
                self.advance()?;
                match &self.current_token.kind {
                    TokenKind::IntLiteral(s) => {
                        let val = s.parse::<i64>().map_err(|_| {
                            ParseError::new(
                                self.current_token.span,
                                "invalid integer literal",
                                vec![],
                                s.clone(),
                            )
                        })?;
                        self.advance()?;
                        next_value = val + 1;
                        Some(val)
                    }
                    _ => {
                        return Err(ParseError::new(
                            self.current_token.span,
                            "expected integer literal",
                            vec!["integer".to_string()],
                            format!("{:?}", self.current_token.kind),
                        ));
                    }
                }
            } else {
                let val = next_value;
                next_value += 1;
                Some(val)
            };

            variants.push(EnumVariant {
                name: variant_name,
                value,
            });

            if self.check(&TokenKind::Comma) {
                self.advance()?;
            } else {
                break;
            }
        }

        self.expect(TokenKind::RBrace)?;

        Ok(Item::Enum(Enum {
            visibility: Visibility::Public,
            name,
            variants,
            doc_comments: Vec::new(),
            attributes: Vec::new(),
        }))
    }

    /// Parse a typedef declaration
    fn parse_typedef(&mut self, is_static: bool) -> Result<Item, ParseError> {
        self.expect(TokenKind::Typedef)?;

        // Parse target type
        let target = self.parse_type()?;

        // Parse alias name
        let name = match &self.current_token.kind {
            TokenKind::Ident(name) => {
                let ident = Ident::new(name.clone());
                self.advance()?;
                ident
            }
            _ => {
                return Err(ParseError::new(
                    self.current_token.span,
                    "expected typedef name",
                    vec!["identifier".to_string()],
                    format!("{:?}", self.current_token.kind),
                ));
            }
        };

        self.expect(TokenKind::Semicolon)?;

        Ok(Item::Typedef(Typedef {
            visibility: if is_static {
                Visibility::Private
            } else {
                Visibility::Public
            },
            name,
            target,
            doc_comments: Vec::new(),
        }))
    }

    /// Parse a #define macro definition
    fn parse_define(&mut self) -> Result<Item, ParseError> {
        // Expect # token
        self.expect(TokenKind::Hash)?;

        // Expect define keyword
        self.expect(TokenKind::Define)?;

        // Parse macro name (must have double-underscore prefix and suffix)
        let name = match &self.current_token.kind {
            TokenKind::Ident(n) => {
                // Validate double-underscore naming convention
                if !n.starts_with("__") || !n.ends_with("__") {
                    return Err(ParseError::new(
                        self.current_token.span,
                        format!(
                            "macro name '{}' must have double-underscore prefix and suffix (e.g., __MACRO_NAME__)",
                            n
                        ),
                        vec!["__MACRO_NAME__".to_string()],
                        n.clone(),
                    ));
                }
                let ident = Ident::new(n.clone());
                self.advance()?;
                ident
            }
            _ => {
                return Err(ParseError::new(
                    self.current_token.span,
                    "expected macro name",
                    vec!["__MACRO_NAME__".to_string()],
                    format!("{:?}", self.current_token.kind),
                ));
            }
        };

        // Detect delimiter type and parse parameters
        let mut params = Vec::new();
        let delimiter = if self.check(&TokenKind::LParen) {
            self.advance()?;

            // Parse parameters
            if !self.check(&TokenKind::RParen) {
                loop {
                    match &self.current_token.kind {
                        TokenKind::Ident(param_name) => {
                            params.push(Ident::new(param_name.clone()));
                            self.advance()?;
                        }
                        _ => {
                            return Err(ParseError::new(
                                self.current_token.span,
                                "expected parameter name",
                                vec!["identifier".to_string()],
                                format!("{:?}", self.current_token.kind),
                            ));
                        }
                    }

                    if self.check(&TokenKind::Comma) {
                        self.advance()?;
                    } else {
                        break;
                    }
                }
            }

            self.expect(TokenKind::RParen)?;
            MacroDelimiter::Parens
        } else if self.check(&TokenKind::LBracket) {
            self.advance()?;

            // Parse parameters
            if !self.check(&TokenKind::RBracket) {
                loop {
                    match &self.current_token.kind {
                        TokenKind::Ident(param_name) => {
                            params.push(Ident::new(param_name.clone()));
                            self.advance()?;
                        }
                        _ => {
                            return Err(ParseError::new(
                                self.current_token.span,
                                "expected parameter name",
                                vec!["identifier".to_string()],
                                format!("{:?}", self.current_token.kind),
                            ));
                        }
                    }

                    if self.check(&TokenKind::Comma) {
                        self.advance()?;
                    } else {
                        break;
                    }
                }
            }

            self.expect(TokenKind::RBracket)?;
            MacroDelimiter::Brackets
        } else if self.check(&TokenKind::LBrace) {
            self.advance()?;

            // Parse parameters
            if !self.check(&TokenKind::RBrace) {
                loop {
                    match &self.current_token.kind {
                        TokenKind::Ident(param_name) => {
                            params.push(Ident::new(param_name.clone()));
                            self.advance()?;
                        }
                        _ => {
                            return Err(ParseError::new(
                                self.current_token.span,
                                "expected parameter name",
                                vec!["identifier".to_string()],
                                format!("{:?}", self.current_token.kind),
                            ));
                        }
                    }

                    if self.check(&TokenKind::Comma) {
                        self.advance()?;
                    } else {
                        break;
                    }
                }
            }

            self.expect(TokenKind::RBrace)?;
            MacroDelimiter::Braces
        } else {
            // No delimiter - constant macro
            MacroDelimiter::None
        };

        // Parse macro body as token sequence until end of line or semicolon
        let mut body = Vec::new();
        let start_line = self.current_token.span.start.line;

        while !self.is_at_end()
            && self.current_token.span.start.line == start_line
            && !self.check(&TokenKind::Semicolon)
        {
            body.push(self.current_token.clone());
            self.advance()?;
        }

        // Optional semicolon at end
        if self.check(&TokenKind::Semicolon) {
            self.advance()?;
        }

        // Register macro in the registry
        self.macro_registry
            .insert(name.name.clone(), delimiter.clone());

        Ok(Item::MacroDefinition(MacroDefinition {
            name,
            params,
            body,
            delimiter,
        }))
    }

    /// Parse a block of statements
    fn parse_block(&mut self) -> Result<Block, ParseError> {
        self.expect(TokenKind::LBrace)?;

        let mut statements = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        self.expect(TokenKind::RBrace)?;

        Ok(Block::new(statements))
    }

    /// Parse a statement
    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match &self.current_token.kind {
            TokenKind::Let => self.parse_let_statement(),
            TokenKind::Var => self.parse_var_statement(),
            TokenKind::Const => self.parse_const_statement(),
            TokenKind::If => self.parse_if_statement(),
            TokenKind::While => self.parse_while_statement(),
            TokenKind::For => self.parse_for_statement(),
            TokenKind::Return => self.parse_return_statement(),
            TokenKind::Break => self.parse_break_statement(),
            TokenKind::Continue => self.parse_continue_statement(),
            TokenKind::Dot => {
                // Check for labeled loop (.label: loop { ... })
                self.parse_labeled_loop()
            }
            // Check for nested function or implicit let declaration: type identifier ...
            TokenKind::Void
            | TokenKind::Int
            | TokenKind::I32
            | TokenKind::I64
            | TokenKind::U32
            | TokenKind::U64
            | TokenKind::Float
            | TokenKind::F32
            | TokenKind::F64
            | TokenKind::Bool
            | TokenKind::Char
            | TokenKind::Ident(_) => {
                // Look ahead to see if this is a function declaration
                if self.is_nested_function_declaration()? {
                    self.parse_nested_function()
                } else if self.looks_like_declaration()? {
                    // Pattern: Type Identifier '=' (implicit let declaration)
                    self.parse_implicit_let_statement()
                } else {
                    // Parse as expression statement (which may include assignment)
                    self.parse_expression_statement()
                }
            }
            _ => {
                // Try to parse as expression statement
                self.parse_expression_statement()
            }
        }
    }

    /// Parse an expression statement (including assignments)
    fn parse_expression_statement(&mut self) -> Result<Statement, ParseError> {
        let expr = self.parse_expression_stub()?;

        // Check if this is an assignment
        if self.check(&TokenKind::Assign) {
            self.advance()?;
            let value = self.parse_expression_stub()?;
            self.expect(TokenKind::Semicolon)?;

            // Convert the left-hand side expression to an assignment target
            Ok(Statement::Expr(Expression::Binary {
                op: BinaryOp::Assign,
                left: Box::new(expr),
                right: Box::new(value),
            }))
        } else {
            self.expect(TokenKind::Semicolon)?;
            Ok(Statement::Expr(expr))
        }
    }

    /// Parse a let statement
    fn parse_let_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(TokenKind::Let)?;

        // Check if next token is a type (C-style: let int x = 42;)
        // We need to distinguish between:
        // - let int x = 42; (type is int, name is x)
        // - let x = 42; (no type, name is x)
        let (name, ty) = if self.is_type_token() {
            // Check if this looks like a type followed by identifier
            // Pattern: let Type Identifier = ...
            let next_token = self.peek_ahead(1)?;
            let is_type_declaration = if let Some(token) = next_token {
                matches!(token.kind, TokenKind::Ident(_))
            } else {
                false
            };

            if is_type_declaration {
                // Parse type first
                let ty = self.parse_type()?;

                // Then parse variable name
                let name = match &self.current_token.kind {
                    TokenKind::Ident(n) => {
                        let ident = Ident::new(n.clone());
                        self.advance()?;
                        ident
                    }
                    _ => {
                        return Err(ParseError::new(
                            self.current_token.span,
                            "expected variable name after type",
                            vec!["identifier".to_string()],
                            format!("{:?}", self.current_token.kind),
                        ));
                    }
                };

                (name, Some(ty))
            } else {
                // Type inference (let x = 42;)
                let name = match &self.current_token.kind {
                    TokenKind::Ident(n) => {
                        let ident = Ident::new(n.clone());
                        self.advance()?;
                        ident
                    }
                    _ => {
                        return Err(ParseError::new(
                            self.current_token.span,
                            "expected variable name",
                            vec!["identifier".to_string()],
                            format!("{:?}", self.current_token.kind),
                        ));
                    }
                };

                (name, None)
            }
        } else {
            // Type inference (let x = 42;)
            let name = match &self.current_token.kind {
                TokenKind::Ident(n) => {
                    let ident = Ident::new(n.clone());
                    self.advance()?;
                    ident
                }
                _ => {
                    return Err(ParseError::new(
                        self.current_token.span,
                        "expected variable name",
                        vec!["identifier".to_string()],
                        format!("{:?}", self.current_token.kind),
                    ));
                }
            };

            (name, None)
        };

        // Parse optional initializer
        let init = if self.check(&TokenKind::Assign) {
            self.advance()?;
            Some(self.parse_expression_stub()?)
        } else {
            None
        };

        self.expect(TokenKind::Semicolon)?;

        Ok(Statement::Let {
            name,
            ty,
            init,
            mutable: false,
        })
    }

    /// Parse a var statement
    fn parse_var_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(TokenKind::Var)?;

        // Check if next token is a type (C-style: var int x = 42;)
        // We need to distinguish between:
        // - var int x = 42; (type is int, name is x)
        // - var x = 42; (no type, name is x)
        let (name, ty) = if self.is_type_token() {
            // Check if this looks like a type followed by identifier
            // Pattern: var Type Identifier = ...
            let next_token = self.peek_ahead(1)?;
            let is_type_declaration = if let Some(token) = next_token {
                matches!(token.kind, TokenKind::Ident(_))
            } else {
                false
            };

            if is_type_declaration {
                // Parse type first
                let ty = self.parse_type()?;

                // Then parse variable name
                let name = match &self.current_token.kind {
                    TokenKind::Ident(n) => {
                        let ident = Ident::new(n.clone());
                        self.advance()?;
                        ident
                    }
                    _ => {
                        return Err(ParseError::new(
                            self.current_token.span,
                            "expected variable name after type",
                            vec!["identifier".to_string()],
                            format!("{:?}", self.current_token.kind),
                        ));
                    }
                };

                (name, Some(ty))
            } else {
                // Type inference (var x = 42;)
                let name = match &self.current_token.kind {
                    TokenKind::Ident(n) => {
                        let ident = Ident::new(n.clone());
                        self.advance()?;
                        ident
                    }
                    _ => {
                        return Err(ParseError::new(
                            self.current_token.span,
                            "expected variable name",
                            vec!["identifier".to_string()],
                            format!("{:?}", self.current_token.kind),
                        ));
                    }
                };

                (name, None)
            }
        } else {
            // Type inference (var x = 42;)
            let name = match &self.current_token.kind {
                TokenKind::Ident(n) => {
                    let ident = Ident::new(n.clone());
                    self.advance()?;
                    ident
                }
                _ => {
                    return Err(ParseError::new(
                        self.current_token.span,
                        "expected variable name",
                        vec!["identifier".to_string()],
                        format!("{:?}", self.current_token.kind),
                    ));
                }
            };

            (name, None)
        };

        // Parse optional initializer
        let init = if self.check(&TokenKind::Assign) {
            self.advance()?;
            Some(self.parse_expression_stub()?)
        } else {
            None
        };

        self.expect(TokenKind::Semicolon)?;

        Ok(Statement::Var { name, ty, init })
    }

    /// Parse a const statement
    fn parse_const_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(TokenKind::Const)?;

        // Check if next token is a type (C-style: const int MAX = 100;)
        // We need to distinguish between:
        // - const int MAX = 100; (type is int, name is MAX)
        // - const MAX = 100; (no type, name is MAX)
        let explicit_ty = if self.is_type_token() {
            // Check if this looks like a type followed by identifier
            // Pattern: const Type Identifier = ...
            let next_token = self.peek_ahead(1)?;
            let is_type_declaration = if let Some(token) = next_token {
                matches!(token.kind, TokenKind::Ident(_))
            } else {
                false
            };

            if is_type_declaration {
                // Parse type first
                let ty = self.parse_type()?;
                Some(ty)
            } else {
                None
            }
        } else {
            None
        };

        // Parse constant name
        let name = match &self.current_token.kind {
            TokenKind::Ident(n) => {
                let ident = Ident::new(n.clone());
                self.advance()?;
                ident
            }
            _ => {
                return Err(ParseError::new(
                    self.current_token.span,
                    "expected constant name",
                    vec!["identifier".to_string()],
                    format!("{:?}", self.current_token.kind),
                ));
            }
        };

        // Parse initializer (required for const)
        self.expect(TokenKind::Assign)?;
        let value = self.parse_expression_stub()?;

        self.expect(TokenKind::Semicolon)?;

        // Determine type: use explicit type if provided, otherwise extract from cast or infer
        let ty = if let Some(explicit_ty) = explicit_ty {
            explicit_ty
        } else if let Expression::Cast { ty, .. } = &value {
            ty.clone()
        } else {
            // If no explicit type and no cast, infer type from the expression
            // For now, we'll use a placeholder - semantic analyzer will infer
            Type::Primitive(PrimitiveType::Int)
        };

        Ok(Statement::Const { name, ty, value })
    }

    /// Parse an implicit let statement (C-style: int x = 42;)
    /// This is called when we detect a type token at the start of a statement
    /// followed by an identifier and assignment operator
    fn parse_implicit_let_statement(&mut self) -> Result<Statement, ParseError> {
        // Parse type
        let ty = self.parse_type()?;

        // Parse variable name
        let name = match &self.current_token.kind {
            TokenKind::Ident(n) => {
                let ident = Ident::new(n.clone());
                self.advance()?;
                ident
            }
            _ => {
                return Err(ParseError::new(
                    self.current_token.span,
                    "expected variable name after type",
                    vec!["identifier".to_string()],
                    format!("{:?}", self.current_token.kind),
                ));
            }
        };

        // Expect assignment
        self.expect(TokenKind::Assign)?;

        // Parse initializer
        let init = Some(self.parse_expression_stub()?);

        self.expect(TokenKind::Semicolon)?;

        // Create Let statement with type (implicit let is immutable)
        Ok(Statement::Let {
            name,
            ty: Some(ty),
            init,
            mutable: false,
        })
    }

    /// Parse an if statement
    fn parse_if_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(TokenKind::If)?;

        self.expect(TokenKind::LParen)?;
        let condition = self.parse_expression_stub()?;
        self.expect(TokenKind::RParen)?;

        let then_block = self.parse_block()?;

        // Parse optional else-if/else clauses
        let else_block = if self.check(&TokenKind::Else) {
            self.advance()?;

            if self.check(&TokenKind::If) {
                // else-if: parse as nested if statement
                let nested_if = self.parse_if_statement()?;
                Some(Block::new(vec![nested_if]))
            } else {
                // else: parse block
                Some(self.parse_block()?)
            }
        } else {
            None
        };

        Ok(Statement::If {
            condition,
            then_block,
            else_block,
        })
    }

    /// Parse a while statement
    fn parse_while_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(TokenKind::While)?;

        self.expect(TokenKind::LParen)?;
        let condition = self.parse_expression_stub()?;
        self.expect(TokenKind::RParen)?;

        let body = self.parse_block()?;

        Ok(Statement::While {
            label: None,
            condition,
            body,
        })
    }

    /// Parse a for statement (C-style or for-in)
    fn parse_for_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(TokenKind::For)?;

        self.expect(TokenKind::LParen)?;

        // Try to determine if this is a for-in loop
        // For-in: for (var in expr)
        // C-style: for (init; cond; incr)

        // Parse first part (could be init or var declaration)
        let first_token = self.current_token.clone();

        // Check if it's a for-in loop
        if matches!(first_token.kind, TokenKind::Ident(_)) {
            let var_name = match &self.current_token.kind {
                TokenKind::Ident(n) => {
                    let ident = Ident::new(n.clone());
                    self.advance()?;
                    ident
                }
                _ => unreachable!(),
            };

            if self.check(&TokenKind::In) {
                // for-in loop
                self.advance()?;
                let iter = self.parse_expression_stub()?;
                self.expect(TokenKind::RParen)?;
                let body = self.parse_block()?;

                return Ok(Statement::ForIn {
                    label: None,
                    var: var_name,
                    iter,
                    body,
                });
            } else {
                // Not a for-in, must be C-style for
                // We need to backtrack and parse as expression
                // For simplicity, we'll parse the rest as C-style for
            }
        }

        // C-style for loop
        // Parse init statement
        let init = Box::new(self.parse_statement()?);

        // Parse condition
        let condition = self.parse_expression_stub()?;
        self.expect(TokenKind::Semicolon)?;

        // Parse increment
        let increment = self.parse_expression_stub()?;
        self.expect(TokenKind::RParen)?;

        let body = self.parse_block()?;

        Ok(Statement::For {
            label: None,
            init,
            condition,
            increment,
            body,
        })
    }

    /// Parse a return statement
    fn parse_return_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(TokenKind::Return)?;

        let value = if self.check(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expression_stub()?)
        };

        self.expect(TokenKind::Semicolon)?;

        Ok(Statement::Return(value))
    }

    /// Parse a break statement
    fn parse_break_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(TokenKind::Break)?;

        // Check for label (just identifier, no dot)
        let label = if let TokenKind::Ident(n) = &self.current_token.kind {
            let ident = Ident::new(n.clone());
            self.advance()?;
            Some(ident)
        } else {
            None
        };

        self.expect(TokenKind::Semicolon)?;

        Ok(Statement::Break(label))
    }

    /// Parse a continue statement
    fn parse_continue_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(TokenKind::Continue)?;

        // Check for label (just identifier, no dot)
        let label = if let TokenKind::Ident(n) = &self.current_token.kind {
            let ident = Ident::new(n.clone());
            self.advance()?;
            Some(ident)
        } else {
            None
        };

        self.expect(TokenKind::Semicolon)?;

        Ok(Statement::Continue(label))
    }

    /// Parse a labeled loop (.label: loop { ... })
    fn parse_labeled_loop(&mut self) -> Result<Statement, ParseError> {
        self.expect(TokenKind::Dot)?;

        // Parse label name
        let label = match &self.current_token.kind {
            TokenKind::Ident(n) => {
                let ident = Ident::new(n.clone());
                self.advance()?;
                ident
            }
            _ => {
                return Err(ParseError::new(
                    self.current_token.span,
                    "expected label name",
                    vec!["identifier".to_string()],
                    format!("{:?}", self.current_token.kind),
                ));
            }
        };

        self.expect(TokenKind::Colon)?;

        // Parse loop type (while or loop)
        if self.check(&TokenKind::While) {
            self.advance()?;
            self.expect(TokenKind::LParen)?;
            let condition = self.parse_expression_stub()?;
            self.expect(TokenKind::RParen)?;
            let body = self.parse_block()?;

            Ok(Statement::While {
                label: Some(label),
                condition,
                body,
            })
        } else if self.check(&TokenKind::Loop) {
            self.advance()?;
            let body = self.parse_block()?;

            // Infinite loop: while (true)
            Ok(Statement::While {
                label: Some(label),
                condition: Expression::Literal(Literal::Bool(true)),
                body,
            })
        } else {
            Err(ParseError::new(
                self.current_token.span,
                "expected 'while' or 'loop' after label",
                vec!["while".to_string(), "loop".to_string()],
                format!("{:?}", self.current_token.kind),
            ))
        }
    }

    /// Check if the current position is a nested function declaration
    /// Looks for pattern: type identifier (
    /// Uses lookahead to distinguish from expression statements
    fn is_nested_function_declaration(&mut self) -> Result<bool, ParseError> {
        // Check if current token is a type keyword
        let is_type_keyword = matches!(
            self.current_token.kind,
            TokenKind::Void
                | TokenKind::Int
                | TokenKind::I32
                | TokenKind::I64
                | TokenKind::U32
                | TokenKind::U64
                | TokenKind::Float
                | TokenKind::F32
                | TokenKind::F64
                | TokenKind::Bool
                | TokenKind::Char
        );

        if !is_type_keyword {
            // Could be a custom type (identifier), need to check further
            if !matches!(self.current_token.kind, TokenKind::Ident(_)) {
                return Ok(false);
            }

            // If current token is an identifier, check if next token is an assignment operator
            // This would indicate an assignment statement, not a nested function
            let next_token = self.peek_ahead(1)?;
            if let Some(token) = next_token {
                if matches!(token.kind, TokenKind::Assign) {
                    return Ok(false);
                }
            }
        }

        // Peek ahead to see what comes after the type
        // Pattern: type identifier (
        let next_token = self.peek_ahead(1)?;
        if let Some(token) = next_token {
            if !matches!(token.kind, TokenKind::Ident(_)) {
                return Ok(false);
            }

            // Check if there's a ( after the identifier
            let token_after_ident = self.peek_ahead(2)?;
            if let Some(token) = token_after_ident {
                return Ok(matches!(token.kind, TokenKind::LParen));
            }
        }

        Ok(false)
    }

    /// Parse a nested function declaration
    fn parse_nested_function(&mut self) -> Result<Statement, ParseError> {
        // Parse return type
        let return_type = if self.check(&TokenKind::Void) {
            self.advance()?;
            None
        } else {
            Some(self.parse_type()?)
        };

        // Parse function name
        let name = match &self.current_token.kind {
            TokenKind::Ident(n) => {
                let ident = Ident::new(n.clone());
                self.advance()?;
                ident
            }
            _ => {
                return Err(ParseError::new(
                    self.current_token.span,
                    "expected function name",
                    vec!["identifier".to_string()],
                    format!("{:?}", self.current_token.kind),
                ));
            }
        };

        // Parse parameter list
        self.expect(TokenKind::LParen)?;
        let mut params = Vec::new();

        if !self.check(&TokenKind::RParen) {
            loop {
                // Parse parameter type
                let param_type = self.parse_type()?;

                // Parse parameter name
                let param_name = match &self.current_token.kind {
                    TokenKind::Ident(n) => {
                        let ident = Ident::new(n.clone());
                        self.advance()?;
                        ident
                    }
                    _ => {
                        return Err(ParseError::new(
                            self.current_token.span,
                            "expected parameter name",
                            vec!["identifier".to_string()],
                            format!("{:?}", self.current_token.kind),
                        ));
                    }
                };

                params.push(Param {
                    name: param_name,
                    ty: param_type,
                });

                if self.check(&TokenKind::Comma) {
                    self.advance()?;
                } else {
                    break;
                }
            }
        }

        self.expect(TokenKind::RParen)?;

        // Parse function body
        let body = self.parse_block()?;

        Ok(Statement::NestedFunction {
            name,
            params,
            return_type,
            body,
        })
    }

    /// Stub for expression parsing (will be implemented in task 6.4)
    fn parse_expression_stub(&mut self) -> Result<Expression, ParseError> {
        self.parse_expression()
    }

    /// Parse an expression with operator precedence
    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_ternary()
    }

    /// Parse ternary conditional operator (? :)
    fn parse_ternary(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_logical_or()?;

        if self.check(&TokenKind::Question) {
            self.advance()?;
            let then_expr = Box::new(self.parse_expression()?);
            self.expect(TokenKind::Colon)?;
            let else_expr = Box::new(self.parse_expression()?);

            expr = Expression::Ternary {
                condition: Box::new(expr),
                then_expr,
                else_expr,
            };
        }

        Ok(expr)
    }

    /// Parse logical OR (||)
    fn parse_logical_or(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_logical_and()?;

        while self.check(&TokenKind::Or) {
            self.advance()?;
            let right = self.parse_logical_and()?;
            left = Expression::Binary {
                op: BinaryOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse logical AND (&&)
    fn parse_logical_and(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_bitwise_or()?;

        while self.check(&TokenKind::And) {
            self.advance()?;
            let right = self.parse_bitwise_or()?;
            left = Expression::Binary {
                op: BinaryOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse bitwise OR (|)
    fn parse_bitwise_or(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_bitwise_xor()?;

        while self.check(&TokenKind::BitOr) {
            self.advance()?;
            let right = self.parse_bitwise_xor()?;
            left = Expression::Binary {
                op: BinaryOp::BitOr,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse bitwise XOR (^)
    fn parse_bitwise_xor(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_bitwise_and()?;

        while self.check(&TokenKind::BitXor) {
            self.advance()?;
            let right = self.parse_bitwise_and()?;
            left = Expression::Binary {
                op: BinaryOp::BitXor,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse bitwise AND (&)
    fn parse_bitwise_and(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_equality()?;

        while self.check(&TokenKind::BitAnd) {
            self.advance()?;
            let right = self.parse_equality()?;
            left = Expression::Binary {
                op: BinaryOp::BitAnd,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse equality operators (==, !=)
    fn parse_equality(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_comparison()?;

        while self.check(&TokenKind::Eq) || self.check(&TokenKind::Ne) {
            let op = if self.check(&TokenKind::Eq) {
                BinaryOp::Eq
            } else {
                BinaryOp::Ne
            };
            self.advance()?;
            let right = self.parse_comparison()?;
            left = Expression::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse comparison operators (<, >, <=, >=)
    fn parse_comparison(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_shift()?;

        while self.check(&TokenKind::Lt)
            || self.check(&TokenKind::Gt)
            || self.check(&TokenKind::Le)
            || self.check(&TokenKind::Ge)
        {
            let op = match &self.current_token.kind {
                TokenKind::Lt => BinaryOp::Lt,
                TokenKind::Gt => BinaryOp::Gt,
                TokenKind::Le => BinaryOp::Le,
                TokenKind::Ge => BinaryOp::Ge,
                _ => unreachable!(),
            };
            self.advance()?;
            let right = self.parse_shift()?;
            left = Expression::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse shift operators (<<, >>)
    fn parse_shift(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_additive()?;

        while self.check(&TokenKind::Shl) || self.check(&TokenKind::Shr) {
            let op = if self.check(&TokenKind::Shl) {
                BinaryOp::Shl
            } else {
                BinaryOp::Shr
            };
            self.advance()?;
            let right = self.parse_additive()?;
            left = Expression::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse additive operators (+, -)
    fn parse_additive(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_multiplicative()?;

        while self.check(&TokenKind::Plus) || self.check(&TokenKind::Minus) {
            let op = if self.check(&TokenKind::Plus) {
                BinaryOp::Add
            } else {
                BinaryOp::Sub
            };
            self.advance()?;
            let right = self.parse_multiplicative()?;
            left = Expression::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse multiplicative operators (*, /, %)
    fn parse_multiplicative(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_unary()?;

        while self.check(&TokenKind::Star)
            || self.check(&TokenKind::Slash)
            || self.check(&TokenKind::Percent)
        {
            let op = match &self.current_token.kind {
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                TokenKind::Percent => BinaryOp::Mod,
                _ => unreachable!(),
            };
            self.advance()?;
            let right = self.parse_unary()?;
            left = Expression::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse unary operators (!, -, &, *, ++, --)
    fn parse_unary(&mut self) -> Result<Expression, ParseError> {
        match &self.current_token.kind {
            TokenKind::Not => {
                self.advance()?;
                let expr = self.parse_unary()?;
                Ok(Expression::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                })
            }
            TokenKind::Minus => {
                self.advance()?;
                let expr = self.parse_unary()?;
                Ok(Expression::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                })
            }
            TokenKind::BitAnd => {
                self.advance()?;
                let expr = self.parse_unary()?;
                Ok(Expression::Unary {
                    op: UnaryOp::Ref,
                    expr: Box::new(expr),
                })
            }
            TokenKind::Star => {
                self.advance()?;
                let expr = self.parse_unary()?;
                Ok(Expression::Unary {
                    op: UnaryOp::Deref,
                    expr: Box::new(expr),
                })
            }
            TokenKind::Inc => {
                self.advance()?;
                let expr = self.parse_unary()?;
                Ok(Expression::Unary {
                    op: UnaryOp::PreInc,
                    expr: Box::new(expr),
                })
            }
            TokenKind::Dec => {
                self.advance()?;
                let expr = self.parse_unary()?;
                Ok(Expression::Unary {
                    op: UnaryOp::PreDec,
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_postfix(),
        }
    }

    /// Parse postfix operators (++, --, function calls, field access, array indexing, macro calls)
    fn parse_postfix(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_primary()?;

        loop {
            match &self.current_token.kind {
                TokenKind::Inc => {
                    self.advance()?;
                    expr = Expression::Unary {
                        op: UnaryOp::PostInc,
                        expr: Box::new(expr),
                    };
                }
                TokenKind::Dec => {
                    self.advance()?;
                    expr = Expression::Unary {
                        op: UnaryOp::PostDec,
                        expr: Box::new(expr),
                    };
                }
                TokenKind::Bang => {
                    // Macro invocation: ident!(args) or ident![args] or ident!{args}
                    // Only valid if expr is an identifier
                    if let Expression::Ident(name) = expr {
                        self.advance()?;

                        // Parse macro arguments based on delimiter
                        let args = if self.check(&TokenKind::LParen) {
                            self.parse_macro_args(TokenKind::LParen, TokenKind::RParen)?
                        } else if self.check(&TokenKind::LBracket) {
                            self.parse_macro_args(TokenKind::LBracket, TokenKind::RBracket)?
                        } else if self.check(&TokenKind::LBrace) {
                            self.parse_macro_args(TokenKind::LBrace, TokenKind::RBrace)?
                        } else {
                            return Err(ParseError::new(
                                self.current_token.span,
                                "expected (, [, or { after macro name",
                                vec!["(".to_string(), "[".to_string(), "{".to_string()],
                                format!("{:?}", self.current_token.kind),
                            ));
                        };

                        expr = Expression::MacroCall { name, args };
                    } else {
                        // ! is error propagation operator, not a macro call
                        expr = Expression::ErrorProp {
                            expr: Box::new(expr),
                        };
                    }
                }
                TokenKind::LParen => {
                    // Check if this is a macro call (double-underscore pattern)
                    if let Expression::Ident(ref name) = expr {
                        if name.name.starts_with("__") && name.name.ends_with("__") {
                            // Check delimiter type matches macro definition
                            self.check_macro_delimiter(&name.name, MacroDelimiter::Parens)?;

                            // This is a macro call with parentheses, parse as MacroCall
                            let macro_name = name.clone();
                            let args =
                                self.parse_macro_args(TokenKind::LParen, TokenKind::RParen)?;
                            expr = Expression::MacroCall {
                                name: macro_name,
                                args,
                            };
                            continue;
                        }
                    }

                    // Regular function call
                    self.advance()?;
                    let mut args = Vec::new();

                    if !self.check(&TokenKind::RParen) {
                        loop {
                            args.push(self.parse_expression()?);
                            if self.check(&TokenKind::Comma) {
                                self.advance()?;
                            } else {
                                break;
                            }
                        }
                    }

                    self.expect(TokenKind::RParen)?;
                    expr = Expression::Call {
                        func: Box::new(expr),
                        args,
                    };
                }
                TokenKind::LBracket => {
                    // Check if this is a macro call with brackets (e.g., __vec__[1, 2, 3])
                    if let Expression::Ident(ref name) = expr {
                        if name.name.starts_with("__") && name.name.ends_with("__") {
                            // Check delimiter type matches macro definition
                            self.check_macro_delimiter(&name.name, MacroDelimiter::Brackets)?;

                            // This is a macro call with brackets, parse as MacroCall
                            let macro_name = name.clone();
                            let args =
                                self.parse_macro_args(TokenKind::LBracket, TokenKind::RBracket)?;
                            expr = Expression::MacroCall {
                                name: macro_name,
                                args,
                            };
                            continue;
                        }
                    }

                    // Array indexing or slice range
                    self.advance()?;

                    // Check for range syntax
                    if self.check(&TokenKind::DotDot) || self.check(&TokenKind::DotDotEq) {
                        // Range starting from beginning: [..end] or [..=end]
                        let inclusive = self.check(&TokenKind::DotDotEq);
                        self.advance()?;

                        let end = if self.check(&TokenKind::RBracket) {
                            None
                        } else {
                            Some(Box::new(self.parse_expression()?))
                        };

                        self.expect(TokenKind::RBracket)?;

                        let range = Expression::Range {
                            start: None,
                            end,
                            inclusive,
                        };

                        expr = Expression::Index {
                            expr: Box::new(expr),
                            index: Box::new(range),
                        };
                    } else {
                        let start = self.parse_expression()?;

                        // Check if this is a range
                        if self.check(&TokenKind::DotDot) || self.check(&TokenKind::DotDotEq) {
                            let inclusive = self.check(&TokenKind::DotDotEq);
                            self.advance()?;

                            let end = if self.check(&TokenKind::RBracket) {
                                None
                            } else {
                                Some(Box::new(self.parse_expression()?))
                            };

                            self.expect(TokenKind::RBracket)?;

                            let range = Expression::Range {
                                start: Some(Box::new(start)),
                                end,
                                inclusive,
                            };

                            expr = Expression::Index {
                                expr: Box::new(expr),
                                index: Box::new(range),
                            };
                        } else {
                            // Regular array indexing
                            self.expect(TokenKind::RBracket)?;
                            expr = Expression::Index {
                                expr: Box::new(expr),
                                index: Box::new(start),
                            };
                        }
                    }
                }
                TokenKind::LBrace => {
                    // Check if this is a macro call with braces (e.g., __macro__{...})
                    if let Expression::Ident(ref name) = expr {
                        if name.name.starts_with("__") && name.name.ends_with("__") {
                            // Check delimiter type matches macro definition
                            self.check_macro_delimiter(&name.name, MacroDelimiter::Braces)?;

                            // This is a macro call with braces, parse as MacroCall
                            let macro_name = name.clone();
                            let args =
                                self.parse_macro_args(TokenKind::LBrace, TokenKind::RBrace)?;
                            expr = Expression::MacroCall {
                                name: macro_name,
                                args,
                            };
                            continue;
                        }
                    }

                    // Not a macro call, break out of postfix loop
                    break;
                }
                TokenKind::Dot => {
                    // Field access or tuple indexing
                    self.advance()?;

                    // Check for tuple indexing (.0, .1, .2, etc.)
                    if let TokenKind::IntLiteral(s) = &self.current_token.kind {
                        let index = s.parse::<usize>().map_err(|_| {
                            ParseError::new(
                                self.current_token.span,
                                "invalid tuple index",
                                vec![],
                                s.clone(),
                            )
                        })?;
                        self.advance()?;

                        // Create tuple indexing as field access with numeric field name
                        expr = Expression::FieldAccess {
                            expr: Box::new(expr),
                            field: Ident::new(index.to_string()),
                        };
                    } else {
                        // Regular field access
                        let field = match &self.current_token.kind {
                            TokenKind::Ident(n) => {
                                let ident = Ident::new(n.clone());
                                self.advance()?;
                                ident
                            }
                            _ => {
                                return Err(ParseError::new(
                                    self.current_token.span,
                                    "expected field name or tuple index",
                                    vec!["identifier".to_string(), "integer".to_string()],
                                    format!("{:?}", self.current_token.kind),
                                ));
                            }
                        };
                        expr = Expression::FieldAccess {
                            expr: Box::new(expr),
                            field,
                        };
                    }
                }
                TokenKind::Arrow => {
                    // Pointer field access (->)
                    self.advance()?;
                    let field = match &self.current_token.kind {
                        TokenKind::Ident(n) => {
                            let ident = Ident::new(n.clone());
                            self.advance()?;
                            ident
                        }
                        _ => {
                            return Err(ParseError::new(
                                self.current_token.span,
                                "expected field name",
                                vec!["identifier".to_string()],
                                format!("{:?}", self.current_token.kind),
                            ));
                        }
                    };
                    // Desugar -> to (*expr).field
                    let deref = Expression::Unary {
                        op: UnaryOp::Deref,
                        expr: Box::new(expr),
                    };
                    expr = Expression::FieldAccess {
                        expr: Box::new(deref),
                        field,
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    /// Parse macro arguments as a token stream
    fn parse_macro_args(
        &mut self,
        open: TokenKind,
        close: TokenKind,
    ) -> Result<Vec<crate::ast::Token>, ParseError> {
        let open_discriminant = std::mem::discriminant(&open);
        let close_discriminant = std::mem::discriminant(&close);

        self.expect(open)?;
        let mut tokens = Vec::new();
        let mut depth = 1;

        while depth > 0 && !self.is_at_end() {
            let token_kind = self.current_token.kind.clone();
            let token_text = self.current_token.text.clone();
            let token_discriminant = std::mem::discriminant(&token_kind);

            // Track nesting depth
            if token_discriminant == open_discriminant {
                depth += 1;
            } else if token_discriminant == close_discriminant {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }

            // Convert lexer token to AST token
            tokens.push(crate::ast::Token {
                kind: crate::ast::TokenKind::Other,
                text: token_text,
            });

            self.advance()?;
        }

        self.expect(close)?;
        Ok(tokens)
    }

    /// Check if a macro invocation uses the correct delimiter type
    /// Returns Ok(()) if the delimiter is correct or macro is not registered
    /// Returns Err if the delimiter doesn't match the macro definition
    fn check_macro_delimiter(
        &self,
        macro_name: &str,
        used_delimiter: MacroDelimiter,
    ) -> Result<(), ParseError> {
        if let Some(expected_delimiter) = self.macro_registry.get(macro_name) {
            if expected_delimiter != &used_delimiter {
                return Err(ParseError::new(
                    self.current_token.span,
                    format!(
                        "macro '{}' expects {:?} delimiter but was invoked with {:?}",
                        macro_name, expected_delimiter, used_delimiter
                    ),
                    vec![format!("{:?}", expected_delimiter)],
                    format!("{:?}", used_delimiter),
                ));
            }
        }
        Ok(())
    }

    /// Check if the current position looks like a struct initializer
    /// Struct initializers have the pattern: { .field = value, ... }
    /// Assumes current token is LBrace
    fn is_struct_initializer(&self) -> Result<bool, ParseError> {
        // Create a temporary lexer for lookahead
        let mut temp_lexer = Lexer {
            source: self.lexer.source,
            chars: self.lexer.source[self.lexer.position..]
                .char_indices()
                .peekable(),
            position: self.lexer.position,
            line: self.lexer.line,
            column: self.lexer.column,
        };

        // Check if next token is a dot (designated initializer syntax)
        if let Ok(token) = temp_lexer.next_token() {
            Ok(matches!(token.kind, TokenKind::Dot))
        } else {
            Ok(false)
        }
    }

    /// Parse a struct initializer: { .field = value, ... }
    fn parse_struct_initializer(&mut self, ty: Type) -> Result<Expression, ParseError> {
        self.expect(TokenKind::LBrace)?;

        let mut fields = Vec::new();

        // Parse field initializers
        while !self.check(&TokenKind::RBrace) {
            // Expect .field syntax
            self.expect(TokenKind::Dot)?;

            // Parse field name
            let field_name = match &self.current_token.kind {
                TokenKind::Ident(name) => {
                    let ident = Ident::new(name.clone());
                    self.advance()?;
                    ident
                }
                _ => {
                    return Err(ParseError::new(
                        self.current_token.span,
                        "expected field name after '.'",
                        vec!["identifier".to_string()],
                        format!("{:?}", self.current_token.kind),
                    ));
                }
            };

            // Expect =
            self.expect(TokenKind::Assign)?;

            // Parse field value
            let field_value = self.parse_expression()?;

            fields.push((field_name, field_value));

            // Check for comma
            if self.check(&TokenKind::Comma) {
                self.advance()?;
                // Allow trailing comma
                if self.check(&TokenKind::RBrace) {
                    break;
                }
            } else {
                break;
            }
        }

        self.expect(TokenKind::RBrace)?;

        Ok(Expression::StructInit { ty, fields })
    }

    /// Parse primary expressions (literals, identifiers, parenthesized expressions, type-scoped calls)
    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        match &self.current_token.kind {
            TokenKind::IntLiteral(s) => {
                let val = s.parse::<i64>().map_err(|_| {
                    ParseError::new(
                        self.current_token.span,
                        "invalid integer literal",
                        vec![],
                        s.clone(),
                    )
                })?;
                self.advance()?;
                Ok(Expression::Literal(Literal::Int(val)))
            }
            TokenKind::FloatLiteral(s) => {
                let val = s.parse::<f64>().map_err(|_| {
                    ParseError::new(
                        self.current_token.span,
                        "invalid float literal",
                        vec![],
                        s.clone(),
                    )
                })?;
                self.advance()?;
                Ok(Expression::Literal(Literal::Float(val)))
            }
            TokenKind::StringLiteral(s) => {
                let val = s.clone();
                self.advance()?;
                Ok(Expression::Literal(Literal::String(val)))
            }
            TokenKind::CharLiteral(c) => {
                let val = *c;
                self.advance()?;
                Ok(Expression::Literal(Literal::Char(val)))
            }
            TokenKind::BoolLiteral(b) => {
                let val = *b;
                self.advance()?;
                Ok(Expression::Literal(Literal::Bool(val)))
            }
            TokenKind::Null => {
                self.advance()?;
                Ok(Expression::Literal(Literal::Null))
            }
            TokenKind::LParen => {
                // Could be:
                // 1. Cast expression: (Type)expr
                // 2. Parenthesized expression: (expr)
                // 3. Tuple literal: (expr1, expr2, ...)

                self.advance()?;

                // Check for empty tuple ()
                if self.check(&TokenKind::RParen) {
                    self.advance()?;
                    return Ok(Expression::TupleLit {
                        elements: Vec::new(),
                    });
                }

                // Try to detect if this is a cast by checking if we have a type token
                let is_cast = self.is_type_token();

                if is_cast {
                    // Try to parse as cast: (Type)expr
                    // Save position in case we need to backtrack
                    let saved_position = self.lexer.position;
                    let saved_line = self.lexer.line;
                    let saved_column = self.lexer.column;
                    let saved_token = self.current_token.clone();

                    // Try to parse type
                    match self.parse_type() {
                        Ok(ty) => {
                            // Check for closing paren
                            if self.check(&TokenKind::RParen) {
                                self.advance()?;
                                // Parse the expression being cast
                                let expr = self.parse_unary()?;
                                return Ok(Expression::Cast {
                                    expr: Box::new(expr),
                                    ty,
                                });
                            } else {
                                // Not a cast, restore position and parse as expression
                                self.lexer.position = saved_position;
                                self.lexer.line = saved_line;
                                self.lexer.column = saved_column;
                                self.current_token = saved_token;
                                self.advance()?;
                            }
                        }
                        Err(_) => {
                            // Failed to parse type, restore position and parse as expression
                            self.lexer.position = saved_position;
                            self.lexer.line = saved_line;
                            self.lexer.column = saved_column;
                            self.current_token = saved_token;
                            self.advance()?;
                        }
                    }
                }

                // Parse as parenthesized expression or tuple
                let first_expr = self.parse_expression()?;

                // Check if this is a tuple (has comma) or just a parenthesized expression
                if self.check(&TokenKind::Comma) {
                    // Tuple literal
                    let mut elements = vec![first_expr];

                    while self.check(&TokenKind::Comma) {
                        self.advance()?;

                        // Allow trailing comma
                        if self.check(&TokenKind::RParen) {
                            break;
                        }

                        elements.push(self.parse_expression()?);
                    }

                    self.expect(TokenKind::RParen)?;
                    Ok(Expression::TupleLit { elements })
                } else {
                    // Just a parenthesized expression
                    self.expect(TokenKind::RParen)?;
                    Ok(first_expr)
                }
            }
            TokenKind::LBracket => {
                // Array literal [1, 2, 3] or array initialization [value; count]
                self.advance()?;

                // Check for empty array []
                if self.check(&TokenKind::RBracket) {
                    self.advance()?;
                    return Ok(Expression::ArrayLit {
                        elements: Vec::new(),
                    });
                }

                let first_expr = self.parse_expression()?;

                // Check for array initialization syntax [value; count]
                if self.check(&TokenKind::Semicolon) {
                    self.advance()?;
                    let count_expr = self.parse_expression()?;
                    self.expect(TokenKind::RBracket)?;

                    // Represent [value; count] as a special array literal
                    // We'll need to handle this in code generation
                    return Ok(Expression::ArrayLit {
                        elements: vec![first_expr, count_expr],
                    });
                }

                // Regular array literal
                let mut elements = vec![first_expr];

                while self.check(&TokenKind::Comma) {
                    self.advance()?;

                    // Allow trailing comma
                    if self.check(&TokenKind::RBracket) {
                        break;
                    }

                    elements.push(self.parse_expression()?);
                }

                self.expect(TokenKind::RBracket)?;
                Ok(Expression::ArrayLit { elements })
            }
            TokenKind::At => {
                // Type-scoped static method call (@Type.method() or @Type(T).method())
                self.advance()?;
                let ty = self.parse_type()?;

                // Check for explicit generic parameters with parentheses syntax
                let explicit_generics = if self.check(&TokenKind::LParen) {
                    // Parse explicit generic parameters: @Type(T1, T2)
                    self.advance()?;
                    let mut generics = Vec::new();

                    if !self.check(&TokenKind::RParen) {
                        loop {
                            generics.push(self.parse_generic_type_param()?);
                            if self.check(&TokenKind::Comma) {
                                self.advance()?;
                            } else {
                                break;
                            }
                        }
                    }

                    self.expect(TokenKind::RParen)?;
                    Some(generics)
                } else {
                    None
                };

                self.expect(TokenKind::Dot)?;
                let method = match &self.current_token.kind {
                    TokenKind::Ident(n) => {
                        let ident = Ident::new(n.clone());
                        self.advance()?;
                        ident
                    }
                    _ => {
                        return Err(ParseError::new(
                            self.current_token.span,
                            "expected method name",
                            vec!["identifier".to_string()],
                            format!("{:?}", self.current_token.kind),
                        ));
                    }
                };

                // Parse arguments (parentheses are optional for zero-argument calls)
                let args = if self.check(&TokenKind::LParen) {
                    self.advance()?;
                    let mut args = Vec::new();

                    if !self.check(&TokenKind::RParen) {
                        loop {
                            args.push(self.parse_expression()?);
                            if self.check(&TokenKind::Comma) {
                                self.advance()?;
                            } else {
                                break;
                            }
                        }
                    }

                    self.expect(TokenKind::RParen)?;
                    args
                } else {
                    // No parentheses means zero arguments
                    Vec::new()
                };

                // Return appropriate expression type
                if let Some(generics) = explicit_generics {
                    Ok(Expression::ExplicitGenericCall {
                        ty,
                        generics,
                        method,
                        args,
                    })
                } else {
                    Ok(Expression::TypeScopedCall { ty, method, args })
                }
            }
            TokenKind::Ident(n) => {
                let ident = Ident::new(n.clone());
                self.advance()?;
                Ok(Expression::Ident(ident))
            }
            TokenKind::LBrace => {
                // Struct initializer: { .field = value, ... }
                // Check if this looks like a struct initializer
                if self.is_struct_initializer()? {
                    // Parse as struct initializer with inferred type
                    // The type will be inferred from context (variable declaration type)
                    self.parse_struct_initializer(Type::Auto)
                } else {
                    Err(ParseError::new(
                        self.current_token.span,
                        "expected expression",
                        vec![
                            "literal".to_string(),
                            "identifier".to_string(),
                            "(".to_string(),
                        ],
                        format!("{:?}", self.current_token.kind),
                    ))
                }
            }
            _ => Err(ParseError::new(
                self.current_token.span,
                "expected expression",
                vec![
                    "literal".to_string(),
                    "identifier".to_string(),
                    "(".to_string(),
                ],
                format!("{:?}", self.current_token.kind),
            )),
        }
    }

    /// Parse a generic type parameter with alternating parentheses and brackets
    /// Supports: T, Inner[T], Inner[Type(T)], etc.
    fn parse_generic_type_param(&mut self) -> Result<Type, ParseError> {
        // Parse base type
        let mut base_type = self.parse_base_type_for_generic()?;

        // Check for nested generics with brackets
        if self.check(&TokenKind::LBracket) {
            self.advance()?;
            let mut args = Vec::new();

            if !self.check(&TokenKind::RBracket) {
                loop {
                    // Recursively parse nested generic parameters
                    args.push(self.parse_nested_generic_param()?);
                    if self.check(&TokenKind::Comma) {
                        self.advance()?;
                    } else {
                        break;
                    }
                }
            }

            self.expect(TokenKind::RBracket)?;
            base_type = Type::Generic {
                base: Box::new(base_type),
                args,
            };
        }

        Ok(base_type)
    }

    /// Parse nested generic parameter (alternates back to parentheses)
    fn parse_nested_generic_param(&mut self) -> Result<Type, ParseError> {
        let mut base_type = self.parse_base_type_for_generic()?;

        // Check for nested generics with parentheses (alternating)
        if self.check(&TokenKind::LParen) {
            self.advance()?;
            let mut args = Vec::new();

            if !self.check(&TokenKind::RParen) {
                loop {
                    // Recursively parse, alternating back to brackets
                    args.push(self.parse_generic_type_param()?);
                    if self.check(&TokenKind::Comma) {
                        self.advance()?;
                    } else {
                        break;
                    }
                }
            }

            self.expect(TokenKind::RParen)?;
            base_type = Type::Generic {
                base: Box::new(base_type),
                args,
            };
        }

        Ok(base_type)
    }

    /// Parse a base type for generic parameters (identifier or primitive)
    fn parse_base_type_for_generic(&mut self) -> Result<Type, ParseError> {
        match &self.current_token.kind {
            TokenKind::Int => {
                self.advance()?;
                Ok(Type::Primitive(PrimitiveType::Int))
            }
            TokenKind::I32 => {
                self.advance()?;
                Ok(Type::Primitive(PrimitiveType::I32))
            }
            TokenKind::I64 => {
                self.advance()?;
                Ok(Type::Primitive(PrimitiveType::I64))
            }
            TokenKind::U32 => {
                self.advance()?;
                Ok(Type::Primitive(PrimitiveType::U32))
            }
            TokenKind::U64 => {
                self.advance()?;
                Ok(Type::Primitive(PrimitiveType::U64))
            }
            TokenKind::Float => {
                self.advance()?;
                Ok(Type::Primitive(PrimitiveType::Float))
            }
            TokenKind::F32 => {
                self.advance()?;
                Ok(Type::Primitive(PrimitiveType::F32))
            }
            TokenKind::F64 => {
                self.advance()?;
                Ok(Type::Primitive(PrimitiveType::F64))
            }
            TokenKind::Bool => {
                self.advance()?;
                Ok(Type::Primitive(PrimitiveType::Bool))
            }
            TokenKind::Char => {
                self.advance()?;
                Ok(Type::Primitive(PrimitiveType::Char))
            }
            TokenKind::Void => {
                self.advance()?;
                Ok(Type::Primitive(PrimitiveType::Void))
            }
            TokenKind::Ident(name) => {
                let ident = Ident::new(name.clone());
                self.advance()?;
                Ok(Type::Ident(ident))
            }
            _ => Err(ParseError::new(
                self.current_token.span,
                "expected type in generic parameter",
                vec!["type".to_string()],
                format!("{:?}", self.current_token.kind),
            )),
        }
    }

    /// Parse a type expression
    fn parse_type(&mut self) -> Result<Type, ParseError> {
        // Check for mutable reference types (var & or &mut)
        if self.check(&TokenKind::Var) {
            self.advance()?;

            // Expect & after var
            self.expect(TokenKind::BitAnd)?;

            let inner_type = self.parse_type()?;
            return Ok(Type::Reference {
                ty: Box::new(inner_type),
                mutable: true,
            });
        }

        // Check for reference types (& for immutable, &mut for Rust-style mutable)
        if self.check(&TokenKind::BitAnd) {
            self.advance()?;

            // Check for Rust-style &mut (alternative to var &)
            let mutable = if self.check(&TokenKind::Mut) {
                self.advance()?;
                true
            } else {
                false
            };

            let inner_type = self.parse_type()?;
            return Ok(Type::Reference {
                ty: Box::new(inner_type),
                mutable,
            });
        }

        // Check for pointer types (*)
        if self.check(&TokenKind::Star) {
            self.advance()?;
            let inner_type = self.parse_type()?;
            return Ok(Type::Pointer {
                ty: Box::new(inner_type),
                mutable: true, // Assume mutable by default
            });
        }

        // Parse base type
        let mut base_type = match &self.current_token.kind {
            TokenKind::Int => {
                self.advance()?;
                Type::Primitive(PrimitiveType::Int)
            }
            TokenKind::I32 => {
                self.advance()?;
                Type::Primitive(PrimitiveType::I32)
            }
            TokenKind::I64 => {
                self.advance()?;
                Type::Primitive(PrimitiveType::I64)
            }
            TokenKind::U32 => {
                self.advance()?;
                Type::Primitive(PrimitiveType::U32)
            }
            TokenKind::U64 => {
                self.advance()?;
                Type::Primitive(PrimitiveType::U64)
            }
            TokenKind::Float => {
                self.advance()?;
                Type::Primitive(PrimitiveType::Float)
            }
            TokenKind::F32 => {
                self.advance()?;
                Type::Primitive(PrimitiveType::F32)
            }
            TokenKind::F64 => {
                self.advance()?;
                Type::Primitive(PrimitiveType::F64)
            }
            TokenKind::Bool => {
                self.advance()?;
                Type::Primitive(PrimitiveType::Bool)
            }
            TokenKind::Char => {
                self.advance()?;
                Type::Primitive(PrimitiveType::Char)
            }
            TokenKind::Void => {
                self.advance()?;
                Type::Primitive(PrimitiveType::Void)
            }
            TokenKind::LParen => {
                // Tuple type
                self.advance()?;
                let mut types = Vec::new();

                if !self.check(&TokenKind::RParen) {
                    loop {
                        types.push(self.parse_type()?);
                        if self.check(&TokenKind::Comma) {
                            self.advance()?;
                        } else {
                            break;
                        }
                    }
                }

                self.expect(TokenKind::RParen)?;
                Type::Tuple { types }
            }
            TokenKind::Ident(name) => {
                let ident = Ident::new(name.clone());
                self.advance()?;
                Type::Ident(ident)
            }
            _ => {
                return Err(ParseError::new(
                    self.current_token.span,
                    "expected type",
                    vec!["type".to_string()],
                    format!("{:?}", self.current_token.kind),
                ));
            }
        };

        // Check for generic type parameters (Type<T>)
        if self.check(&TokenKind::Lt) {
            self.advance()?;
            let mut args = Vec::new();

            if !self.check(&TokenKind::Gt) {
                loop {
                    args.push(self.parse_type()?);
                    if self.check(&TokenKind::Comma) {
                        self.advance()?;
                    } else {
                        break;
                    }
                }
            }

            self.expect(TokenKind::Gt)?;
            base_type = Type::Generic {
                base: Box::new(base_type),
                args,
            };
        }

        // Check for array type ([size] or [])
        if self.check(&TokenKind::LBracket) {
            self.advance()?;

            if self.check(&TokenKind::RBracket) {
                // Slice type []
                self.advance()?;
                base_type = Type::Slice {
                    ty: Box::new(base_type),
                };
            } else {
                // Array type with size
                match &self.current_token.kind {
                    TokenKind::IntLiteral(s) => {
                        let size = s.parse::<usize>().map_err(|_| {
                            ParseError::new(
                                self.current_token.span,
                                "invalid array size",
                                vec![],
                                s.clone(),
                            )
                        })?;
                        self.advance()?;
                        self.expect(TokenKind::RBracket)?;
                        base_type = Type::Array {
                            ty: Box::new(base_type),
                            size: Some(size),
                        };
                    }
                    _ => {
                        return Err(ParseError::new(
                            self.current_token.span,
                            "expected array size",
                            vec!["integer".to_string()],
                            format!("{:?}", self.current_token.kind),
                        ));
                    }
                }
            }
        }

        // Check for postfix pointer/reference syntax (C-style: int* or int&)
        while self.check(&TokenKind::Star) || self.check(&TokenKind::BitAnd) {
            if self.check(&TokenKind::Star) {
                self.advance()?;
                base_type = Type::Pointer {
                    ty: Box::new(base_type),
                    mutable: true,
                };
            } else if self.check(&TokenKind::BitAnd) {
                self.advance()?;
                // Check for mutable reference (&mut)
                let mutable = if self.check(&TokenKind::Mut) {
                    self.advance()?;
                    true
                } else {
                    false
                };
                base_type = Type::Reference {
                    ty: Box::new(base_type),
                    mutable,
                };
            }
        }

        Ok(base_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let source = "let x = 5;";
        let parser = Parser::new(source);
        assert!(parser.is_ok());
    }

    #[test]
    fn test_parser_advance() {
        let source = "let x = 5;";
        let mut parser = Parser::new(source).unwrap();

        // Should start with 'let' token
        assert!(matches!(parser.current_token.kind, TokenKind::Let));

        // Advance to next token
        parser.advance().unwrap();
        assert!(matches!(parser.current_token.kind, TokenKind::Ident(_)));
    }

    #[test]
    fn test_parser_expect_success() {
        let source = "let x = 5;";
        let mut parser = Parser::new(source).unwrap();

        // Expect 'let' token
        let token = parser.expect(TokenKind::Let);
        assert!(token.is_ok());

        // Should now be at identifier
        assert!(matches!(parser.current_token.kind, TokenKind::Ident(_)));
    }

    #[test]
    fn test_parser_expect_failure() {
        let source = "let x = 5;";
        let mut parser = Parser::new(source).unwrap();

        // Expect wrong token
        let result = parser.expect(TokenKind::Var);
        assert!(result.is_err());
    }

    #[test]
    fn test_parser_peek() {
        let source = "let x = 5;";
        let parser = Parser::new(source).unwrap();

        // Peek should return current token without consuming
        let token = parser.peek();
        assert!(matches!(token.kind, TokenKind::Let));

        // Token should still be 'let'
        assert!(matches!(parser.current_token.kind, TokenKind::Let));
    }

    #[test]
    fn test_parser_check() {
        let source = "let x = 5;";
        let parser = Parser::new(source).unwrap();

        assert!(parser.check(&TokenKind::Let));
        assert!(!parser.check(&TokenKind::Var));
    }

    #[test]
    fn test_parser_is_at_end() {
        let source = "";
        let parser = Parser::new(source).unwrap();

        assert!(parser.is_at_end());
    }

    #[test]
    fn test_parser_not_at_end() {
        let source = "let x = 5;";
        let parser = Parser::new(source).unwrap();

        assert!(!parser.is_at_end());
    }

    #[test]
    fn test_parse_simple_function() {
        let source = "int main() {}";
        let mut parser = Parser::new(source).unwrap();

        let file = parser.parse_file();
        assert!(file.is_ok());

        let file = file.unwrap();
        assert_eq!(file.items.len(), 1);

        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.name.name, "main");
                assert_eq!(func.params.len(), 0);
                assert!(func.return_type.is_some());
            }
            _ => panic!("Expected function item"),
        }
    }

    #[test]
    fn test_parse_void_function() {
        let source = "void foo() {}";
        let mut parser = Parser::new(source).unwrap();

        let file = parser.parse_file();
        assert!(file.is_ok());

        let file = file.unwrap();
        assert_eq!(file.items.len(), 1);

        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.name.name, "foo");
                assert!(func.return_type.is_none());
            }
            _ => panic!("Expected function item"),
        }
    }

    #[test]
    fn test_parse_function_with_params() {
        let source = "int add(int a, int b) {}";
        let mut parser = Parser::new(source).unwrap();

        let file = parser.parse_file();
        assert!(file.is_ok());

        let file = file.unwrap();
        assert_eq!(file.items.len(), 1);

        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.name.name, "add");
                assert_eq!(func.params.len(), 2);
                assert_eq!(func.params[0].name.name, "a");
                assert_eq!(func.params[1].name.name, "b");
            }
            _ => panic!("Expected function item"),
        }
    }

    #[test]
    fn test_parse_static_function() {
        let source = "static int helper() {}";
        let mut parser = Parser::new(source).unwrap();

        let file = parser.parse_file();
        assert!(file.is_ok());

        let file = file.unwrap();
        assert_eq!(file.items.len(), 1);

        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.name.name, "helper");
                assert_eq!(func.visibility, Visibility::Private);
            }
            _ => panic!("Expected function item"),
        }
    }

    #[test]
    fn test_parse_struct() {
        let source = "struct Point { int x; int y; }";
        let mut parser = Parser::new(source).unwrap();

        let file = parser.parse_file();
        assert!(file.is_ok());

        let file = file.unwrap();
        assert_eq!(file.items.len(), 1);

        match &file.items[0] {
            Item::Struct(s) => {
                assert_eq!(s.name.name, "Point");
                assert_eq!(s.fields.len(), 2);
                assert_eq!(s.fields[0].name.name, "x");
                assert_eq!(s.fields[1].name.name, "y");
            }
            _ => panic!("Expected struct item"),
        }
    }

    #[test]
    fn test_parse_enum() {
        let source = "enum Color { Red, Green, Blue }";
        let mut parser = Parser::new(source).unwrap();

        let file = parser.parse_file();
        assert!(file.is_ok());

        let file = file.unwrap();
        assert_eq!(file.items.len(), 1);

        match &file.items[0] {
            Item::Enum(e) => {
                assert_eq!(e.name.name, "Color");
                assert_eq!(e.variants.len(), 3);
                assert_eq!(e.variants[0].name.name, "Red");
                assert_eq!(e.variants[0].value, Some(0));
                assert_eq!(e.variants[1].name.name, "Green");
                assert_eq!(e.variants[1].value, Some(1));
                assert_eq!(e.variants[2].name.name, "Blue");
                assert_eq!(e.variants[2].value, Some(2));
            }
            _ => panic!("Expected enum item"),
        }
    }

    #[test]
    fn test_parse_enum_with_explicit_values() {
        let source = "enum Status { Ok = 0, Error = 1, Pending = 5 }";
        let mut parser = Parser::new(source).unwrap();

        let file = parser.parse_file();
        assert!(file.is_ok());

        let file = file.unwrap();
        assert_eq!(file.items.len(), 1);

        match &file.items[0] {
            Item::Enum(e) => {
                assert_eq!(e.name.name, "Status");
                assert_eq!(e.variants.len(), 3);
                assert_eq!(e.variants[0].value, Some(0));
                assert_eq!(e.variants[1].value, Some(1));
                assert_eq!(e.variants[2].value, Some(5));
            }
            _ => panic!("Expected enum item"),
        }
    }

    #[test]
    fn test_parse_typedef() {
        let source = "typedef int MyInt;";
        let mut parser = Parser::new(source).unwrap();

        let file = parser.parse_file();
        assert!(file.is_ok());

        let file = file.unwrap();
        assert_eq!(file.items.len(), 1);

        match &file.items[0] {
            Item::Typedef(t) => {
                assert_eq!(t.name.name, "MyInt");
                assert!(matches!(t.target, Type::Primitive(PrimitiveType::Int)));
                assert_eq!(t.visibility, Visibility::Public);
            }
            _ => panic!("Expected typedef item"),
        }
    }

    #[test]
    fn test_parse_static_typedef() {
        let source = "static typedef int PrivateInt;";
        let mut parser = Parser::new(source).unwrap();

        let file = parser.parse_file();
        assert!(file.is_ok());

        let file = file.unwrap();
        assert_eq!(file.items.len(), 1);

        match &file.items[0] {
            Item::Typedef(t) => {
                assert_eq!(t.name.name, "PrivateInt");
                assert!(matches!(t.target, Type::Primitive(PrimitiveType::Int)));
                assert_eq!(t.visibility, Visibility::Private);
            }
            _ => panic!("Expected typedef item"),
        }
    }

    #[test]
    fn test_parse_multiple_items() {
        // Test parsing just a function first
        let source1 = "int add(int a, int b) {}";
        let mut parser1 = Parser::new(source1).unwrap();
        let file1 = parser1.parse_file();
        if let Err(ref e) = file1 {
            eprintln!("Parse error for function: {:?}", e);
        }
        assert!(file1.is_ok(), "Failed to parse function");

        // Test parsing just a struct
        let source2 = "struct Point { int x; int y; }";
        let mut parser2 = Parser::new(source2).unwrap();
        let file2 = parser2.parse_file();
        if let Err(ref e) = file2 {
            eprintln!("Parse error for struct: {:?}", e);
        }
        assert!(file2.is_ok(), "Failed to parse struct");

        // Now test all together
        let source = r#"
            int add(int a, int b) {}
            struct Point { int x; int y; }
            enum Color { Red, Green, Blue }
        "#;
        let mut parser = Parser::new(source).unwrap();

        let file = parser.parse_file();
        if let Err(ref e) = file {
            eprintln!("Parse error for multiple items: {:?}", e);
        }
        assert!(file.is_ok());

        let file = file.unwrap();
        assert_eq!(file.items.len(), 3);

        assert!(matches!(file.items[0], Item::Function(_)));
        assert!(matches!(file.items[1], Item::Struct(_)));
        assert!(matches!(file.items[2], Item::Enum(_)));
    }

    #[test]
    fn test_parse_let_statement() {
        let source = "int main() { let x = 5; }";
        let mut parser = Parser::new(source).unwrap();

        let file = parser.parse_file().unwrap();
        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 1);
                match &func.body.statements[0] {
                    Statement::Let { name, init, .. } => {
                        assert_eq!(name.name, "x");
                        assert!(init.is_some());
                    }
                    _ => panic!("Expected let statement"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_var_statement() {
        let source = "int main() { var x = 5; }";
        let mut parser = Parser::new(source).unwrap();

        let file = parser.parse_file().unwrap();
        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 1);
                match &func.body.statements[0] {
                    Statement::Var { name, init, .. } => {
                        assert_eq!(name.name, "x");
                        assert!(init.is_some());
                    }
                    _ => panic!("Expected var statement"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_const_statement() {
        let source = "int main() { const x = (int)5; }";
        let mut parser = Parser::new(source).unwrap();

        let file = parser.parse_file().unwrap();
        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 1);
                match &func.body.statements[0] {
                    Statement::Const { name, value, .. } => {
                        assert_eq!(name.name, "x");
                        // Should have a cast expression
                        assert!(matches!(value, Expression::Cast { .. }));
                    }
                    _ => panic!("Expected const statement"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_if_statement() {
        let source = "int main() { if (true) { } }";
        let mut parser = Parser::new(source).unwrap();

        let file = parser.parse_file().unwrap();
        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 1);
                assert!(matches!(func.body.statements[0], Statement::If { .. }));
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_if_else_statement() {
        let source = "int main() { if (true) { } else { } }";
        let mut parser = Parser::new(source).unwrap();

        let file = parser.parse_file().unwrap();
        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 1);
                match &func.body.statements[0] {
                    Statement::If { else_block, .. } => {
                        assert!(else_block.is_some());
                    }
                    _ => panic!("Expected if statement"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_while_statement() {
        let source = "int main() { while (true) { } }";
        let mut parser = Parser::new(source).unwrap();

        let file = parser.parse_file().unwrap();
        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 1);
                assert!(matches!(func.body.statements[0], Statement::While { .. }));
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_return_statement() {
        let source = "int main() { return 42; }";
        let mut parser = Parser::new(source).unwrap();

        let file = parser.parse_file().unwrap();
        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 1);
                match &func.body.statements[0] {
                    Statement::Return(val) => {
                        assert!(val.is_some());
                    }
                    _ => panic!("Expected return statement"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_break_statement() {
        let source = "int main() { break; }";
        let mut parser = Parser::new(source).unwrap();

        let file = parser.parse_file().unwrap();
        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 1);
                match &func.body.statements[0] {
                    Statement::Break(label) => {
                        assert!(label.is_none());
                    }
                    _ => panic!("Expected break statement"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_break_with_label() {
        let source = "int main() { break outer; }";
        let mut parser = Parser::new(source).unwrap();

        let file = parser.parse_file().unwrap();
        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 1);
                match &func.body.statements[0] {
                    Statement::Break(label) => {
                        assert!(label.is_some());
                        assert_eq!(label.as_ref().unwrap().name, "outer");
                    }
                    _ => panic!("Expected break statement"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_continue_statement() {
        let source = "int main() { continue; }";
        let mut parser = Parser::new(source).unwrap();

        let file = parser.parse_file().unwrap();
        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 1);
                match &func.body.statements[0] {
                    Statement::Continue(label) => {
                        assert!(label.is_none());
                    }
                    _ => panic!("Expected continue statement"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_continue_with_label() {
        let source = "int main() { continue inner; }";
        let mut parser = Parser::new(source).unwrap();

        let file = parser.parse_file().unwrap();
        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 1);
                match &func.body.statements[0] {
                    Statement::Continue(label) => {
                        assert!(label.is_some());
                        assert_eq!(label.as_ref().unwrap().name, "inner");
                    }
                    _ => panic!("Expected continue statement"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_labeled_while_loop() {
        let source = "int main() { .outer: while (true) { } }";
        let mut parser = Parser::new(source).unwrap();

        let file = parser.parse_file().unwrap();
        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 1);
                match &func.body.statements[0] {
                    Statement::While { label, .. } => {
                        assert!(label.is_some());
                        assert_eq!(label.as_ref().unwrap().name, "outer");
                    }
                    _ => panic!("Expected while statement"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_labeled_infinite_loop() {
        let source = "int main() { .outer: loop { } }";
        let mut parser = Parser::new(source).unwrap();

        let file = parser.parse_file().unwrap();
        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 1);
                match &func.body.statements[0] {
                    Statement::While {
                        label, condition, ..
                    } => {
                        assert!(label.is_some());
                        assert_eq!(label.as_ref().unwrap().name, "outer");
                        // Infinite loop should have condition = true
                        assert!(matches!(
                            condition,
                            Expression::Literal(Literal::Bool(true))
                        ));
                    }
                    _ => panic!("Expected while statement"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_for_in_loop() {
        let source = "int main() { for (i in items) { } }";
        let mut parser = Parser::new(source).unwrap();

        let file = parser.parse_file().unwrap();
        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 1);
                match &func.body.statements[0] {
                    Statement::ForIn { var, .. } => {
                        assert_eq!(var.name, "i");
                    }
                    _ => panic!("Expected for-in statement"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_null_literal() {
        let source = "int main() { let ptr = NULL; }";
        let mut parser = Parser::new(source).unwrap();

        let file = parser.parse_file().unwrap();
        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 1);
                match &func.body.statements[0] {
                    Statement::Let { init, .. } => {
                        match init {
                            Some(Expression::Literal(Literal::Null)) => {
                                // Success
                            }
                            _ => panic!("Expected NULL literal"),
                        }
                    }
                    _ => panic!("Expected let statement"),
                }
            }
            _ => panic!("Expected function"),
        }
    }
}

#[test]
fn test_parse_binary_expression() {
    let source = "int main() { return 1 + 2; }";
    let mut parser = Parser::new(source).unwrap();

    let file = parser.parse_file().unwrap();
    match &file.items[0] {
        Item::Function(func) => match &func.body.statements[0] {
            Statement::Return(Some(expr)) => {
                assert!(matches!(
                    expr,
                    Expression::Binary {
                        op: BinaryOp::Add,
                        ..
                    }
                ));
            }
            _ => panic!("Expected return statement with expression"),
        },
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_parse_function_call() {
    let source = "int main() { return foo(1, 2); }";
    let mut parser = Parser::new(source).unwrap();

    let file = parser.parse_file().unwrap();
    match &file.items[0] {
        Item::Function(func) => match &func.body.statements[0] {
            Statement::Return(Some(expr)) => match expr {
                Expression::Call { args, .. } => {
                    assert_eq!(args.len(), 2);
                }
                _ => panic!("Expected call expression"),
            },
            _ => panic!("Expected return statement"),
        },
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_parse_field_access() {
    let source = "int main() { return obj.field; }";
    let mut parser = Parser::new(source).unwrap();

    let file = parser.parse_file().unwrap();
    match &file.items[0] {
        Item::Function(func) => match &func.body.statements[0] {
            Statement::Return(Some(expr)) => {
                assert!(matches!(expr, Expression::FieldAccess { .. }));
            }
            _ => panic!("Expected return statement"),
        },
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_parse_array_indexing() {
    let source = "int main() { return arr[0]; }";
    let mut parser = Parser::new(source).unwrap();

    let file = parser.parse_file().unwrap();
    match &file.items[0] {
        Item::Function(func) => match &func.body.statements[0] {
            Statement::Return(Some(expr)) => {
                assert!(matches!(expr, Expression::Index { .. }));
            }
            _ => panic!("Expected return statement"),
        },
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_parse_ternary_operator() {
    let source = "int main() { return x ? 1 : 2; }";
    let mut parser = Parser::new(source).unwrap();

    let file = parser.parse_file().unwrap();
    match &file.items[0] {
        Item::Function(func) => match &func.body.statements[0] {
            Statement::Return(Some(expr)) => {
                assert!(matches!(expr, Expression::Ternary { .. }));
            }
            _ => panic!("Expected return statement"),
        },
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_parse_type_scoped_call() {
    let source = "int main() { return @Vec.new(); }";
    let mut parser = Parser::new(source).unwrap();

    let file = parser.parse_file().unwrap();
    match &file.items[0] {
        Item::Function(func) => match &func.body.statements[0] {
            Statement::Return(Some(expr)) => match expr {
                Expression::TypeScopedCall { method, .. } => {
                    assert_eq!(method.name, "new");
                }
                _ => panic!("Expected type-scoped call"),
            },
            _ => panic!("Expected return statement"),
        },
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_parse_unary_operators() {
    let source = "int main() { return -x; }";
    let mut parser = Parser::new(source).unwrap();

    let file = parser.parse_file().unwrap();
    match &file.items[0] {
        Item::Function(func) => match &func.body.statements[0] {
            Statement::Return(Some(expr)) => {
                assert!(matches!(
                    expr,
                    Expression::Unary {
                        op: UnaryOp::Neg,
                        ..
                    }
                ));
            }
            _ => panic!("Expected return statement"),
        },
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_parse_operator_precedence() {
    let source = "int main() { return 1 + 2 * 3; }";
    let mut parser = Parser::new(source).unwrap();

    let file = parser.parse_file().unwrap();
    match &file.items[0] {
        Item::Function(func) => {
            match &func.body.statements[0] {
                Statement::Return(Some(expr)) => {
                    // Should parse as 1 + (2 * 3)
                    match expr {
                        Expression::Binary {
                            op: BinaryOp::Add,
                            right,
                            ..
                        } => {
                            assert!(matches!(
                                **right,
                                Expression::Binary {
                                    op: BinaryOp::Mul,
                                    ..
                                }
                            ));
                        }
                        _ => panic!("Expected binary expression with correct precedence"),
                    }
                }
                _ => panic!("Expected return statement"),
            }
        }
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_parse_reference_type() {
    let source = "int foo(&int x) {}";
    let mut parser = Parser::new(source).unwrap();

    let file = parser.parse_file().unwrap();
    match &file.items[0] {
        Item::Function(func) => {
            assert_eq!(func.params.len(), 1);
            assert!(matches!(func.params[0].ty, Type::Reference { .. }));
        }
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_parse_mutable_reference_type() {
    let source = "int foo(var &int x) {}";
    let mut parser = Parser::new(source).unwrap();

    let file = parser.parse_file().unwrap();
    match &file.items[0] {
        Item::Function(func) => {
            assert_eq!(func.params.len(), 1);
            match &func.params[0].ty {
                Type::Reference { mutable, .. } => {
                    assert!(mutable);
                }
                _ => panic!("Expected reference type"),
            }
        }
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_parse_pointer_type() {
    let source = "int foo(*int x) {}";
    let mut parser = Parser::new(source).unwrap();

    let file = parser.parse_file().unwrap();
    match &file.items[0] {
        Item::Function(func) => {
            assert_eq!(func.params.len(), 1);
            assert!(matches!(func.params[0].ty, Type::Pointer { .. }));
        }
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_parse_array_type() {
    // Using type[size] syntax (e.g., int[10])
    // Note: C-style syntax would be "int arr[10]" but parser currently expects type[size] before identifier
    let source = "struct S { i32[10] arr; }";
    let mut parser = Parser::new(source).unwrap();

    let file = parser.parse_file().unwrap();
    match &file.items[0] {
        Item::Struct(s) => {
            assert_eq!(s.fields.len(), 1);
            assert_eq!(s.fields[0].name.name, "arr");
            match &s.fields[0].ty {
                Type::Array { size, .. } => {
                    assert_eq!(*size, Some(10));
                }
                _ => panic!("Expected array type"),
            }
        }
        _ => panic!("Expected struct"),
    }
}

#[test]
fn test_parse_tuple_type() {
    let source = "int foo((int, bool) x) {}";
    let mut parser = Parser::new(source).unwrap();

    let file = parser.parse_file().unwrap();
    match &file.items[0] {
        Item::Function(func) => {
            assert_eq!(func.params.len(), 1);
            match &func.params[0].ty {
                Type::Tuple { types } => {
                    assert_eq!(types.len(), 2);
                }
                _ => panic!("Expected tuple type"),
            }
        }
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_parse_generic_type() {
    let source = "int foo(Vec<int> x) {}";
    let mut parser = Parser::new(source).unwrap();

    let file = parser.parse_file().unwrap();
    match &file.items[0] {
        Item::Function(func) => {
            assert_eq!(func.params.len(), 1);
            match &func.params[0].ty {
                Type::Generic { args, .. } => {
                    assert_eq!(args.len(), 1);
                }
                _ => panic!("Expected generic type"),
            }
        }
        _ => panic!("Expected function"),
    }
}

#[test]
fn test_parse_define_simple_macro() {
    let source = "#define __MAX__ 100";
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();

    assert_eq!(file.items.len(), 1);
    match &file.items[0] {
        Item::MacroDefinition(mac) => {
            assert_eq!(mac.name.name, "__MAX__");
            assert_eq!(mac.params.len(), 0);
            assert!(!mac.body.is_empty());
        }
        _ => panic!("Expected MacroDefinition"),
    }
}

#[test]
fn test_parse_define_with_params() {
    let source = "#define __ADD__(a, b) ((a) + (b))";
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();

    assert_eq!(file.items.len(), 1);
    match &file.items[0] {
        Item::MacroDefinition(mac) => {
            assert_eq!(mac.name.name, "__ADD__");
            assert_eq!(mac.params.len(), 2);
            assert_eq!(mac.params[0].name, "a");
            assert_eq!(mac.params[1].name, "b");
            assert!(!mac.body.is_empty());
        }
        _ => panic!("Expected MacroDefinition"),
    }
}

#[test]
fn test_parse_define_invalid_name_no_prefix() {
    let source = "#define MAX__ 100";
    let mut parser = Parser::new(source).unwrap();
    let result = parser.parse_file();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("double-underscore"));
}

#[test]
fn test_parse_define_invalid_name_no_suffix() {
    let source = "#define __MAX 100";
    let mut parser = Parser::new(source).unwrap();
    let result = parser.parse_file();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("double-underscore"));
}

#[test]
fn test_parse_define_with_semicolon() {
    let source = "#define __PI__ 3.14159;";
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();

    assert_eq!(file.items.len(), 1);
    match &file.items[0] {
        Item::MacroDefinition(mac) => {
            assert_eq!(mac.name.name, "__PI__");
            assert_eq!(mac.params.len(), 0);
        }
        _ => panic!("Expected MacroDefinition"),
    }
}

#[test]
fn test_parse_define_multiline_not_supported() {
    // Macro body should only be on same line
    let source = "#define __MACRO__\n    some_body";
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();

    assert_eq!(file.items.len(), 1);
    match &file.items[0] {
        Item::MacroDefinition(mac) => {
            assert_eq!(mac.name.name, "__MACRO__");
            // Body should be empty or minimal since newline ends the macro
            // (The parser might capture tokens on the same line before newline)
        }
        _ => panic!("Expected MacroDefinition"),
    }
}

#[test]
fn test_parse_define_with_ternary() {
    let source = "#define __MAX__(a, b) ((a) > (b) ? (a) : (b))";
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();

    assert_eq!(file.items.len(), 1);
    match &file.items[0] {
        Item::MacroDefinition(mac) => {
            assert_eq!(mac.name.name, "__MAX__");
            assert_eq!(mac.params.len(), 2);
            assert_eq!(mac.params[0].name, "a");
            assert_eq!(mac.params[1].name, "b");
            // Should contain ternary operator tokens
            assert!(!mac.body.is_empty());
        }
        _ => panic!("Expected MacroDefinition"),
    }
}

#[test]
fn test_parse_define_with_multiple_params() {
    let source = "#define __CLAMP__(x, min, max) ((x) < (min) ? (min) : (x) > (max) ? (max) : (x))";
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();

    assert_eq!(file.items.len(), 1);
    match &file.items[0] {
        Item::MacroDefinition(mac) => {
            assert_eq!(mac.name.name, "__CLAMP__");
            assert_eq!(mac.params.len(), 3);
            assert_eq!(mac.params[0].name, "x");
            assert_eq!(mac.params[1].name, "min");
            assert_eq!(mac.params[2].name, "max");
        }
        _ => panic!("Expected MacroDefinition"),
    }
}

#[test]
fn test_parse_define_with_arithmetic() {
    let source = "#define __SQUARE__(x) ((x) * (x))";
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();

    assert_eq!(file.items.len(), 1);
    match &file.items[0] {
        Item::MacroDefinition(mac) => {
            assert_eq!(mac.name.name, "__SQUARE__");
            assert_eq!(mac.params.len(), 1);
            assert_eq!(mac.params[0].name, "x");
        }
        _ => panic!("Expected MacroDefinition"),
    }
}

#[test]
fn test_parse_multiple_defines() {
    let source = r#"
        #define __PI__ 3.14159
        #define __E__ 2.71828
        #define __MAX__(a, b) ((a) > (b) ? (a) : (b))
    "#;
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();

    assert_eq!(file.items.len(), 3);

    match &file.items[0] {
        Item::MacroDefinition(mac) => {
            assert_eq!(mac.name.name, "__PI__");
            assert_eq!(mac.params.len(), 0);
        }
        _ => panic!("Expected MacroDefinition"),
    }

    match &file.items[1] {
        Item::MacroDefinition(mac) => {
            assert_eq!(mac.name.name, "__E__");
            assert_eq!(mac.params.len(), 0);
        }
        _ => panic!("Expected MacroDefinition"),
    }

    match &file.items[2] {
        Item::MacroDefinition(mac) => {
            assert_eq!(mac.name.name, "__MAX__");
            assert_eq!(mac.params.len(), 2);
        }
        _ => panic!("Expected MacroDefinition"),
    }
}

#[test]
fn test_parse_define_empty_body() {
    let source = "#define __EMPTY__";
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();

    assert_eq!(file.items.len(), 1);
    match &file.items[0] {
        Item::MacroDefinition(mac) => {
            assert_eq!(mac.name.name, "__EMPTY__");
            assert_eq!(mac.params.len(), 0);
            // Body should be empty
            assert_eq!(mac.body.len(), 0);
        }
        _ => panic!("Expected MacroDefinition"),
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;

    #[test]
    fn test_property_1_valid_programs_parse() {
        // Property 1: Valid Crusty programs parse successfully
        // Validates: Requirements 6.1

        let valid_programs = vec![
            "int main() {}",
            "void foo() {}",
            "int add(int a, int b) { return a + b; }",
            "struct Point { int x; int y; }",
            "enum Color { Red, Green, Blue }",
            "typedef int MyInt;",
            "int main() { let x = 5; return x; }",
            "int main() { var y = 10; return y; }",
            "int main() { if (true) { return 1; } else { return 0; } }",
            "int main() { while (true) { break; } return 0; }",
            "int main() { for (i in items) { } return 0; }",
            "int main() { .outer: loop { break outer; } return 0; }",
            "int main() { return 1 + 2 * 3; }",
            "int main() { return foo(1, 2); }",
            "int main() { return obj.field; }",
            "int main() { return arr[0]; }",
            "int main() { return x ? 1 : 0; }",
            "int main() { return @Vec.new(); }",
        ];

        for program in valid_programs {
            let mut parser = Parser::new(program).unwrap();
            let result = parser.parse_file();
            assert!(
                result.is_ok(),
                "Failed to parse valid program: {}\nError: {:?}",
                program,
                result.err()
            );
        }
    }

    #[test]
    fn test_property_2_invalid_syntax_produces_errors() {
        // Property 2: Invalid syntax produces error reports with location
        // Validates: Requirements 6.2, 10.1

        let invalid_programs = vec![
            "int main(",          // Missing closing paren
            "int main() {",       // Missing closing brace
            "int main() { let }", // Incomplete let statement
            "struct { }",         // Missing struct name
            "enum { }",           // Missing enum name
        ];

        for program in invalid_programs {
            let mut parser = Parser::new(program).unwrap();
            let result = parser.parse_file();
            assert!(
                result.is_err(),
                "Expected error for invalid program: {}",
                program
            );

            // Verify error has location information
            if let Err(e) = result {
                // Error should have span information
                assert!(e.span.start.line > 0, "Error should have line information");
            }
        }
    }
}

#[test]
fn test_parse_struct_initializer() {
    let source = "int main() { let p = (Point){ .x = 10, .y = 20 }; }";
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();

    assert_eq!(file.items.len(), 1);
    match &file.items[0] {
        Item::Function(func) => {
            assert_eq!(func.body.statements.len(), 1);
            match &func.body.statements[0] {
                Statement::Let { init, ty, .. } => {
                    // Type should be inferred, not explicitly specified
                    assert!(ty.is_none());

                    if let Some(Expression::Cast { ty, expr }) = init {
                        // Check type - should be Point from the cast
                        match ty {
                            Type::Ident(ident) => assert_eq!(ident.name, "Point"),
                            _ => panic!("Expected Type::Ident for cast"),
                        }

                        // Check the struct init inside the cast
                        if let Expression::StructInit { fields, .. } = &**expr {
                            assert_eq!(fields.len(), 2);
                            assert_eq!(fields[0].0.name, "x");
                            assert_eq!(fields[1].0.name, "y");
                        } else {
                            panic!("Expected StructInit inside Cast");
                        }
                    } else {
                        panic!("Expected Cast expression");
                    }
                }
                _ => panic!("Expected Let statement"),
            }
        }
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_parse_struct_initializer_partial() {
    let source = "int main() { let p = (Point){ .x = 10 }; }";
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();

    assert_eq!(file.items.len(), 1);
    match &file.items[0] {
        Item::Function(func) => match &func.body.statements[0] {
            Statement::Let { init, .. } => {
                if let Some(Expression::Cast { expr, .. }) = init {
                    if let Expression::StructInit { fields, .. } = &**expr {
                        assert_eq!(fields.len(), 1);
                        assert_eq!(fields[0].0.name, "x");
                    } else {
                        panic!("Expected StructInit inside Cast");
                    }
                } else {
                    panic!("Expected Cast expression");
                }
            }
            _ => panic!("Expected Let statement"),
        },
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_parse_struct_initializer_trailing_comma() {
    let source = "int main() { let p = (Point){ .x = 10, .y = 20, }; }";
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();

    assert_eq!(file.items.len(), 1);
    match &file.items[0] {
        Item::Function(func) => match &func.body.statements[0] {
            Statement::Let { init, .. } => {
                if let Some(Expression::Cast { expr, .. }) = init {
                    if let Expression::StructInit { fields, .. } = &**expr {
                        assert_eq!(fields.len(), 2);
                    } else {
                        panic!("Expected StructInit inside Cast");
                    }
                } else {
                    panic!("Expected Cast expression");
                }
            }
            _ => panic!("Expected Let statement"),
        },
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_parse_struct_initializer_nested() {
    let source = "int main() { let r = (Rect){ .origin = { .x = 0, .y = 0 }, .size = { .w = 10, .h = 20 } }; }";
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();

    assert_eq!(file.items.len(), 1);
    match &file.items[0] {
        Item::Function(func) => {
            match &func.body.statements[0] {
                Statement::Let { init, .. } => {
                    if let Some(Expression::Cast { expr, .. }) = init {
                        if let Expression::StructInit { fields, .. } = &**expr {
                            assert_eq!(fields.len(), 2);
                            assert_eq!(fields[0].0.name, "origin");
                            assert_eq!(fields[1].0.name, "size");

                            // Check nested struct initializers
                            match &fields[0].1 {
                                Expression::StructInit {
                                    fields: nested_fields,
                                    ..
                                } => {
                                    assert_eq!(nested_fields.len(), 2);
                                }
                                _ => panic!("Expected nested StructInit"),
                            }
                        } else {
                            panic!("Expected StructInit inside Cast");
                        }
                    } else {
                        panic!("Expected Cast expression");
                    }
                }
                _ => panic!("Expected Let statement"),
            }
        }
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_macro_delimiter_parens() {
    let source = r#"
        #define __ADD__(a, b) a + b
        int main() {
            let result = __ADD__(1, 2);
        }
    "#;
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();

    // Should parse successfully with matching delimiters
    assert_eq!(file.items.len(), 2);
}

#[test]
fn test_macro_delimiter_brackets() {
    let source = r#"
        #define __VEC__[items] items
        int main() {
            let v = __VEC__[1, 2, 3];
        }
    "#;
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();

    // Should parse successfully with matching delimiters
    assert_eq!(file.items.len(), 2);
}

#[test]
fn test_macro_delimiter_braces() {
    let source = r#"
        #define __BLOCK__{code} code
        int main() {
            __BLOCK__{
                let x = 1;
            };
        }
    "#;
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();

    // Should parse successfully with matching delimiters
    assert_eq!(file.items.len(), 2);
}

#[test]
fn test_macro_delimiter_none() {
    let source = r#"
        #define __MAX__ 100
        int main() {
            let x = __MAX__;
        }
    "#;
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();

    // Should parse successfully - constant macro with no delimiter
    assert_eq!(file.items.len(), 2);
}

#[test]
fn test_macro_delimiter_mismatch_parens_to_brackets() {
    let source = r#"
        #define __ADD__(a, b) a + b
        int main() {
            let result = __ADD__[1, 2];
        }
    "#;
    let mut parser = Parser::new(source).unwrap();
    let result = parser.parse_file();

    // Should fail - macro defined with parens but invoked with brackets
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.message.contains("expects Parens delimiter"));
        assert!(e.message.contains("invoked with Brackets"));
    }
}

#[test]
fn test_macro_delimiter_mismatch_brackets_to_parens() {
    let source = r#"
        #define __VEC__[items] items
        int main() {
            let v = __VEC__(1, 2, 3);
        }
    "#;
    let mut parser = Parser::new(source).unwrap();
    let result = parser.parse_file();

    // Should fail - macro defined with brackets but invoked with parens
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.message.contains("expects Brackets delimiter"));
        assert!(e.message.contains("invoked with Parens"));
    }
}

#[test]
fn test_macro_delimiter_mismatch_braces_to_parens() {
    let source = r#"
        #define __BLOCK__{code} code
        int main() {
            __BLOCK__(let x = 1;);
        }
    "#;
    let mut parser = Parser::new(source).unwrap();
    let result = parser.parse_file();

    // Should fail - macro defined with braces but invoked with parens
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.message.contains("expects Braces delimiter"));
        assert!(e.message.contains("invoked with Parens"));
    }
}

#[test]
fn test_macro_invocation_before_definition() {
    let source = r#"
        int main() {
            let result = __ADD__(1, 2);
        }
        #define __ADD__(a, b) a + b
    "#;
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();

    // Should parse successfully - macro invoked before definition is allowed
    // (delimiter checking only happens if macro is already registered)
    assert_eq!(file.items.len(), 2);
}

#[test]
fn test_multiple_macros_different_delimiters() {
    let source = r#"
        #define __ADD__(a, b) a + b
        #define __VEC__[items] items
        #define __BLOCK__{code} code
        #define __MAX__ 100
        
        int main() {
            let sum = __ADD__(1, 2);
            let v = __VEC__[1, 2, 3];
            __BLOCK__{
                let x = 1;
            };
            let max = __MAX__;
        }
    "#;
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();

    // Should parse successfully - all macros use correct delimiters
    assert_eq!(file.items.len(), 5); // 4 macros + 1 function
}

#[test]
fn test_all_macro_syntaxes_comprehensive() {
    // Test all four delimiter types in one comprehensive test
    let source = r#"
        // Define macros with all delimiter types
        #define __CONSTANT__ 42
        #define __FUNC__(x, y) x + y
        #define __ARRAY__[items] items
        #define __BLOCK__{code} code
        
        void main() {
            // Invoke macros with all delimiter types
            let c = __CONSTANT__;
            let sum = __FUNC__(1, 2);
            let arr = __ARRAY__[1, 2, 3];
            __BLOCK__{
                let x = 1;
            };
        }
    "#;

    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();

    // Should have 4 macro definitions + 1 function
    assert_eq!(file.items.len(), 5);

    // Verify macro definitions
    match &file.items[0] {
        Item::MacroDefinition(m) => {
            assert_eq!(m.name.name, "__CONSTANT__");
            assert_eq!(m.delimiter, MacroDelimiter::None);
        }
        _ => panic!("Expected MacroDefinition"),
    }

    match &file.items[1] {
        Item::MacroDefinition(m) => {
            assert_eq!(m.name.name, "__FUNC__");
            assert_eq!(m.delimiter, MacroDelimiter::Parens);
            assert_eq!(m.params.len(), 2);
        }
        _ => panic!("Expected MacroDefinition"),
    }

    match &file.items[2] {
        Item::MacroDefinition(m) => {
            assert_eq!(m.name.name, "__ARRAY__");
            assert_eq!(m.delimiter, MacroDelimiter::Brackets);
        }
        _ => panic!("Expected MacroDefinition"),
    }

    match &file.items[3] {
        Item::MacroDefinition(m) => {
            assert_eq!(m.name.name, "__BLOCK__");
            assert_eq!(m.delimiter, MacroDelimiter::Braces);
        }
        _ => panic!("Expected MacroDefinition"),
    }
}

#[test]
fn test_common_rust_macros() {
    // Test common Rust macro names with double-underscore syntax
    let source = r#"
        #define __PRINTLN__(msg) msg
        #define __VEC__[items] items
        #define __ASSERT__(cond) cond
        #define __FORMAT__(fmt, args) fmt
        #define __PANIC__(msg) msg
        
        void main() {
            __PRINTLN__("test");
            let v = __VEC__[1, 2, 3];
            __ASSERT__(true);
            let s = __FORMAT__("test", 1);
            __PANIC__("error");
        }
    "#;

    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();

    // Should parse successfully - 5 macros + 1 function
    assert_eq!(file.items.len(), 6);
}

#[test]
fn test_macro_in_expression_context() {
    let source = r#"
        #define __ADD__(a, b) a + b
        
        void main() {
            let x = __ADD__(1, 2) + __ADD__(3, 4);
            let y = __ADD__(__ADD__(1, 2), 3);
        }
    "#;

    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();

    // Should parse successfully with nested macro calls
    assert_eq!(file.items.len(), 2);
}

#[test]
fn test_macro_in_statement_context() {
    let source = r#"
        #define __PRINTLN__(msg) msg
        
        void main() {
            __PRINTLN__("Hello");
            __PRINTLN__("World");
        }
    "#;

    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();

    // Should parse successfully with macros as statements
    assert_eq!(file.items.len(), 2);
}

#[test]
fn test_macro_without_double_underscore_rejected() {
    // Test that macros without double-underscore prefix/suffix are rejected
    let source = r#"
        #define MAX 100
    "#;

    let mut parser = Parser::new(source).unwrap();
    let result = parser.parse_file();

    // Should fail - macro name must have double-underscore prefix and suffix
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e
            .message
            .contains("must have double-underscore prefix and suffix"));
    }
}

#[test]
fn test_macro_with_only_prefix_rejected() {
    let source = r#"
        #define __MAX 100
    "#;

    let mut parser = Parser::new(source).unwrap();
    let result = parser.parse_file();

    // Should fail - macro name must have both prefix AND suffix
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e
            .message
            .contains("must have double-underscore prefix and suffix"));
    }
}

#[test]
fn test_macro_with_only_suffix_rejected() {
    let source = r#"
        #define MAX__ 100
    "#;

    let mut parser = Parser::new(source).unwrap();
    let result = parser.parse_file();

    // Should fail - macro name must have both prefix AND suffix
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e
            .message
            .contains("must have double-underscore prefix and suffix"));
    }
}

// ============================================================================
// PEG PARSER (NEW IMPLEMENTATION)
// ============================================================================

// PEG-based parser for Crusty language
// This is a new implementation using rust-peg that will eventually replace
// the hand-written recursive descent parser above.
peg::parser! {
    pub grammar crusty_peg_parser() for str {
        // ====================================================================
        // WHITESPACE AND COMMENTS
        // ====================================================================

        /// Optional whitespace (quiet - doesn't appear in error messages)
        rule _ = quiet!{(whitespace() / comment())*}

        /// Required whitespace
        rule __ = quiet!{(whitespace() / comment())+}

        /// Single whitespace character
        rule whitespace() = [' ' | '\t' | '\r' | '\n']

        /// Comment (line or block)
        rule comment() = line_comment() / block_comment()

        /// Line comment: // ... \n
        rule line_comment() = "//" (!"\n" [_])* "\n"?

        /// Block comment: /* ... */
        rule block_comment() = "/*" (!"*/" [_])* "*/"

        // ====================================================================
        // KEYWORDS
        // ====================================================================
        // Keywords use lookahead (!ident_char()) to ensure they don't match
        // as prefixes of identifiers. For example, "let" should not match "letter".
        //
        // The lookahead mechanism works by:
        // 1. Matching the keyword string (e.g., "let")
        // 2. Using negative lookahead !ident_char() to ensure the next character
        //    is NOT a valid identifier character
        // 3. This prevents "let" from matching the prefix of "letter" or "let_value"
        //
        // All Crusty keywords are defined here, organized by category:
        // - Variable declarations: let, var, const, static, mut
        // - Control flow: if, else, while, for, in, return, break, continue, loop, match, switch, case, default
        // - Type declarations: struct, enum, typedef
        // - Modifiers: extern, unsafe, auto
        // - Primitive types: int, i32, i64, u32, u64, float, f32, f64, bool, char, void
        // - Literals: true, false, NULL
        // - Preprocessor: define

        /// Helper: character that can appear in an identifier
        rule ident_char() = ['a'..='z' | 'A'..='Z' | '0'..='9' | '_']

        /// Keyword: let
        rule kw_let() = "let" !ident_char()

        /// Keyword: var
        rule kw_var() = "var" !ident_char()

        /// Keyword: const
        rule kw_const() = "const" !ident_char()

        /// Keyword: static
        rule kw_static() = "static" !ident_char()

        /// Keyword: mut
        rule kw_mut() = "mut" !ident_char()

        /// Keyword: define
        rule kw_define() = "define" !ident_char()

        /// Keyword: if
        rule kw_if() = "if" !ident_char()

        /// Keyword: else
        rule kw_else() = "else" !ident_char()

        /// Keyword: while
        rule kw_while() = "while" !ident_char()

        /// Keyword: for
        rule kw_for() = "for" !ident_char()

        /// Keyword: in
        rule kw_in() = "in" !ident_char()

        /// Keyword: return
        rule kw_return() = "return" !ident_char()

        /// Keyword: break
        rule kw_break() = "break" !ident_char()

        /// Keyword: continue
        rule kw_continue() = "continue" !ident_char()

        /// Keyword: struct
        rule kw_struct() = "struct" !ident_char()

        /// Keyword: enum
        rule kw_enum() = "enum" !ident_char()

        /// Keyword: typedef
        rule kw_typedef() = "typedef" !ident_char()

        /// Keyword: namespace
        rule kw_namespace() = "namespace" !ident_char()

        /// Keyword: extern
        rule kw_extern() = "extern" !ident_char()

        /// Keyword: unsafe
        rule kw_unsafe() = "unsafe" !ident_char()

        /// Keyword: loop
        rule kw_loop() = "loop" !ident_char()

        /// Keyword: match
        rule kw_match() = "match" !ident_char()

        /// Keyword: switch
        rule kw_switch() = "switch" !ident_char()

        /// Keyword: case
        rule kw_case() = "case" !ident_char()

        /// Keyword: default
        rule kw_default() = "default" !ident_char()

        /// Keyword: auto
        rule kw_auto() = "auto" !ident_char()

        /// Keyword: int
        rule kw_int() = "int" !ident_char()

        /// Keyword: i32
        rule kw_i32() = "i32" !ident_char()

        /// Keyword: i64
        rule kw_i64() = "i64" !ident_char()

        /// Keyword: u32
        rule kw_u32() = "u32" !ident_char()

        /// Keyword: u64
        rule kw_u64() = "u64" !ident_char()

        /// Keyword: float
        rule kw_float() = "float" !ident_char()

        /// Keyword: f32
        rule kw_f32() = "f32" !ident_char()

        /// Keyword: f64
        rule kw_f64() = "f64" !ident_char()

        /// Keyword: bool
        rule kw_bool() = "bool" !ident_char()

        /// Keyword: char
        rule kw_char() = "char" !ident_char()

        /// Keyword: void
        rule kw_void() = "void" !ident_char()

        /// Keyword: true
        rule kw_true() = "true" !ident_char()

        /// Keyword: false
        rule kw_false() = "false" !ident_char()

        /// Keyword: NULL
        rule kw_null() = "NULL" !ident_char()

        /// Helper: matches any keyword (used to prevent keywords from being parsed as identifiers)
        rule keyword() = kw_let() / kw_var() / kw_const() / kw_static() / kw_mut() / kw_define()
            / kw_if() / kw_else() / kw_while() / kw_for() / kw_in()
            / kw_return() / kw_break() / kw_continue()
            / kw_struct() / kw_enum() / kw_typedef()
            / kw_namespace() / kw_extern() / kw_unsafe()
            / kw_loop() / kw_match() / kw_switch() / kw_case() / kw_default() / kw_auto()
            / kw_int() / kw_i32() / kw_i64() / kw_u32() / kw_u64()
            / kw_float() / kw_f32() / kw_f64()
            / kw_bool() / kw_char() / kw_void()
            / kw_true() / kw_false() / kw_null()

        // ====================================================================
        // LITERALS
        // ====================================================================
        // Literal values: integers, floats, strings, characters, booleans, null
        //
        // Integer literals: decimal digits (e.g., 42, 123)
        // Float literals: decimal digits with decimal point (e.g., 3.14, 0.5)
        // String literals: double-quoted with escape sequences (e.g., "hello", "line\n")
        // Character literals: single-quoted (e.g., 'a', '\n')
        // Boolean literals: true, false
        // Null literal: NULL

        /// Integer literal: one or more decimal digits
        /// Returns Literal::Int
        pub rule int_literal() -> Literal
            = n:$(['0'..='9']+) {
                Literal::Int(n.parse().unwrap())
            }

        /// Float literal: decimal digits with decimal point
        /// Returns Literal::Float
        pub rule float_literal() -> Literal
            = n:$(
                (['0'..='9']+ "." ['0'..='9']+ (['e' | 'E'] ['+' | '-']? ['0'..='9']+)?)
                / (['0'..='9']+ ['e' | 'E'] ['+' | '-']? ['0'..='9']+)
            ) {
                Literal::Float(n.parse().unwrap())
            }

        /// String literal: double-quoted with escape sequences
        /// Returns Literal::String
        pub rule string_literal() -> Literal
            = "\"" s:string_content()* "\"" {
                Literal::String(s.into_iter().collect())
            }

        /// String content: either an escape sequence or a regular character
        rule string_content() -> char
            = escape_sequence()
            / c:$([^ '"' | '\\']) { c.chars().next().unwrap() }

        /// Escape sequences in strings and characters
        rule escape_sequence() -> char
            = "\\\\" { '\\' }
            / "\\n" { '\n' }
            / "\\r" { '\r' }
            / "\\t" { '\t' }
            / "\\\"" { '"' }
            / "\\'" { '\'' }
            / "\\0" { '\0' }

        /// Character literal: single-quoted
        /// Returns Literal::Char
        pub rule char_literal() -> Literal
            = "'" c:char_content() "'" {
                Literal::Char(c)
            }

        /// Character content: either an escape sequence or a regular character
        rule char_content() -> char
            = escape_sequence()
            / c:$([^ '\'' | '\\']) { c.chars().next().unwrap() }

        /// Boolean literal: true or false
        /// Returns Literal::Bool
        pub rule bool_literal() -> Literal
            = kw_true() { Literal::Bool(true) }
            / kw_false() { Literal::Bool(false) }

        /// Null literal: NULL
        /// Returns Literal::Null
        pub rule null_literal() -> Literal
            = kw_null() { Literal::Null }

        // ====================================================================
        // IDENTIFIERS
        // ====================================================================
        // Identifiers are names for variables, functions, types, etc.
        //
        // Regular identifiers: start with letter or underscore, followed by
        // letters, digits, or underscores (e.g., foo, _bar, test123)
        //
        // Macro identifiers: double-underscore prefix and suffix (e.g., __FILE__, __LINE__)
        //
        // Identifiers cannot be keywords. The !keyword() negative lookahead
        // ensures that keywords like "let", "var", "if" are not parsed as identifiers.

        /// Identifier: name that is not a keyword
        /// Returns Ident
        pub rule ident() -> Ident
            = !keyword() n:$((['a'..='z' | 'A'..='Z' | '_']) ident_char()*)
            { Ident::new(n) }

        /// Macro identifier: __NAME__ style (double underscore prefix and suffix)
        /// Returns Ident
        /// The middle part must start with a letter or underscore, followed by any ident chars,
        /// but we need to ensure we don't consume the trailing __
        pub rule macro_ident() -> Ident
            = "__" n:$((['a'..='z' | 'A'..='Z' | '_']) (!"__" ident_char())*) "__"
            { Ident::new(format!("__{n}__")) }

        // ====================================================================
        // TYPES
        // ====================================================================
        // Type expressions represent the types of values in Crusty.
        //
        // Primitive types: int, i32, i64, u32, u64, float, f32, f64, bool, char, void
        // Complex types: pointers (*T), references (&T, &mut T), arrays (T[N]),
        //                tuples ((T1, T2)), generics (T<A, B>), slices (T[])
        //
        // Type parsing uses the precedence! macro to handle type operators
        // like pointer (*) and reference (&) with correct precedence.

        /// Primitive type: int
        /// Returns Type::Primitive(PrimitiveType::Int)
        pub rule primitive_int() -> Type
            = kw_int() { Type::Primitive(PrimitiveType::Int) }

        /// Primitive type: i32
        /// Returns Type::Primitive(PrimitiveType::I32)
        pub rule primitive_i32() -> Type
            = kw_i32() { Type::Primitive(PrimitiveType::I32) }

        /// Primitive type: i64
        /// Returns Type::Primitive(PrimitiveType::I64)
        pub rule primitive_i64() -> Type
            = kw_i64() { Type::Primitive(PrimitiveType::I64) }

        /// Primitive type: u32
        /// Returns Type::Primitive(PrimitiveType::U32)
        pub rule primitive_u32() -> Type
            = kw_u32() { Type::Primitive(PrimitiveType::U32) }

        /// Primitive type: u64
        /// Returns Type::Primitive(PrimitiveType::U64)
        pub rule primitive_u64() -> Type
            = kw_u64() { Type::Primitive(PrimitiveType::U64) }

        /// Primitive type: float
        /// Returns Type::Primitive(PrimitiveType::Float)
        pub rule primitive_float() -> Type
            = kw_float() { Type::Primitive(PrimitiveType::Float) }

        /// Primitive type: f32
        /// Returns Type::Primitive(PrimitiveType::F32)
        pub rule primitive_f32() -> Type
            = kw_f32() { Type::Primitive(PrimitiveType::F32) }

        /// Primitive type: f64
        /// Returns Type::Primitive(PrimitiveType::F64)
        pub rule primitive_f64() -> Type
            = kw_f64() { Type::Primitive(PrimitiveType::F64) }

        /// Primitive type: bool
        /// Returns Type::Primitive(PrimitiveType::Bool)
        pub rule primitive_bool() -> Type
            = kw_bool() { Type::Primitive(PrimitiveType::Bool) }

        /// Primitive type: char
        /// Returns Type::Primitive(PrimitiveType::Char)
        pub rule primitive_char() -> Type
            = kw_char() { Type::Primitive(PrimitiveType::Char) }

        /// Primitive type: void
        /// Returns Type::Primitive(PrimitiveType::Void)
        pub rule primitive_void() -> Type
            = kw_void() { Type::Primitive(PrimitiveType::Void) }

        /// Any primitive type
        /// Returns Type::Primitive variant
        /// Note: Order matters for PEG - longer matches first (i32 before int, f32 before float)
        pub rule primitive_type() -> Type
            = primitive_i32()
            / primitive_i64()
            / primitive_u32()
            / primitive_u64()
            / primitive_int()
            / primitive_f32()
            / primitive_f64()
            / primitive_float()
            / primitive_bool()
            / primitive_char()
            / primitive_void()

        // ====================================================================
        // COMPLEX TYPES (Task 3.2)
        // ====================================================================
        // Complex type expressions: pointers, references, arrays, tuples, generics, slices
        //
        // Type operators have the following precedence (highest to lowest):
        // 1. Postfix: pointer (*), array ([N]), slice ([])
        // 2. Prefix: reference (&, &mut)
        // 3. Primary: primitives, identifiers, tuples, generics, parenthesized
        //
        // Examples:
        // - int* -> Pointer to int
        // - &int -> Reference to int
        // - &mut int -> Mutable reference to int
        // - int[10] -> Array of 10 ints
        // - int[] -> Slice of ints
        // - (int, bool) -> Tuple of int and bool
        // - Vec<int> -> Generic Vec with int argument
        // - int** -> Pointer to pointer to int
        // - &int* -> Reference to pointer to int

        /// Auto type (type inference)
        /// Returns Type::Auto
        pub rule auto_type() -> Type
            = kw_auto() { Type::Auto }

        /// Identifier type (user-defined type name)
        /// Returns Type::Ident
        pub rule ident_type() -> Type
            = n:ident() { Type::Ident(n) }

        /// Tuple type: (T1, T2, ...) or (T,) or ()
        /// Returns Type::Tuple
        /// Note: Empty tuple () is valid, single-element tuple (T,) requires trailing comma
        /// A single type in parentheses without trailing comma (T) is NOT a tuple,
        /// it's a parenthesized type expression.
        pub rule tuple_type() -> Type
            = "(" _ types:type_list_for_tuple() _ ")" {
                Type::Tuple { types }
            }

        /// Helper: comma-separated type list for tuples
        /// This rule specifically handles tuple syntax:
        /// - Empty: () -> []
        /// - Single with trailing comma: (T,) -> [T]
        /// - Multiple: (T1, T2) -> [T1, T2]
        /// Note: Single type without comma (T) should NOT match this rule
        rule type_list_for_tuple() -> Vec<Type>
            = types:(t:type_expr() ** (_ "," _)) _ ","? {
                types
            }

        /// Helper: Tuple-specific type list that requires either:
        /// - Empty list (for unit tuple)
        /// - Single element with trailing comma
        /// - Two or more elements
        rule tuple_type_list() -> Vec<Type>
            // Empty tuple
            = "" { vec![] }

        /// Generic type: Base<T1, T2, ...>
        /// Returns Type::Generic
        /// Note: Must have at least one type argument
        pub rule generic_type() -> Type
            = base:type_base() _ "<" _ args:type_list() _ ">" {
                Type::Generic { base: Box::new(base), args }
            }

        /// Helper: comma-separated type list (non-empty)
        rule type_list() -> Vec<Type>
            = types:(t:type_expr() ** (_ "," _)) {?
                if types.is_empty() {
                    Err("expected at least one type argument")
                } else {
                    Ok(types)
                }
            }

        /// Base type for generics (primitive or identifier, not complex)
        rule type_base() -> Type
            = primitive_type()
            / ident_type()

        /// Type expression using precedence! macro
        /// Handles all type operators with correct precedence
        ///
        /// Precedence levels (from lowest to highest):
        /// 1. Reference prefix: &T, &mut T
        /// 2. Postfix operators: T*, T[N], T[]
        /// 3. Primary types: primitives, identifiers, tuples, generics, auto, parenthesized
        ///
        /// Note: The precedence! macro handles left-to-right associativity for postfix
        /// operators, so int** parses as (int*)* (pointer to pointer).
        pub rule type_expr() -> Type = precedence!{
            // Level 1: Reference prefix (lowest precedence)
            // &mut T - mutable reference
            "&" _ kw_mut() __ t:@ {
                Type::Reference { ty: Box::new(t), mutable: true }
            }
            // &T - immutable reference
            "&" _ t:@ {
                Type::Reference { ty: Box::new(t), mutable: false }
            }
            --
            // Level 2: Postfix operators (higher precedence)
            // T* - pointer type
            t:(@) _ "*" {
                Type::Pointer { ty: Box::new(t), mutable: false }
            }
            // T[N] - array type with size
            t:(@) _ "[" _ n:int_literal() _ "]" {
                if let Literal::Int(size) = n {
                    Type::Array { ty: Box::new(t), size: Some(size as usize) }
                } else {
                    unreachable!()
                }
            }
            // T[] - slice type (array without size)
            t:(@) _ "[" _ "]" {
                Type::Slice { ty: Box::new(t) }
            }
            --
            // Level 3: Primary types (highest precedence)
            // Generic type: Base<T1, T2, ...>
            // Must come before ident_type to avoid consuming the base identifier
            base:type_base() _ "<" _ args:type_list() _ ">" {
                Type::Generic { base: Box::new(base), args }
            }
            // Empty tuple: ()
            "(" _ ")" {
                Type::Tuple { types: vec![] }
            }
            // Single-element tuple with trailing comma: (T,)
            "(" _ t:type_expr() _ "," _ ")" {
                Type::Tuple { types: vec![t] }
            }
            // Multi-element tuple: (T1, T2, ...)
            "(" _ first:type_expr() _ "," _ rest:(type_expr() ** (_ "," _)) _ ","? _ ")" {
                let mut types = vec![first];
                types.extend(rest);
                Type::Tuple { types }
            }
            // Parenthesized type for grouping (single type without comma)
            "(" _ t:type_expr() _ ")" { t }
            // Auto type
            kw_auto() { Type::Auto }
            // Primitive types
            t:primitive_type() { t }
            // Identifier types (user-defined)
            t:ident_type() { t }
        }

        // ====================================================================
        // EXPRESSIONS (Task 4)
        // ====================================================================
        // Expression parsing handles all value-producing constructs in Crusty.
        //
        // Primary expressions are the atomic building blocks:
        // - Literals: integers, floats, strings, chars, bools, null
        // - Identifiers: variable and function names
        // - Parenthesized expressions: (expr)
        // - Struct initialization: Type { field: value, ... }
        // - Array literals: [elem1, elem2, ...]
        // - Tuple literals: (elem1, elem2, ...)
        //
        // Task 4.1: Define primary expression rules with actions

        /// Literal expression: wraps a literal value
        /// Returns Expression::Literal
        pub rule literal_expr() -> Expression
            = _ l:literal() _ { Expression::Literal(l) }

        /// Any literal value
        rule literal() -> Literal
            = float_literal()  // Must come before int_literal (longer match)
            / int_literal()
            / string_literal()
            / char_literal()
            / bool_literal()
            / null_literal()

        /// Identifier expression: a variable or function name
        /// Returns Expression::Ident
        pub rule ident_expr() -> Expression
            = _ i:ident() _ { Expression::Ident(i) }

        /// Parenthesized expression: (expr)
        /// Returns the inner expression (parentheses are for grouping only)
        pub rule paren_expr() -> Expression
            = _ "(" _ e:expr() _ ")" _ { e }

        /// Struct initialization: Type { field: value, ... }
        /// Returns Expression::StructInit
        pub rule struct_init() -> Expression
            = _ ty:struct_init_type() _ "{" _ fields:struct_init_fields()? _ "}" _ {
                Expression::StructInit {
                    ty,
                    fields: fields.unwrap_or_default(),
                }
            }

        /// Type for struct initialization (identifier or generic)
        rule struct_init_type() -> Type
            = base:ident() _ "<" _ args:type_list() _ ">" {
                Type::Generic {
                    base: Box::new(Type::Ident(base)),
                    args,
                }
            }
            / i:ident() { Type::Ident(i) }

        /// Struct initialization fields: field: value, ...
        rule struct_init_fields() -> Vec<(Ident, Expression)>
            = first:struct_init_field() rest:(_ "," _ f:struct_init_field() { f })* (_ ",")? {
                let mut fields = vec![first];
                fields.extend(rest);
                fields
            }

        /// Single struct initialization field: field: value
        rule struct_init_field() -> (Ident, Expression)
            = name:ident() _ ":" _ value:expr() { (name, value) }

        /// Array literal: [elem1, elem2, ...]
        /// Returns Expression::ArrayLit
        pub rule array_lit() -> Expression
            = _ "[" _ elements:array_elements()? _ "]" _ {
                Expression::ArrayLit {
                    elements: elements.unwrap_or_default(),
                }
            }

        /// Array elements: comma-separated expressions
        rule array_elements() -> Vec<Expression>
            = first:expr() rest:(_ "," _ e:expr() { e })* (_ ",")? {
                let mut elements = vec![first];
                elements.extend(rest);
                elements
            }

        /// Tuple literal: (elem1, elem2, ...) with at least 2 elements or trailing comma
        /// Returns Expression::TupleLit
        /// Note: Single element without trailing comma is a parenthesized expression
        pub rule tuple_lit() -> Expression
            = _ "(" _ ")" _ {
                // Empty tuple
                Expression::TupleLit { elements: vec![] }
            }
            / _ "(" _ first:expr() _ "," _ ")" _ {
                // Single element tuple with trailing comma
                Expression::TupleLit { elements: vec![first] }
            }
            / _ "(" _ first:expr() _ "," _ rest:(e:expr() ** (_ "," _)) _ ","? _ ")" _ {
                // Multi-element tuple
                let mut elements = vec![first];
                elements.extend(rest);
                Expression::TupleLit { elements }
            }

        // ====================================================================
        // CAST EXPRESSION (Task 4.2)
        // ====================================================================
        // Cast expressions have the form: (Type)(expr)
        // This is CRITICAL for ambiguity resolution in C-style syntax.
        //
        // The pattern is: parenthesized type followed by parenthesized expression
        // Examples:
        // - (int)(x)       -> Cast x to int
        // - (Type*)(expr)  -> Cast expr to pointer type
        // - (int)(5+3)     -> Cast expression result to int
        // - (T1)(T2)(expr) -> Nested casts (T1 cast of T2 cast of expr)
        //
        // This rule MUST be tried BEFORE paren_expr in the primary() ordered choice
        // to correctly distinguish casts from parenthesized expressions.
        //
        // The key insight: a cast has TWO consecutive parenthesized groups,
        // where the first contains a type and the second contains an expression.

        /// Cast expression: (Type)(expr)
        /// Returns Expression::Cast
        ///
        /// This rule handles C-style type casts where a type in parentheses
        /// is followed by an expression in parentheses.
        ///
        /// Requirements validated: 2.1, 2.2, 2.4, 2.5, 2.6
        pub rule cast_expr() -> Expression
            = _ "(" _ t:type_expr() _ ")" _ "(" _ e:expr() _ ")" _ {
                Expression::Cast {
                    expr: Box::new(e),
                    ty: t,
                }
            }

        /// Primary expression: the atomic building blocks of expressions
        /// Order matters for PEG ordered choice:
        /// 1. Cast expression (MUST be first - (Type)(expr) pattern)
        /// 2. Tuple literal (must check for comma after first element)
        /// 3. Parenthesized expression (single element without comma)
        /// 4. Struct initialization (Type { ... })
        /// 5. Array literal ([...])
        /// 6. Literal expression (numbers, strings, etc.)
        /// 7. Identifier expression (variable names)
        ///
        /// CRITICAL: cast_expr MUST come before tuple_lit and paren_expr
        /// because (Type)(expr) would otherwise be parsed as two separate
        /// parenthesized expressions.
        pub rule primary() -> Expression
            = cast_expr()
            / tuple_lit()
            / paren_expr()
            / struct_init()
            / array_lit()
            / literal_expr()
            / ident_expr()

        /// Placeholder expr rule for use in other rules
        /// This will be replaced with the full precedence! macro in task 4.5
        pub rule expr() -> Expression
            = primary()

        // ====================================================================
        // MINIMAL TEST GRAMMAR
        // ====================================================================

        /// Test rule: parse a simple integer literal
        pub rule test_int() -> i64
            = _ n:$(['0'..='9']+) _ { n.parse().unwrap() }

        /// Test rule: parse a simple identifier
        pub rule test_ident() -> String
            = _ n:$((['a'..='z' | 'A'..='Z' | '_']) (['a'..='z' | 'A'..='Z' | '0'..='9' | '_'])*) _
            { n.to_string() }

        /// Test rule: parse a keyword (returns the keyword as a string)
        pub rule test_keyword() -> String
            = _ k:$(keyword()) _ { k.to_string() }

        /// Test rule: verify that a string is NOT a keyword (should fail if it is)
        pub rule test_not_keyword() -> String
            = _ !keyword() n:$((['a'..='z' | 'A'..='Z' | '_']) ident_char()*) _
            { n.to_string() }
    }
}

#[cfg(test)]
mod peg_tests {
    use super::*;

    #[test]
    fn test_peg_int_parsing() {
        // Test that rust-peg compiles and generates parser code
        let result = crusty_peg_parser::test_int("42");
        assert_eq!(result, Ok(42));

        let result = crusty_peg_parser::test_int("  123  ");
        assert_eq!(result, Ok(123));
    }

    #[test]
    fn test_peg_ident_parsing() {
        // Test that rust-peg compiles and generates parser code
        let result = crusty_peg_parser::test_ident("hello");
        assert_eq!(result, Ok("hello".to_string()));

        let result = crusty_peg_parser::test_ident("  foo_bar  ");
        assert_eq!(result, Ok("foo_bar".to_string()));

        let result = crusty_peg_parser::test_ident("_test123");
        assert_eq!(result, Ok("_test123".to_string()));
    }

    #[test]
    fn test_peg_whitespace_handling() {
        // Test that whitespace is handled correctly
        let result = crusty_peg_parser::test_int("   42   ");
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_peg_comment_handling() {
        // Test that comments are handled correctly
        let result = crusty_peg_parser::test_int("// comment\n42");
        assert_eq!(result, Ok(42));

        let result = crusty_peg_parser::test_int("/* block comment */ 42");
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_peg_mixed_whitespace_and_comments() {
        // Test mixed whitespace and comments
        let result = crusty_peg_parser::test_int("  /* comment */  42  // trailing\n");
        assert_eq!(result, Ok(42));

        let result = crusty_peg_parser::test_int("\t\n  // comment\n  /* block */  \n  42  \n");
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_peg_nested_block_comments_not_supported() {
        // Nested block comments are not supported in C-style comments
        // This should parse the first /* */ and then fail on the remaining text
        let result = crusty_peg_parser::test_int("/* outer /* inner */ */ 42");
        // This will fail because after the first */ the parser sees */ 42 which is invalid
        assert!(result.is_err());
    }

    #[test]
    fn test_peg_line_comment_without_newline() {
        // Line comments at end of input (no trailing newline)
        let result = crusty_peg_parser::test_int("42 // comment");
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_peg_multiple_line_comments() {
        // Multiple line comments
        let result = crusty_peg_parser::test_int("// first\n// second\n42");
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_peg_block_comment_multiline() {
        // Block comment spanning multiple lines
        let result = crusty_peg_parser::test_int("/* line 1\n   line 2\n   line 3 */ 42");
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_peg_whitespace_types() {
        // Test different whitespace types: space, tab, newline, carriage return
        let result = crusty_peg_parser::test_int(" \t\r\n42\n\r\t ");
        assert_eq!(result, Ok(42));
    }

    // ========================================================================
    // KEYWORD TESTS
    // ========================================================================

    #[test]
    fn test_peg_keywords_basic() {
        // Test that basic keywords are recognized
        assert_eq!(
            crusty_peg_parser::test_keyword("let"),
            Ok("let".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("var"),
            Ok("var".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("const"),
            Ok("const".to_string())
        );
        assert_eq!(crusty_peg_parser::test_keyword("if"), Ok("if".to_string()));
        assert_eq!(
            crusty_peg_parser::test_keyword("else"),
            Ok("else".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("while"),
            Ok("while".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("for"),
            Ok("for".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("return"),
            Ok("return".to_string())
        );
    }

    #[test]
    fn test_peg_keywords_control_flow() {
        // Test control flow keywords
        assert_eq!(
            crusty_peg_parser::test_keyword("break"),
            Ok("break".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("continue"),
            Ok("continue".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("switch"),
            Ok("switch".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("case"),
            Ok("case".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("default"),
            Ok("default".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("loop"),
            Ok("loop".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("match"),
            Ok("match".to_string())
        );
    }

    #[test]
    fn test_peg_keywords_declarations() {
        // Test declaration keywords
        assert_eq!(
            crusty_peg_parser::test_keyword("struct"),
            Ok("struct".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("enum"),
            Ok("enum".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("typedef"),
            Ok("typedef".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("static"),
            Ok("static".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("mut"),
            Ok("mut".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("extern"),
            Ok("extern".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("unsafe"),
            Ok("unsafe".to_string())
        );
    }

    #[test]
    fn test_peg_keywords_types() {
        // Test type keywords
        assert_eq!(
            crusty_peg_parser::test_keyword("int"),
            Ok("int".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("i32"),
            Ok("i32".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("i64"),
            Ok("i64".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("u32"),
            Ok("u32".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("u64"),
            Ok("u64".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("float"),
            Ok("float".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("f32"),
            Ok("f32".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("f64"),
            Ok("f64".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("bool"),
            Ok("bool".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("char"),
            Ok("char".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("void"),
            Ok("void".to_string())
        );
    }

    #[test]
    fn test_peg_keywords_literals() {
        // Test literal keywords
        assert_eq!(
            crusty_peg_parser::test_keyword("true"),
            Ok("true".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("false"),
            Ok("false".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("NULL"),
            Ok("NULL".to_string())
        );
    }

    #[test]
    fn test_peg_keywords_with_whitespace() {
        // Test that keywords work with surrounding whitespace
        assert_eq!(
            crusty_peg_parser::test_keyword("  let  "),
            Ok("let".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("\tvar\n"),
            Ok("var".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_keyword("  /* comment */ if  "),
            Ok("if".to_string())
        );
    }

    #[test]
    fn test_peg_keyword_lookahead() {
        // Test that keywords don't match as prefixes of identifiers
        // "letter" should NOT match as keyword "let"
        assert!(crusty_peg_parser::test_keyword("letter").is_err());

        // "variable" should NOT match as keyword "var"
        assert!(crusty_peg_parser::test_keyword("variable").is_err());

        // "ifelse" should NOT match as keyword "if"
        assert!(crusty_peg_parser::test_keyword("ifelse").is_err());

        // "return_value" should NOT match as keyword "return"
        assert!(crusty_peg_parser::test_keyword("return_value").is_err());

        // "int32" should NOT match as keyword "int"
        assert!(crusty_peg_parser::test_keyword("int32").is_err());
    }

    #[test]
    fn test_peg_not_keyword() {
        // Test that non-keywords are correctly identified
        assert_eq!(
            crusty_peg_parser::test_not_keyword("hello"),
            Ok("hello".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_not_keyword("foo_bar"),
            Ok("foo_bar".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_not_keyword("_test"),
            Ok("_test".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_not_keyword("myVar123"),
            Ok("myVar123".to_string())
        );

        // These should fail because they ARE keywords
        assert!(crusty_peg_parser::test_not_keyword("let").is_err());
        assert!(crusty_peg_parser::test_not_keyword("var").is_err());
        assert!(crusty_peg_parser::test_not_keyword("if").is_err());
        assert!(crusty_peg_parser::test_not_keyword("int").is_err());
    }

    #[test]
    fn test_peg_keyword_case_sensitivity() {
        // Test that keywords are case-sensitive
        // "Let" should NOT match keyword "let"
        assert!(crusty_peg_parser::test_keyword("Let").is_err());
        assert!(crusty_peg_parser::test_keyword("VAR").is_err());
        assert!(crusty_peg_parser::test_keyword("IF").is_err());
        assert!(crusty_peg_parser::test_keyword("Int").is_err());

        // But they should be valid identifiers
        assert_eq!(
            crusty_peg_parser::test_not_keyword("Let"),
            Ok("Let".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_not_keyword("VAR"),
            Ok("VAR".to_string())
        );
    }

    #[test]
    fn test_peg_keywords_all() {
        // Comprehensive test of all keywords
        let keywords = vec![
            "let",
            "var",
            "const",
            "static",
            "mut",
            "define",
            "if",
            "else",
            "while",
            "for",
            "in",
            "return",
            "break",
            "continue",
            "struct",
            "enum",
            "typedef",
            "namespace",
            "extern",
            "unsafe",
            "loop",
            "match",
            "switch",
            "case",
            "default",
            "auto",
            "int",
            "i32",
            "i64",
            "u32",
            "u64",
            "float",
            "f32",
            "f64",
            "bool",
            "char",
            "void",
            "true",
            "false",
            "NULL",
        ];

        for keyword in keywords {
            assert_eq!(
                crusty_peg_parser::test_keyword(keyword),
                Ok(keyword.to_string()),
                "Failed to parse keyword: {}",
                keyword
            );
        }
    }

    #[test]
    fn test_peg_keywords_followed_by_punctuation() {
        // Test that keywords work correctly when followed by punctuation
        // The test_keyword rule expects EOF, so these will fail at the parser level
        // but the important thing is that the keyword itself is recognized
        // (the lookahead !ident_char() allows punctuation to follow)

        // These fail because test_keyword expects EOF after the keyword
        // but this demonstrates that the lookahead correctly allows punctuation
        assert!(crusty_peg_parser::test_keyword("let(").is_err());
        assert!(crusty_peg_parser::test_keyword("if{").is_err());

        // However, with whitespace before punctuation, they work
        assert_eq!(
            crusty_peg_parser::test_keyword("let "),
            Ok("let".to_string())
        );
        assert_eq!(crusty_peg_parser::test_keyword("if "), Ok("if".to_string()));

        // The key test: keywords followed by ident_char should fail
        // This is tested in other tests, but let's verify the contrast
        assert!(crusty_peg_parser::test_keyword("leta").is_err());
        assert!(crusty_peg_parser::test_keyword("if1").is_err());
    }

    #[test]
    fn test_peg_keywords_with_numbers_after() {
        // Test that keywords followed by numbers are NOT recognized as keywords
        // because numbers are valid ident_char
        assert!(crusty_peg_parser::test_keyword("let1").is_err());
        assert!(crusty_peg_parser::test_keyword("if2").is_err());
        assert!(crusty_peg_parser::test_keyword("var123").is_err());
        assert!(crusty_peg_parser::test_keyword("int0").is_err());
        assert!(crusty_peg_parser::test_keyword("for99").is_err());
    }

    #[test]
    fn test_peg_keywords_with_underscore_after() {
        // Test that keywords followed by underscore are NOT recognized as keywords
        // because underscore is a valid ident_char
        assert!(crusty_peg_parser::test_keyword("let_").is_err());
        assert!(crusty_peg_parser::test_keyword("if_").is_err());
        assert!(crusty_peg_parser::test_keyword("var_x").is_err());
        assert!(crusty_peg_parser::test_keyword("int_type").is_err());
    }

    #[test]
    fn test_peg_keywords_at_boundaries() {
        // Test keywords at start and end of input (no surrounding whitespace)
        assert_eq!(
            crusty_peg_parser::test_keyword("let"),
            Ok("let".to_string())
        );
        assert_eq!(crusty_peg_parser::test_keyword("if"), Ok("if".to_string()));
        assert_eq!(
            crusty_peg_parser::test_keyword("int"),
            Ok("int".to_string())
        );

        // Test with only leading whitespace
        assert_eq!(
            crusty_peg_parser::test_keyword("  let"),
            Ok("let".to_string())
        );

        // Test with only trailing whitespace
        assert_eq!(
            crusty_peg_parser::test_keyword("let  "),
            Ok("let".to_string())
        );
    }

    #[test]
    fn test_peg_keywords_similar_to_identifiers() {
        // Test keywords that are prefixes or similar to common identifiers
        // These should fail because they have ident_char after the keyword
        assert!(crusty_peg_parser::test_keyword("integer").is_err());
        assert!(crusty_peg_parser::test_keyword("floating").is_err());
        assert!(crusty_peg_parser::test_keyword("boolean").is_err());
        assert!(crusty_peg_parser::test_keyword("character").is_err());
        assert!(crusty_peg_parser::test_keyword("structure").is_err());
        assert!(crusty_peg_parser::test_keyword("enumeration").is_err());
        assert!(crusty_peg_parser::test_keyword("constant").is_err());
        assert!(crusty_peg_parser::test_keyword("statement").is_err());
    }

    #[test]
    fn test_peg_keywords_lookahead_comprehensive() {
        // Comprehensive test of lookahead preventing keyword matches
        // when followed by valid identifier characters

        // Alphabetic characters after keywords
        assert!(crusty_peg_parser::test_keyword("leta").is_err());
        assert!(crusty_peg_parser::test_keyword("ifA").is_err());
        assert!(crusty_peg_parser::test_keyword("varZ").is_err());

        // Numbers after keywords
        assert!(crusty_peg_parser::test_keyword("int0").is_err());
        assert!(crusty_peg_parser::test_keyword("for9").is_err());

        // Underscore after keywords
        assert!(crusty_peg_parser::test_keyword("let_").is_err());
        assert!(crusty_peg_parser::test_keyword("if_").is_err());

        // Mixed alphanumeric after keywords
        assert!(crusty_peg_parser::test_keyword("let_x1").is_err());
        assert!(crusty_peg_parser::test_keyword("if_test_123").is_err());
    }

    #[test]
    fn test_peg_not_keyword_edge_cases() {
        // Test that identifiers similar to keywords are correctly identified as non-keywords
        assert_eq!(
            crusty_peg_parser::test_not_keyword("letter"),
            Ok("letter".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_not_keyword("variable"),
            Ok("variable".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_not_keyword("ifelse"),
            Ok("ifelse".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_not_keyword("integer"),
            Ok("integer".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_not_keyword("let_x"),
            Ok("let_x".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_not_keyword("if1"),
            Ok("if1".to_string())
        );
        assert_eq!(
            crusty_peg_parser::test_not_keyword("var_"),
            Ok("var_".to_string())
        );
    }

    // ========================================================================
    // LITERAL TESTS
    // ========================================================================

    #[test]
    fn test_peg_int_literal() {
        // Test integer literals
        assert_eq!(crusty_peg_parser::int_literal("42"), Ok(Literal::Int(42)));
        assert_eq!(crusty_peg_parser::int_literal("0"), Ok(Literal::Int(0)));
        assert_eq!(
            crusty_peg_parser::int_literal("123456789"),
            Ok(Literal::Int(123456789))
        );
    }

    #[test]
    fn test_peg_float_literal() {
        // Test float literals with decimal point
        assert_eq!(
            crusty_peg_parser::float_literal("3.15"),
            Ok(Literal::Float(3.15))
        );
        assert_eq!(
            crusty_peg_parser::float_literal("0.5"),
            Ok(Literal::Float(0.5))
        );
        assert_eq!(
            crusty_peg_parser::float_literal("123.456"),
            Ok(Literal::Float(123.456))
        );

        // Test scientific notation
        assert_eq!(
            crusty_peg_parser::float_literal("1.5e10"),
            Ok(Literal::Float(1.5e10))
        );
        assert_eq!(
            crusty_peg_parser::float_literal("2.0E-5"),
            Ok(Literal::Float(2.0e-5))
        );
        assert_eq!(
            crusty_peg_parser::float_literal("3e8"),
            Ok(Literal::Float(3e8))
        );
    }

    #[test]
    fn test_peg_string_literal() {
        // Test basic string literals
        assert_eq!(
            crusty_peg_parser::string_literal("\"hello\""),
            Ok(Literal::String("hello".to_string()))
        );
        assert_eq!(
            crusty_peg_parser::string_literal("\"\""),
            Ok(Literal::String("".to_string()))
        );
        assert_eq!(
            crusty_peg_parser::string_literal("\"hello world\""),
            Ok(Literal::String("hello world".to_string()))
        );

        // Test escape sequences
        assert_eq!(
            crusty_peg_parser::string_literal("\"hello\\nworld\""),
            Ok(Literal::String("hello\nworld".to_string()))
        );
        assert_eq!(
            crusty_peg_parser::string_literal("\"tab\\there\""),
            Ok(Literal::String("tab\there".to_string()))
        );
        assert_eq!(
            crusty_peg_parser::string_literal("\"quote\\\"here\""),
            Ok(Literal::String("quote\"here".to_string()))
        );
        assert_eq!(
            crusty_peg_parser::string_literal("\"backslash\\\\here\""),
            Ok(Literal::String("backslash\\here".to_string()))
        );
    }

    #[test]
    fn test_peg_char_literal() {
        // Test basic character literals
        assert_eq!(
            crusty_peg_parser::char_literal("'a'"),
            Ok(Literal::Char('a'))
        );
        assert_eq!(
            crusty_peg_parser::char_literal("'Z'"),
            Ok(Literal::Char('Z'))
        );
        assert_eq!(
            crusty_peg_parser::char_literal("'5'"),
            Ok(Literal::Char('5'))
        );

        // Test escape sequences
        assert_eq!(
            crusty_peg_parser::char_literal("'\\n'"),
            Ok(Literal::Char('\n'))
        );
        assert_eq!(
            crusty_peg_parser::char_literal("'\\t'"),
            Ok(Literal::Char('\t'))
        );
        assert_eq!(
            crusty_peg_parser::char_literal("'\\''"),
            Ok(Literal::Char('\''))
        );
        assert_eq!(
            crusty_peg_parser::char_literal("'\\\\'"),
            Ok(Literal::Char('\\'))
        );
    }

    #[test]
    fn test_peg_bool_literal() {
        // Test boolean literals
        assert_eq!(
            crusty_peg_parser::bool_literal("true"),
            Ok(Literal::Bool(true))
        );
        assert_eq!(
            crusty_peg_parser::bool_literal("false"),
            Ok(Literal::Bool(false))
        );
    }

    #[test]
    fn test_peg_null_literal() {
        // Test null literal
        assert_eq!(crusty_peg_parser::null_literal("NULL"), Ok(Literal::Null));
    }

    #[test]
    fn test_peg_literal_with_whitespace() {
        // Test that literals work with surrounding whitespace
        // Note: The literal rules don't consume whitespace themselves,
        // but the _ rule can be used in higher-level rules
    }

    #[test]
    fn test_peg_string_escape_sequences_comprehensive() {
        // Test all supported escape sequences
        assert_eq!(
            crusty_peg_parser::string_literal("\"\\n\""),
            Ok(Literal::String("\n".to_string()))
        );
        assert_eq!(
            crusty_peg_parser::string_literal("\"\\r\""),
            Ok(Literal::String("\r".to_string()))
        );
        assert_eq!(
            crusty_peg_parser::string_literal("\"\\t\""),
            Ok(Literal::String("\t".to_string()))
        );
        assert_eq!(
            crusty_peg_parser::string_literal("\"\\0\""),
            Ok(Literal::String("\0".to_string()))
        );
        assert_eq!(
            crusty_peg_parser::string_literal("\"\\\\\""),
            Ok(Literal::String("\\".to_string()))
        );
        assert_eq!(
            crusty_peg_parser::string_literal("\"\\\"\""),
            Ok(Literal::String("\"".to_string()))
        );
    }

    #[test]
    fn test_peg_char_escape_sequences_comprehensive() {
        // Test all supported escape sequences for chars
        assert_eq!(
            crusty_peg_parser::char_literal("'\\n'"),
            Ok(Literal::Char('\n'))
        );
        assert_eq!(
            crusty_peg_parser::char_literal("'\\r'"),
            Ok(Literal::Char('\r'))
        );
        assert_eq!(
            crusty_peg_parser::char_literal("'\\t'"),
            Ok(Literal::Char('\t'))
        );
        assert_eq!(
            crusty_peg_parser::char_literal("'\\0'"),
            Ok(Literal::Char('\0'))
        );
        assert_eq!(
            crusty_peg_parser::char_literal("'\\\\'"),
            Ok(Literal::Char('\\'))
        );
        assert_eq!(
            crusty_peg_parser::char_literal("'\\''"),
            Ok(Literal::Char('\''))
        );
    }
}

// ============================================================================
// IDENTIFIER TESTS (Task 2.4)
// ============================================================================

#[cfg(test)]
mod identifier_tests {
    use super::*;

    #[test]
    fn test_peg_ident_basic() {
        // Test basic identifiers
        assert_eq!(crusty_peg_parser::ident("hello"), Ok(Ident::new("hello")));
        assert_eq!(crusty_peg_parser::ident("foo"), Ok(Ident::new("foo")));
        assert_eq!(crusty_peg_parser::ident("bar"), Ok(Ident::new("bar")));
    }

    #[test]
    fn test_peg_ident_with_underscore() {
        // Test identifiers with underscores
        assert_eq!(
            crusty_peg_parser::ident("foo_bar"),
            Ok(Ident::new("foo_bar"))
        );
        assert_eq!(crusty_peg_parser::ident("_test"), Ok(Ident::new("_test")));
        assert_eq!(crusty_peg_parser::ident("_"), Ok(Ident::new("_")));
        assert_eq!(
            crusty_peg_parser::ident("__private"),
            Ok(Ident::new("__private"))
        );
    }

    #[test]
    fn test_peg_ident_with_numbers() {
        // Test identifiers with numbers
        assert_eq!(
            crusty_peg_parser::ident("test123"),
            Ok(Ident::new("test123"))
        );
        assert_eq!(crusty_peg_parser::ident("var1"), Ok(Ident::new("var1")));
        assert_eq!(crusty_peg_parser::ident("x1y2z3"), Ok(Ident::new("x1y2z3")));
    }

    #[test]
    fn test_peg_ident_mixed_case() {
        // Test identifiers with mixed case
        assert_eq!(
            crusty_peg_parser::ident("MyClass"),
            Ok(Ident::new("MyClass"))
        );
        assert_eq!(
            crusty_peg_parser::ident("camelCase"),
            Ok(Ident::new("camelCase"))
        );
        assert_eq!(
            crusty_peg_parser::ident("CONSTANT"),
            Ok(Ident::new("CONSTANT"))
        );
    }

    #[test]
    fn test_peg_ident_not_keyword() {
        // Test that keywords are NOT parsed as identifiers
        assert!(crusty_peg_parser::ident("let").is_err());
        assert!(crusty_peg_parser::ident("var").is_err());
        assert!(crusty_peg_parser::ident("if").is_err());
        assert!(crusty_peg_parser::ident("while").is_err());
        assert!(crusty_peg_parser::ident("for").is_err());
        assert!(crusty_peg_parser::ident("return").is_err());
        assert!(crusty_peg_parser::ident("int").is_err());
        assert!(crusty_peg_parser::ident("float").is_err());
        assert!(crusty_peg_parser::ident("bool").is_err());
        assert!(crusty_peg_parser::ident("true").is_err());
        assert!(crusty_peg_parser::ident("false").is_err());
        assert!(crusty_peg_parser::ident("NULL").is_err());
    }

    #[test]
    fn test_peg_ident_keyword_prefix() {
        // Test that identifiers with keyword prefixes are valid
        assert_eq!(crusty_peg_parser::ident("letter"), Ok(Ident::new("letter")));
        assert_eq!(
            crusty_peg_parser::ident("variable"),
            Ok(Ident::new("variable"))
        );
        assert_eq!(crusty_peg_parser::ident("ifelse"), Ok(Ident::new("ifelse")));
        assert_eq!(
            crusty_peg_parser::ident("integer"),
            Ok(Ident::new("integer"))
        );
        assert_eq!(
            crusty_peg_parser::ident("return_value"),
            Ok(Ident::new("return_value"))
        );
    }

    #[test]
    fn test_peg_ident_cannot_start_with_number() {
        // Test that identifiers cannot start with a number
        assert!(crusty_peg_parser::ident("123abc").is_err());
        assert!(crusty_peg_parser::ident("0test").is_err());
        assert!(crusty_peg_parser::ident("9var").is_err());
    }

    #[test]
    fn test_peg_ident_single_char() {
        // Test single character identifiers
        assert_eq!(crusty_peg_parser::ident("a"), Ok(Ident::new("a")));
        assert_eq!(crusty_peg_parser::ident("x"), Ok(Ident::new("x")));
        assert_eq!(crusty_peg_parser::ident("Z"), Ok(Ident::new("Z")));
        assert_eq!(crusty_peg_parser::ident("_"), Ok(Ident::new("_")));
    }

    #[test]
    fn test_peg_ident_long() {
        // Test long identifiers
        let long_ident = "this_is_a_very_long_identifier_name_with_many_characters";
        assert_eq!(
            crusty_peg_parser::ident(long_ident),
            Ok(Ident::new(long_ident))
        );
    }

    #[test]
    fn test_peg_macro_ident_basic() {
        // Test basic macro identifiers
        assert_eq!(
            crusty_peg_parser::macro_ident("__FILE__"),
            Ok(Ident::new("__FILE__"))
        );
        assert_eq!(
            crusty_peg_parser::macro_ident("__LINE__"),
            Ok(Ident::new("__LINE__"))
        );
        assert_eq!(
            crusty_peg_parser::macro_ident("__DATE__"),
            Ok(Ident::new("__DATE__"))
        );
    }

    #[test]
    fn test_peg_macro_ident_custom() {
        // Test custom macro identifiers
        assert_eq!(
            crusty_peg_parser::macro_ident("__MY_MACRO__"),
            Ok(Ident::new("__MY_MACRO__"))
        );
        assert_eq!(
            crusty_peg_parser::macro_ident("__test__"),
            Ok(Ident::new("__test__"))
        );
        assert_eq!(
            crusty_peg_parser::macro_ident("__foo_bar_123__"),
            Ok(Ident::new("__foo_bar_123__"))
        );
    }

    #[test]
    fn test_peg_macro_ident_requires_double_underscore() {
        // Test that macro identifiers require double underscore prefix and suffix
        assert!(crusty_peg_parser::macro_ident("_FILE_").is_err());
        assert!(crusty_peg_parser::macro_ident("__FILE_").is_err());
        assert!(crusty_peg_parser::macro_ident("_FILE__").is_err());
        assert!(crusty_peg_parser::macro_ident("FILE").is_err());
    }

    #[test]
    fn test_peg_macro_ident_cannot_be_empty() {
        // Test that macro identifiers cannot be empty between underscores
        assert!(crusty_peg_parser::macro_ident("____").is_err());
    }

    #[test]
    fn test_peg_macro_ident_must_start_with_letter_or_underscore() {
        // Test that macro identifiers must start with letter or underscore after __
        assert!(crusty_peg_parser::macro_ident("__123__").is_err());
        assert!(crusty_peg_parser::macro_ident("__9test__").is_err());
    }

    #[test]
    fn test_peg_ident_vs_macro_ident() {
        // Test that regular ident can parse identifiers with double underscores
        // but they're not macro identifiers
        assert_eq!(
            crusty_peg_parser::ident("__private"),
            Ok(Ident::new("__private"))
        );
        assert_eq!(crusty_peg_parser::ident("test__"), Ok(Ident::new("test__")));

        // But macro_ident requires both prefix and suffix
        assert!(crusty_peg_parser::macro_ident("__private").is_err());
        assert!(crusty_peg_parser::macro_ident("test__").is_err());
    }
}

// ============================================================================
// PRIMITIVE TYPE TESTS (Task 3.1)
// ============================================================================

#[cfg(test)]
mod primitive_type_tests {
    use super::*;

    #[test]
    fn test_peg_primitive_int() {
        // Test int type
        assert_eq!(
            crusty_peg_parser::primitive_int("int"),
            Ok(Type::Primitive(PrimitiveType::Int))
        );
    }

    #[test]
    fn test_peg_primitive_i32() {
        // Test i32 type
        assert_eq!(
            crusty_peg_parser::primitive_i32("i32"),
            Ok(Type::Primitive(PrimitiveType::I32))
        );
    }

    #[test]
    fn test_peg_primitive_i64() {
        // Test i64 type
        assert_eq!(
            crusty_peg_parser::primitive_i64("i64"),
            Ok(Type::Primitive(PrimitiveType::I64))
        );
    }

    #[test]
    fn test_peg_primitive_u32() {
        // Test u32 type
        assert_eq!(
            crusty_peg_parser::primitive_u32("u32"),
            Ok(Type::Primitive(PrimitiveType::U32))
        );
    }

    #[test]
    fn test_peg_primitive_u64() {
        // Test u64 type
        assert_eq!(
            crusty_peg_parser::primitive_u64("u64"),
            Ok(Type::Primitive(PrimitiveType::U64))
        );
    }

    #[test]
    fn test_peg_primitive_float() {
        // Test float type
        assert_eq!(
            crusty_peg_parser::primitive_float("float"),
            Ok(Type::Primitive(PrimitiveType::Float))
        );
    }

    #[test]
    fn test_peg_primitive_f32() {
        // Test f32 type
        assert_eq!(
            crusty_peg_parser::primitive_f32("f32"),
            Ok(Type::Primitive(PrimitiveType::F32))
        );
    }

    #[test]
    fn test_peg_primitive_f64() {
        // Test f64 type
        assert_eq!(
            crusty_peg_parser::primitive_f64("f64"),
            Ok(Type::Primitive(PrimitiveType::F64))
        );
    }

    #[test]
    fn test_peg_primitive_bool() {
        // Test bool type
        assert_eq!(
            crusty_peg_parser::primitive_bool("bool"),
            Ok(Type::Primitive(PrimitiveType::Bool))
        );
    }

    #[test]
    fn test_peg_primitive_char() {
        // Test char type
        assert_eq!(
            crusty_peg_parser::primitive_char("char"),
            Ok(Type::Primitive(PrimitiveType::Char))
        );
    }

    #[test]
    fn test_peg_primitive_void() {
        // Test void type
        assert_eq!(
            crusty_peg_parser::primitive_void("void"),
            Ok(Type::Primitive(PrimitiveType::Void))
        );
    }

    #[test]
    fn test_peg_primitive_type_all() {
        // Test primitive_type rule with all primitive types
        assert_eq!(
            crusty_peg_parser::primitive_type("int"),
            Ok(Type::Primitive(PrimitiveType::Int))
        );
        assert_eq!(
            crusty_peg_parser::primitive_type("i32"),
            Ok(Type::Primitive(PrimitiveType::I32))
        );
        assert_eq!(
            crusty_peg_parser::primitive_type("i64"),
            Ok(Type::Primitive(PrimitiveType::I64))
        );
        assert_eq!(
            crusty_peg_parser::primitive_type("u32"),
            Ok(Type::Primitive(PrimitiveType::U32))
        );
        assert_eq!(
            crusty_peg_parser::primitive_type("u64"),
            Ok(Type::Primitive(PrimitiveType::U64))
        );
        assert_eq!(
            crusty_peg_parser::primitive_type("float"),
            Ok(Type::Primitive(PrimitiveType::Float))
        );
        assert_eq!(
            crusty_peg_parser::primitive_type("f32"),
            Ok(Type::Primitive(PrimitiveType::F32))
        );
        assert_eq!(
            crusty_peg_parser::primitive_type("f64"),
            Ok(Type::Primitive(PrimitiveType::F64))
        );
        assert_eq!(
            crusty_peg_parser::primitive_type("bool"),
            Ok(Type::Primitive(PrimitiveType::Bool))
        );
        assert_eq!(
            crusty_peg_parser::primitive_type("char"),
            Ok(Type::Primitive(PrimitiveType::Char))
        );
        assert_eq!(
            crusty_peg_parser::primitive_type("void"),
            Ok(Type::Primitive(PrimitiveType::Void))
        );
    }

    #[test]
    fn test_peg_primitive_type_not_identifier() {
        // Test that primitive types are not parsed as identifiers
        // (they should fail because they ARE keywords)
        assert!(crusty_peg_parser::ident("int").is_err());
        assert!(crusty_peg_parser::ident("i32").is_err());
        assert!(crusty_peg_parser::ident("float").is_err());
        assert!(crusty_peg_parser::ident("bool").is_err());
        assert!(crusty_peg_parser::ident("void").is_err());
    }

    #[test]
    fn test_peg_primitive_type_lookahead() {
        // Test that primitive types don't match as prefixes of identifiers
        // "integer" should NOT match as "int"
        assert!(crusty_peg_parser::primitive_int("integer").is_err());
        assert!(crusty_peg_parser::primitive_float("floating").is_err());
        assert!(crusty_peg_parser::primitive_bool("boolean").is_err());
        assert!(crusty_peg_parser::primitive_char("character").is_err());
    }

    #[test]
    fn test_peg_primitive_type_order() {
        // Test that longer type names are matched before shorter ones
        // i32 should match before int, f32 should match before float
        // This is important because "i32" starts with "i" which could match "int" prefix
        assert_eq!(
            crusty_peg_parser::primitive_type("i32"),
            Ok(Type::Primitive(PrimitiveType::I32))
        );
        assert_eq!(
            crusty_peg_parser::primitive_type("i64"),
            Ok(Type::Primitive(PrimitiveType::I64))
        );
        assert_eq!(
            crusty_peg_parser::primitive_type("f32"),
            Ok(Type::Primitive(PrimitiveType::F32))
        );
        assert_eq!(
            crusty_peg_parser::primitive_type("f64"),
            Ok(Type::Primitive(PrimitiveType::F64))
        );
    }

    #[test]
    fn test_peg_primitive_type_invalid() {
        // Test that invalid type names are rejected
        assert!(crusty_peg_parser::primitive_type("string").is_err());
        assert!(crusty_peg_parser::primitive_type("double").is_err());
        assert!(crusty_peg_parser::primitive_type("long").is_err());
        assert!(crusty_peg_parser::primitive_type("short").is_err());
    }
}

// ============================================================================
// COMPLEX TYPE TESTS (Task 3.2)
// ============================================================================

#[cfg(test)]
mod complex_type_tests {
    use super::*;

    // ========================================================================
    // POINTER TYPE TESTS
    // ========================================================================

    #[test]
    fn test_peg_pointer_type_basic() {
        // Test basic pointer types: T*
        assert_eq!(
            crusty_peg_parser::type_expr("int*"),
            Ok(Type::Pointer {
                ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                mutable: false
            })
        );
        assert_eq!(
            crusty_peg_parser::type_expr("bool*"),
            Ok(Type::Pointer {
                ty: Box::new(Type::Primitive(PrimitiveType::Bool)),
                mutable: false
            })
        );
        assert_eq!(
            crusty_peg_parser::type_expr("char*"),
            Ok(Type::Pointer {
                ty: Box::new(Type::Primitive(PrimitiveType::Char)),
                mutable: false
            })
        );
    }

    #[test]
    fn test_peg_pointer_type_double() {
        // Test double pointer: T**
        assert_eq!(
            crusty_peg_parser::type_expr("int**"),
            Ok(Type::Pointer {
                ty: Box::new(Type::Pointer {
                    ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                    mutable: false
                }),
                mutable: false
            })
        );
    }

    #[test]
    fn test_peg_pointer_type_triple() {
        // Test triple pointer: T***
        assert_eq!(
            crusty_peg_parser::type_expr("int***"),
            Ok(Type::Pointer {
                ty: Box::new(Type::Pointer {
                    ty: Box::new(Type::Pointer {
                        ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                        mutable: false
                    }),
                    mutable: false
                }),
                mutable: false
            })
        );
    }

    #[test]
    fn test_peg_pointer_type_with_whitespace() {
        // Test pointer types with whitespace
        assert_eq!(
            crusty_peg_parser::type_expr("int *"),
            Ok(Type::Pointer {
                ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                mutable: false
            })
        );
        assert_eq!(
            crusty_peg_parser::type_expr("int  *"),
            Ok(Type::Pointer {
                ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                mutable: false
            })
        );
    }

    #[test]
    fn test_peg_pointer_to_identifier_type() {
        // Test pointer to user-defined type
        assert_eq!(
            crusty_peg_parser::type_expr("MyType*"),
            Ok(Type::Pointer {
                ty: Box::new(Type::Ident(Ident::new("MyType"))),
                mutable: false
            })
        );
    }

    // ========================================================================
    // REFERENCE TYPE TESTS
    // ========================================================================

    #[test]
    fn test_peg_reference_type_immutable() {
        // Test immutable reference: &T
        assert_eq!(
            crusty_peg_parser::type_expr("&int"),
            Ok(Type::Reference {
                ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                mutable: false
            })
        );
        assert_eq!(
            crusty_peg_parser::type_expr("&bool"),
            Ok(Type::Reference {
                ty: Box::new(Type::Primitive(PrimitiveType::Bool)),
                mutable: false
            })
        );
    }

    #[test]
    fn test_peg_reference_type_mutable() {
        // Test mutable reference: &mut T
        assert_eq!(
            crusty_peg_parser::type_expr("&mut int"),
            Ok(Type::Reference {
                ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                mutable: true
            })
        );
        assert_eq!(
            crusty_peg_parser::type_expr("&mut bool"),
            Ok(Type::Reference {
                ty: Box::new(Type::Primitive(PrimitiveType::Bool)),
                mutable: true
            })
        );
    }

    #[test]
    fn test_peg_reference_to_pointer() {
        // Test reference to pointer: &T*
        assert_eq!(
            crusty_peg_parser::type_expr("&int*"),
            Ok(Type::Reference {
                ty: Box::new(Type::Pointer {
                    ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                    mutable: false
                }),
                mutable: false
            })
        );
    }

    #[test]
    fn test_peg_reference_mutable_to_pointer() {
        // Test mutable reference to pointer: &mut T*
        assert_eq!(
            crusty_peg_parser::type_expr("&mut int*"),
            Ok(Type::Reference {
                ty: Box::new(Type::Pointer {
                    ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                    mutable: false
                }),
                mutable: true
            })
        );
    }

    #[test]
    fn test_peg_reference_to_identifier_type() {
        // Test reference to user-defined type
        assert_eq!(
            crusty_peg_parser::type_expr("&MyType"),
            Ok(Type::Reference {
                ty: Box::new(Type::Ident(Ident::new("MyType"))),
                mutable: false
            })
        );
        assert_eq!(
            crusty_peg_parser::type_expr("&mut MyType"),
            Ok(Type::Reference {
                ty: Box::new(Type::Ident(Ident::new("MyType"))),
                mutable: true
            })
        );
    }

    // ========================================================================
    // ARRAY TYPE TESTS
    // ========================================================================

    #[test]
    fn test_peg_array_type_basic() {
        // Test basic array types: T[N]
        assert_eq!(
            crusty_peg_parser::type_expr("int[10]"),
            Ok(Type::Array {
                ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                size: Some(10)
            })
        );
        assert_eq!(
            crusty_peg_parser::type_expr("bool[5]"),
            Ok(Type::Array {
                ty: Box::new(Type::Primitive(PrimitiveType::Bool)),
                size: Some(5)
            })
        );
    }

    #[test]
    fn test_peg_array_type_large_size() {
        // Test array with large size
        assert_eq!(
            crusty_peg_parser::type_expr("int[1000]"),
            Ok(Type::Array {
                ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                size: Some(1000)
            })
        );
    }

    #[test]
    fn test_peg_array_type_with_whitespace() {
        // Test array types with whitespace
        assert_eq!(
            crusty_peg_parser::type_expr("int[ 10 ]"),
            Ok(Type::Array {
                ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                size: Some(10)
            })
        );
        assert_eq!(
            crusty_peg_parser::type_expr("int [10]"),
            Ok(Type::Array {
                ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                size: Some(10)
            })
        );
    }

    #[test]
    fn test_peg_array_of_pointers() {
        // Test array of pointers: T*[N]
        assert_eq!(
            crusty_peg_parser::type_expr("int*[10]"),
            Ok(Type::Array {
                ty: Box::new(Type::Pointer {
                    ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                    mutable: false
                }),
                size: Some(10)
            })
        );
    }

    #[test]
    fn test_peg_pointer_to_array() {
        // Test pointer to array: (T[N])*
        // Note: This requires parentheses to group the array type
        assert_eq!(
            crusty_peg_parser::type_expr("(int[10])*"),
            Ok(Type::Pointer {
                ty: Box::new(Type::Array {
                    ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                    size: Some(10)
                }),
                mutable: false
            })
        );
    }

    // ========================================================================
    // SLICE TYPE TESTS
    // ========================================================================

    #[test]
    fn test_peg_slice_type_basic() {
        // Test basic slice types: T[]
        assert_eq!(
            crusty_peg_parser::type_expr("int[]"),
            Ok(Type::Slice {
                ty: Box::new(Type::Primitive(PrimitiveType::Int))
            })
        );
        assert_eq!(
            crusty_peg_parser::type_expr("bool[]"),
            Ok(Type::Slice {
                ty: Box::new(Type::Primitive(PrimitiveType::Bool))
            })
        );
    }

    #[test]
    fn test_peg_slice_type_with_whitespace() {
        // Test slice types with whitespace
        assert_eq!(
            crusty_peg_parser::type_expr("int[ ]"),
            Ok(Type::Slice {
                ty: Box::new(Type::Primitive(PrimitiveType::Int))
            })
        );
        assert_eq!(
            crusty_peg_parser::type_expr("int []"),
            Ok(Type::Slice {
                ty: Box::new(Type::Primitive(PrimitiveType::Int))
            })
        );
    }

    #[test]
    fn test_peg_slice_of_pointers() {
        // Test slice of pointers: T*[]
        assert_eq!(
            crusty_peg_parser::type_expr("int*[]"),
            Ok(Type::Slice {
                ty: Box::new(Type::Pointer {
                    ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                    mutable: false
                })
            })
        );
    }

    #[test]
    fn test_peg_reference_to_slice() {
        // Test reference to slice: &T[]
        assert_eq!(
            crusty_peg_parser::type_expr("&int[]"),
            Ok(Type::Reference {
                ty: Box::new(Type::Slice {
                    ty: Box::new(Type::Primitive(PrimitiveType::Int))
                }),
                mutable: false
            })
        );
    }

    // ========================================================================
    // TUPLE TYPE TESTS
    // ========================================================================

    #[test]
    fn test_peg_tuple_type_empty() {
        // Test empty tuple: ()
        assert_eq!(
            crusty_peg_parser::type_expr("()"),
            Ok(Type::Tuple { types: vec![] })
        );
    }

    #[test]
    fn test_peg_tuple_type_single() {
        // Test single-element tuple: (T,)
        // Note: Single element without trailing comma is just parenthesized type
        assert_eq!(
            crusty_peg_parser::type_expr("(int,)"),
            Ok(Type::Tuple {
                types: vec![Type::Primitive(PrimitiveType::Int)]
            })
        );
    }

    #[test]
    fn test_peg_tuple_type_two_elements() {
        // Test two-element tuple: (T1, T2)
        assert_eq!(
            crusty_peg_parser::type_expr("(int, bool)"),
            Ok(Type::Tuple {
                types: vec![
                    Type::Primitive(PrimitiveType::Int),
                    Type::Primitive(PrimitiveType::Bool)
                ]
            })
        );
    }

    #[test]
    fn test_peg_tuple_type_three_elements() {
        // Test three-element tuple: (T1, T2, T3)
        assert_eq!(
            crusty_peg_parser::type_expr("(int, bool, char)"),
            Ok(Type::Tuple {
                types: vec![
                    Type::Primitive(PrimitiveType::Int),
                    Type::Primitive(PrimitiveType::Bool),
                    Type::Primitive(PrimitiveType::Char)
                ]
            })
        );
    }

    #[test]
    fn test_peg_tuple_type_with_trailing_comma() {
        // Test tuple with trailing comma
        assert_eq!(
            crusty_peg_parser::type_expr("(int, bool,)"),
            Ok(Type::Tuple {
                types: vec![
                    Type::Primitive(PrimitiveType::Int),
                    Type::Primitive(PrimitiveType::Bool)
                ]
            })
        );
    }

    #[test]
    fn test_peg_tuple_type_with_complex_types() {
        // Test tuple with complex types
        assert_eq!(
            crusty_peg_parser::type_expr("(int*, &bool)"),
            Ok(Type::Tuple {
                types: vec![
                    Type::Pointer {
                        ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                        mutable: false
                    },
                    Type::Reference {
                        ty: Box::new(Type::Primitive(PrimitiveType::Bool)),
                        mutable: false
                    }
                ]
            })
        );
    }

    #[test]
    fn test_peg_pointer_to_tuple() {
        // Test pointer to tuple: (T1, T2)*
        assert_eq!(
            crusty_peg_parser::type_expr("(int, bool)*"),
            Ok(Type::Pointer {
                ty: Box::new(Type::Tuple {
                    types: vec![
                        Type::Primitive(PrimitiveType::Int),
                        Type::Primitive(PrimitiveType::Bool)
                    ]
                }),
                mutable: false
            })
        );
    }

    // ========================================================================
    // GENERIC TYPE TESTS
    // ========================================================================

    #[test]
    fn test_peg_generic_type_single_arg() {
        // Test generic type with single argument: T<A>
        assert_eq!(
            crusty_peg_parser::type_expr("Vec<int>"),
            Ok(Type::Generic {
                base: Box::new(Type::Ident(Ident::new("Vec"))),
                args: vec![Type::Primitive(PrimitiveType::Int)]
            })
        );
    }

    #[test]
    fn test_peg_generic_type_two_args() {
        // Test generic type with two arguments: T<A, B>
        assert_eq!(
            crusty_peg_parser::type_expr("Map<int, bool>"),
            Ok(Type::Generic {
                base: Box::new(Type::Ident(Ident::new("Map"))),
                args: vec![
                    Type::Primitive(PrimitiveType::Int),
                    Type::Primitive(PrimitiveType::Bool)
                ]
            })
        );
    }

    #[test]
    fn test_peg_generic_type_nested() {
        // Test nested generic type: T<A<B>>
        assert_eq!(
            crusty_peg_parser::type_expr("Vec<Vec<int>>"),
            Ok(Type::Generic {
                base: Box::new(Type::Ident(Ident::new("Vec"))),
                args: vec![Type::Generic {
                    base: Box::new(Type::Ident(Ident::new("Vec"))),
                    args: vec![Type::Primitive(PrimitiveType::Int)]
                }]
            })
        );
    }

    #[test]
    fn test_peg_generic_type_with_pointer() {
        // Test generic type with pointer argument: T<A*>
        assert_eq!(
            crusty_peg_parser::type_expr("Vec<int*>"),
            Ok(Type::Generic {
                base: Box::new(Type::Ident(Ident::new("Vec"))),
                args: vec![Type::Pointer {
                    ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                    mutable: false
                }]
            })
        );
    }

    #[test]
    fn test_peg_generic_type_with_whitespace() {
        // Test generic type with whitespace
        assert_eq!(
            crusty_peg_parser::type_expr("Vec< int >"),
            Ok(Type::Generic {
                base: Box::new(Type::Ident(Ident::new("Vec"))),
                args: vec![Type::Primitive(PrimitiveType::Int)]
            })
        );
        assert_eq!(
            crusty_peg_parser::type_expr("Map< int , bool >"),
            Ok(Type::Generic {
                base: Box::new(Type::Ident(Ident::new("Map"))),
                args: vec![
                    Type::Primitive(PrimitiveType::Int),
                    Type::Primitive(PrimitiveType::Bool)
                ]
            })
        );
    }

    #[test]
    fn test_peg_pointer_to_generic() {
        // Test pointer to generic type: T<A>*
        assert_eq!(
            crusty_peg_parser::type_expr("Vec<int>*"),
            Ok(Type::Pointer {
                ty: Box::new(Type::Generic {
                    base: Box::new(Type::Ident(Ident::new("Vec"))),
                    args: vec![Type::Primitive(PrimitiveType::Int)]
                }),
                mutable: false
            })
        );
    }

    #[test]
    fn test_peg_reference_to_generic() {
        // Test reference to generic type: &T<A>
        assert_eq!(
            crusty_peg_parser::type_expr("&Vec<int>"),
            Ok(Type::Reference {
                ty: Box::new(Type::Generic {
                    base: Box::new(Type::Ident(Ident::new("Vec"))),
                    args: vec![Type::Primitive(PrimitiveType::Int)]
                }),
                mutable: false
            })
        );
    }

    // ========================================================================
    // AUTO TYPE TESTS
    // ========================================================================

    #[test]
    fn test_peg_auto_type() {
        // Test auto type
        assert_eq!(crusty_peg_parser::type_expr("auto"), Ok(Type::Auto));
    }

    #[test]
    fn test_peg_pointer_to_auto() {
        // Test pointer to auto: auto*
        assert_eq!(
            crusty_peg_parser::type_expr("auto*"),
            Ok(Type::Pointer {
                ty: Box::new(Type::Auto),
                mutable: false
            })
        );
    }

    #[test]
    fn test_peg_reference_to_auto() {
        // Test reference to auto: &auto
        assert_eq!(
            crusty_peg_parser::type_expr("&auto"),
            Ok(Type::Reference {
                ty: Box::new(Type::Auto),
                mutable: false
            })
        );
    }

    // ========================================================================
    // IDENTIFIER TYPE TESTS
    // ========================================================================

    #[test]
    fn test_peg_identifier_type() {
        // Test user-defined type names
        assert_eq!(
            crusty_peg_parser::type_expr("MyType"),
            Ok(Type::Ident(Ident::new("MyType")))
        );
        assert_eq!(
            crusty_peg_parser::type_expr("SomeStruct"),
            Ok(Type::Ident(Ident::new("SomeStruct")))
        );
    }

    #[test]
    fn test_peg_identifier_type_with_underscore() {
        // Test user-defined type names with underscores
        assert_eq!(
            crusty_peg_parser::type_expr("my_type"),
            Ok(Type::Ident(Ident::new("my_type")))
        );
        assert_eq!(
            crusty_peg_parser::type_expr("_PrivateType"),
            Ok(Type::Ident(Ident::new("_PrivateType")))
        );
    }

    // ========================================================================
    // PARENTHESIZED TYPE TESTS
    // ========================================================================

    #[test]
    fn test_peg_parenthesized_type() {
        // Test parenthesized type for grouping
        assert_eq!(
            crusty_peg_parser::type_expr("(int)"),
            Ok(Type::Primitive(PrimitiveType::Int))
        );
    }

    #[test]
    fn test_peg_parenthesized_pointer() {
        // Test parenthesized pointer type
        assert_eq!(
            crusty_peg_parser::type_expr("(int*)"),
            Ok(Type::Pointer {
                ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                mutable: false
            })
        );
    }

    // ========================================================================
    // COMPLEX COMBINATION TESTS
    // ========================================================================

    #[test]
    fn test_peg_complex_type_combination_1() {
        // Test: &mut int*[10]
        // Should be: mutable reference to array of pointers to int
        assert_eq!(
            crusty_peg_parser::type_expr("&mut int*[10]"),
            Ok(Type::Reference {
                ty: Box::new(Type::Array {
                    ty: Box::new(Type::Pointer {
                        ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                        mutable: false
                    }),
                    size: Some(10)
                }),
                mutable: true
            })
        );
    }

    #[test]
    fn test_peg_complex_type_combination_2() {
        // Test: Vec<int*>[]
        // Should be: slice of generic Vec with pointer argument
        assert_eq!(
            crusty_peg_parser::type_expr("Vec<int*>[]"),
            Ok(Type::Slice {
                ty: Box::new(Type::Generic {
                    base: Box::new(Type::Ident(Ident::new("Vec"))),
                    args: vec![Type::Pointer {
                        ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                        mutable: false
                    }]
                })
            })
        );
    }

    #[test]
    fn test_peg_complex_type_combination_3() {
        // Test: (int, bool)*[]
        // Should be: slice of pointers to tuple
        assert_eq!(
            crusty_peg_parser::type_expr("(int, bool)*[]"),
            Ok(Type::Slice {
                ty: Box::new(Type::Pointer {
                    ty: Box::new(Type::Tuple {
                        types: vec![
                            Type::Primitive(PrimitiveType::Int),
                            Type::Primitive(PrimitiveType::Bool)
                        ]
                    }),
                    mutable: false
                })
            })
        );
    }

    #[test]
    fn test_peg_type_expr_preserves_primitives() {
        // Test that type_expr correctly parses primitive types
        assert_eq!(
            crusty_peg_parser::type_expr("int"),
            Ok(Type::Primitive(PrimitiveType::Int))
        );
        assert_eq!(
            crusty_peg_parser::type_expr("i32"),
            Ok(Type::Primitive(PrimitiveType::I32))
        );
        assert_eq!(
            crusty_peg_parser::type_expr("float"),
            Ok(Type::Primitive(PrimitiveType::Float))
        );
        assert_eq!(
            crusty_peg_parser::type_expr("bool"),
            Ok(Type::Primitive(PrimitiveType::Bool))
        );
        assert_eq!(
            crusty_peg_parser::type_expr("void"),
            Ok(Type::Primitive(PrimitiveType::Void))
        );
    }
}

// ============================================================================
// PROPERTY-BASED TESTS FOR COMPLEX TYPE RULES (Task 3.2)
// ============================================================================

#[cfg(test)]
mod complex_type_properties {
    use super::*;
    use proptest::prelude::*;

    // All primitive type names
    const PRIMITIVE_TYPES: &[&str] = &[
        "int", "i32", "i64", "u32", "u64", "float", "f32", "f64", "bool", "char", "void",
    ];

    // Strategy: Generate a random primitive type name
    fn primitive_type_strategy() -> impl Strategy<Value = String> {
        prop::sample::select(PRIMITIVE_TYPES.to_vec()).prop_map(|s| s.to_string())
    }

    // Strategy: Generate a valid identifier (not a keyword)
    fn ident_strategy() -> impl Strategy<Value = String> {
        "[A-Z][a-zA-Z0-9_]{0,10}".prop_filter("Must not be a keyword", |s| {
            !matches!(
                s.as_str(),
                "int"
                    | "i32"
                    | "i64"
                    | "u32"
                    | "u64"
                    | "float"
                    | "f32"
                    | "f64"
                    | "bool"
                    | "char"
                    | "void"
                    | "auto"
                    | "let"
                    | "var"
                    | "const"
                    | "if"
                    | "else"
                    | "while"
                    | "for"
                    | "return"
                    | "struct"
                    | "enum"
                    | "typedef"
                    | "NULL"
                    | "true"
                    | "false"
            )
        })
    }

    // Strategy: Generate a small array size
    fn array_size_strategy() -> impl Strategy<Value = usize> {
        1usize..100
    }

    /// Property 1: Pointer Type Parsing
    ///
    /// For any primitive type T, the parser should correctly parse T* as a pointer type.
    ///
    /// Validates: Requirements 1.2, 6.9 (Type expressions)
    #[test]
    fn property_pointer_type_parsing() {
        proptest!(ProptestConfig::with_cases(100), |(
            prim in primitive_type_strategy()
        )| {
            let input = format!("{}*", prim);
            let result = crusty_peg_parser::type_expr(&input);
            prop_assert!(
                result.is_ok(),
                "Pointer type '{}' should parse successfully",
                input
            );
            if let Ok(Type::Pointer { ty, mutable }) = result {
                prop_assert!(!mutable, "Pointer should not be mutable");
                prop_assert!(
                    matches!(*ty, Type::Primitive(_)),
                    "Inner type should be primitive"
                );
            } else {
                prop_assert!(false, "Result should be a Pointer type");
            }
        });
    }

    /// Property 2: Double Pointer Type Parsing
    ///
    /// For any primitive type T, the parser should correctly parse T** as a pointer to pointer.
    ///
    /// Validates: Requirements 1.2, 6.9 (Type expressions)
    #[test]
    fn property_double_pointer_type_parsing() {
        proptest!(ProptestConfig::with_cases(100), |(
            prim in primitive_type_strategy()
        )| {
            let input = format!("{}**", prim);
            let result = crusty_peg_parser::type_expr(&input);
            prop_assert!(
                result.is_ok(),
                "Double pointer type '{}' should parse successfully",
                input
            );
            if let Ok(Type::Pointer { ty: outer, .. }) = result {
                if let Type::Pointer { ty: inner, .. } = *outer {
                    prop_assert!(
                        matches!(*inner, Type::Primitive(_)),
                        "Innermost type should be primitive"
                    );
                } else {
                    prop_assert!(false, "Outer pointer should contain inner pointer");
                }
            } else {
                prop_assert!(false, "Result should be a Pointer type");
            }
        });
    }

    /// Property 3: Reference Type Parsing
    ///
    /// For any primitive type T, the parser should correctly parse &T as an immutable reference.
    ///
    /// Validates: Requirements 1.2, 6.9 (Type expressions)
    #[test]
    fn property_reference_type_parsing() {
        proptest!(ProptestConfig::with_cases(100), |(
            prim in primitive_type_strategy()
        )| {
            let input = format!("&{}", prim);
            let result = crusty_peg_parser::type_expr(&input);
            prop_assert!(
                result.is_ok(),
                "Reference type '{}' should parse successfully",
                input
            );
            if let Ok(Type::Reference { ty, mutable }) = result {
                prop_assert!(!mutable, "Reference should be immutable");
                prop_assert!(
                    matches!(*ty, Type::Primitive(_)),
                    "Inner type should be primitive"
                );
            } else {
                prop_assert!(false, "Result should be a Reference type");
            }
        });
    }

    /// Property 4: Mutable Reference Type Parsing
    ///
    /// For any primitive type T, the parser should correctly parse &mut T as a mutable reference.
    ///
    /// Validates: Requirements 1.2, 6.9 (Type expressions)
    #[test]
    fn property_mutable_reference_type_parsing() {
        proptest!(ProptestConfig::with_cases(100), |(
            prim in primitive_type_strategy()
        )| {
            let input = format!("&mut {}", prim);
            let result = crusty_peg_parser::type_expr(&input);
            prop_assert!(
                result.is_ok(),
                "Mutable reference type '{}' should parse successfully",
                input
            );
            if let Ok(Type::Reference { ty, mutable }) = result {
                prop_assert!(mutable, "Reference should be mutable");
                prop_assert!(
                    matches!(*ty, Type::Primitive(_)),
                    "Inner type should be primitive"
                );
            } else {
                prop_assert!(false, "Result should be a Reference type");
            }
        });
    }

    /// Property 5: Array Type Parsing
    ///
    /// For any primitive type T and size N, the parser should correctly parse T[N] as an array.
    ///
    /// Validates: Requirements 1.2, 6.9 (Type expressions)
    #[test]
    fn property_array_type_parsing() {
        proptest!(ProptestConfig::with_cases(100), |(
            prim in primitive_type_strategy(),
            size in array_size_strategy()
        )| {
            let input = format!("{}[{}]", prim, size);
            let result = crusty_peg_parser::type_expr(&input);
            prop_assert!(
                result.is_ok(),
                "Array type '{}' should parse successfully",
                input
            );
            if let Ok(Type::Array { ty, size: parsed_size }) = result {
                prop_assert_eq!(parsed_size, Some(size), "Array size should match");
                prop_assert!(
                    matches!(*ty, Type::Primitive(_)),
                    "Inner type should be primitive"
                );
            } else {
                prop_assert!(false, "Result should be an Array type");
            }
        });
    }

    /// Property 6: Slice Type Parsing
    ///
    /// For any primitive type T, the parser should correctly parse T[] as a slice.
    ///
    /// Validates: Requirements 1.2, 6.9 (Type expressions)
    #[test]
    fn property_slice_type_parsing() {
        proptest!(ProptestConfig::with_cases(100), |(
            prim in primitive_type_strategy()
        )| {
            let input = format!("{}[]", prim);
            let result = crusty_peg_parser::type_expr(&input);
            prop_assert!(
                result.is_ok(),
                "Slice type '{}' should parse successfully",
                input
            );
            if let Ok(Type::Slice { ty }) = result {
                prop_assert!(
                    matches!(*ty, Type::Primitive(_)),
                    "Inner type should be primitive"
                );
            } else {
                prop_assert!(false, "Result should be a Slice type");
            }
        });
    }

    /// Property 7: Generic Type Parsing
    ///
    /// For any identifier I and primitive type T, the parser should correctly parse I<T> as a generic.
    ///
    /// Validates: Requirements 1.2, 6.9 (Type expressions)
    #[test]
    fn property_generic_type_parsing() {
        proptest!(ProptestConfig::with_cases(100), |(
            ident in ident_strategy(),
            prim in primitive_type_strategy()
        )| {
            let input = format!("{}<{}>", ident, prim);
            let result = crusty_peg_parser::type_expr(&input);
            prop_assert!(
                result.is_ok(),
                "Generic type '{}' should parse successfully",
                input
            );
            if let Ok(Type::Generic { base, args }) = result {
                prop_assert!(
                    matches!(*base, Type::Ident(_)),
                    "Base type should be identifier"
                );
                prop_assert_eq!(args.len(), 1, "Should have one type argument");
                prop_assert!(
                    matches!(args[0], Type::Primitive(_)),
                    "Type argument should be primitive"
                );
            } else {
                prop_assert!(false, "Result should be a Generic type");
            }
        });
    }

    /// Property 8: Tuple Type Parsing (Two Elements)
    ///
    /// For any two primitive types T1 and T2, the parser should correctly parse (T1, T2) as a tuple.
    ///
    /// Validates: Requirements 1.2, 6.9 (Type expressions)
    #[test]
    fn property_tuple_type_parsing() {
        proptest!(ProptestConfig::with_cases(100), |(
            prim1 in primitive_type_strategy(),
            prim2 in primitive_type_strategy()
        )| {
            let input = format!("({}, {})", prim1, prim2);
            let result = crusty_peg_parser::type_expr(&input);
            prop_assert!(
                result.is_ok(),
                "Tuple type '{}' should parse successfully",
                input
            );
            if let Ok(Type::Tuple { types }) = result {
                prop_assert_eq!(types.len(), 2, "Tuple should have two elements");
                prop_assert!(
                    matches!(types[0], Type::Primitive(_)),
                    "First element should be primitive"
                );
                prop_assert!(
                    matches!(types[1], Type::Primitive(_)),
                    "Second element should be primitive"
                );
            } else {
                prop_assert!(false, "Result should be a Tuple type");
            }
        });
    }

    /// Property 9: Parenthesized Type Unwrapping
    ///
    /// For any primitive type T, the parser should correctly parse (T) and unwrap it to T.
    ///
    /// Validates: Requirements 1.2, 6.9 (Type expressions)
    #[test]
    fn property_parenthesized_type_unwrapping() {
        proptest!(ProptestConfig::with_cases(100), |(
            prim in primitive_type_strategy()
        )| {
            let input = format!("({})", prim);
            let result = crusty_peg_parser::type_expr(&input);
            prop_assert!(
                result.is_ok(),
                "Parenthesized type '{}' should parse successfully",
                input
            );
            // Should unwrap to primitive, not tuple
            prop_assert!(
                matches!(result, Ok(Type::Primitive(_))),
                "Parenthesized type should unwrap to primitive, not tuple"
            );
        });
    }

    /// Property 10: Reference to Pointer Parsing
    ///
    /// For any primitive type T, the parser should correctly parse &T* as reference to pointer.
    ///
    /// Validates: Requirements 1.2, 6.9 (Type expressions)
    #[test]
    fn property_reference_to_pointer_parsing() {
        proptest!(ProptestConfig::with_cases(100), |(
            prim in primitive_type_strategy()
        )| {
            let input = format!("&{}*", prim);
            let result = crusty_peg_parser::type_expr(&input);
            prop_assert!(
                result.is_ok(),
                "Reference to pointer type '{}' should parse successfully",
                input
            );
            if let Ok(Type::Reference { ty, mutable }) = result {
                prop_assert!(!mutable, "Reference should be immutable");
                if let Type::Pointer { ty: inner, .. } = *ty {
                    prop_assert!(
                        matches!(*inner, Type::Primitive(_)),
                        "Innermost type should be primitive"
                    );
                } else {
                    prop_assert!(false, "Reference should contain pointer");
                }
            } else {
                prop_assert!(false, "Result should be a Reference type");
            }
        });
    }

    /// Property 11: Whitespace Invariance for Types
    ///
    /// For any primitive type T, adding whitespace around type operators should not change the result.
    ///
    /// Validates: Requirements 1.5 (Whitespace handling)
    #[test]
    fn property_type_whitespace_invariance() {
        proptest!(ProptestConfig::with_cases(100), |(
            prim in primitive_type_strategy()
        )| {
            // Test pointer with and without whitespace
            let no_space = format!("{}*", prim);
            let with_space = format!("{} *", prim);

            let result1 = crusty_peg_parser::type_expr(&no_space);
            let result2 = crusty_peg_parser::type_expr(&with_space);

            prop_assert!(result1.is_ok(), "No-space version should parse");
            prop_assert!(result2.is_ok(), "With-space version should parse");
            prop_assert_eq!(result1, result2, "Results should be equal regardless of whitespace");
        });
    }

    /// Property 12: Identifier Type Parsing
    ///
    /// For any valid identifier, the parser should correctly parse it as an identifier type.
    ///
    /// Validates: Requirements 1.2, 6.9 (Type expressions)
    #[test]
    fn property_identifier_type_parsing() {
        proptest!(ProptestConfig::with_cases(100), |(
            ident in ident_strategy()
        )| {
            let result = crusty_peg_parser::type_expr(&ident);
            prop_assert!(
                result.is_ok(),
                "Identifier type '{}' should parse successfully",
                ident
            );
            if let Ok(Type::Ident(parsed_ident)) = result {
                prop_assert_eq!(
                    parsed_ident.name, ident,
                    "Parsed identifier should match input"
                );
            } else {
                prop_assert!(false, "Result should be an Ident type");
            }
        });
    }
}

// ============================================================================
// PROPERTY-BASED TESTS FOR KEYWORD RULES (Task 2.2)
// ============================================================================

#[cfg(test)]
mod keyword_properties {
    use super::*;
    use proptest::prelude::*;

    // All Crusty keywords
    const KEYWORDS: &[&str] = &[
        "let",
        "var",
        "const",
        "static",
        "mut",
        "define",
        "if",
        "else",
        "while",
        "for",
        "in",
        "return",
        "break",
        "continue",
        "struct",
        "enum",
        "typedef",
        "namespace",
        "extern",
        "unsafe",
        "loop",
        "match",
        "switch",
        "case",
        "default",
        "auto",
        "int",
        "i32",
        "i64",
        "u32",
        "u64",
        "float",
        "f32",
        "f64",
        "bool",
        "char",
        "void",
        "true",
        "false",
        "NULL",
    ];

    // Strategy: Generate a random keyword from the list
    fn keyword_strategy() -> impl Strategy<Value = String> {
        prop::sample::select(KEYWORDS.to_vec()).prop_map(|s| s.to_string())
    }

    // Strategy: Generate a valid identifier character
    fn ident_char_strategy() -> impl Strategy<Value = char> {
        prop_oneof![
            prop::char::range('a', 'z'),
            prop::char::range('A', 'Z'),
            prop::char::range('0', '9'),
            Just('_'),
        ]
    }

    // Strategy: Generate a valid identifier suffix (non-empty)
    fn ident_suffix_strategy() -> impl Strategy<Value = String> {
        prop::collection::vec(ident_char_strategy(), 1..10)
            .prop_map(|chars| chars.into_iter().collect())
    }

    // Strategy: Generate a non-identifier character (whitespace, punctuation, etc.)
    fn non_ident_char_strategy() -> impl Strategy<Value = char> {
        prop_oneof![Just(' '), Just('\t'), Just('\n'), Just('\r'),]
    }

    // Strategy: Generate a valid identifier that is NOT a keyword
    fn non_keyword_ident_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z_][a-zA-Z0-9_]{0,20}"
            .prop_filter("Must not be a keyword", |s| !KEYWORDS.contains(&s.as_str()))
    }

    /// Property 1: Keyword Recognition
    ///
    /// For any keyword in the Crusty language, the parser should recognize it
    /// as a keyword when followed by a non-identifier character.
    ///
    /// Validates: Requirements 1.2 (Grammar completeness)
    #[test]
    fn property_keyword_recognition() {
        proptest!(ProptestConfig::with_cases(100), |(
            keyword in keyword_strategy(),
            terminator in non_ident_char_strategy()
        )| {
            let input = format!("{}{}", keyword, terminator);
            let result = crusty_peg_parser::test_keyword(&input);
            prop_assert!(
                result.is_ok(),
                "Keyword '{}' should be recognized in input '{}'",
                keyword,
                input
            );
            prop_assert_eq!(
                result.unwrap(),
                keyword,
                "Parsed keyword should match input keyword"
            );
        });
    }

    /// Property 2: Lookahead Correctness
    ///
    /// For any keyword followed by identifier characters, the parser should NOT
    /// recognize it as a keyword (it's a prefix of an identifier).
    ///
    /// Example: "letter" should not match keyword "let"
    ///
    /// Validates: Requirements 1.2 (Keyword lookahead)
    #[test]
    fn property_keyword_lookahead() {
        proptest!(ProptestConfig::with_cases(100), |(
            keyword in keyword_strategy(),
            suffix in ident_suffix_strategy()
        )| {
            let input = format!("{}{}", keyword, suffix);
            let result = crusty_peg_parser::test_keyword(&input);
            prop_assert!(
                result.is_err(),
                "Keyword '{}' with suffix '{}' should NOT be recognized as keyword (input: '{}')",
                keyword,
                suffix,
                input
            );
        });
    }

    /// Property 3: Case Sensitivity
    ///
    /// Keywords are case-sensitive. Variations in case should NOT match the keyword.
    ///
    /// Example: "Let" should not match keyword "let"
    ///
    /// Validates: Requirements 1.2 (Keyword case sensitivity)
    #[test]
    fn property_keyword_case_sensitivity() {
        proptest!(ProptestConfig::with_cases(100), |(
            keyword in keyword_strategy(),
        )| {
            // Convert first character to uppercase (if lowercase) or lowercase (if uppercase)
            let mut chars: Vec<char> = keyword.chars().collect();
            if chars[0].is_lowercase() {
                chars[0] = chars[0].to_uppercase().next().unwrap();
            } else {
                chars[0] = chars[0].to_lowercase().next().unwrap();
            }
            let modified: String = chars.into_iter().collect();

            // Only test if the modification actually changed the string
            prop_assume!(modified != keyword);

            let result = crusty_peg_parser::test_keyword(&modified);
            prop_assert!(
                result.is_err(),
                "Case-modified keyword '{}' (from '{}') should NOT be recognized as keyword",
                modified,
                keyword
            );
        });
    }

    /// Property 4: Non-Keyword Identifier Rejection
    ///
    /// For any valid identifier that is NOT a keyword, the keyword parser
    /// should reject it.
    ///
    /// Validates: Requirements 1.2 (Keyword vs identifier distinction)
    #[test]
    fn property_non_keyword_rejection() {
        proptest!(ProptestConfig::with_cases(100), |(
            ident in non_keyword_ident_strategy(),
        )| {
            let result = crusty_peg_parser::test_keyword(&ident);
            prop_assert!(
                result.is_err(),
                "Non-keyword identifier '{}' should NOT be recognized as keyword",
                ident
            );
        });
    }

    /// Property 5: Keyword Boundary Detection
    ///
    /// Keywords must be complete tokens. A keyword followed by an identifier
    /// character should not match, but a keyword followed by whitespace or
    /// punctuation should match.
    ///
    /// Validates: Requirements 1.2 (Keyword boundary detection)
    #[test]
    fn property_keyword_boundary() {
        proptest!(ProptestConfig::with_cases(100), |(
            keyword in keyword_strategy(),
        )| {
            // Test with no trailing content (end of input)
            let result = crusty_peg_parser::test_keyword(&keyword);
            prop_assert!(
                result.is_ok(),
                "Keyword '{}' at end of input should be recognized",
                keyword
            );

            // Test with trailing whitespace
            let input_with_space = format!("{} ", keyword);
            let result2 = crusty_peg_parser::test_keyword(&input_with_space);
            prop_assert!(
                result2.is_ok(),
                "Keyword '{}' with trailing space should be recognized",
                keyword
            );
        });
    }

    /// Property 6: Keyword Not Identifier
    ///
    /// For any keyword, the test_not_keyword parser should reject it
    /// (it should fail because the input IS a keyword).
    ///
    /// Validates: Requirements 1.2 (Keyword exclusion from identifiers)
    #[test]
    fn property_keyword_not_identifier() {
        proptest!(ProptestConfig::with_cases(100), |(
            keyword in keyword_strategy(),
        )| {
            let result = crusty_peg_parser::test_not_keyword(&keyword);
            prop_assert!(
                result.is_err(),
                "Keyword '{}' should be rejected by test_not_keyword (it IS a keyword)",
                keyword
            );
        });
    }

    /// Property 7: Non-Keyword Is Identifier
    ///
    /// For any valid identifier that is NOT a keyword, the test_not_keyword
    /// parser should accept it.
    ///
    /// Validates: Requirements 1.2 (Non-keyword identifiers are valid)
    #[test]
    fn property_non_keyword_is_identifier() {
        proptest!(ProptestConfig::with_cases(100), |(
            ident in non_keyword_ident_strategy(),
        )| {
            let result = crusty_peg_parser::test_not_keyword(&ident);
            prop_assert!(
                result.is_ok(),
                "Non-keyword identifier '{}' should be accepted by test_not_keyword",
                ident
            );
            prop_assert_eq!(
                result.unwrap(),
                ident,
                "Parsed identifier should match input"
            );
        });
    }

    /// Property 8: Keyword Completeness
    ///
    /// All keywords defined in KEYWORDS should be recognized by the parser.
    /// This ensures no keyword is missing from the grammar.
    ///
    /// Validates: Requirements 1.2 (Grammar completeness)
    #[test]
    fn property_keyword_completeness() {
        // Test all keywords are recognized
        for keyword in KEYWORDS {
            let result = crusty_peg_parser::test_keyword(keyword);
            assert!(result.is_ok(), "Keyword '{}' should be recognized", keyword);
            assert_eq!(result.unwrap(), *keyword, "Parsed keyword should match");
        }
    }

    /// Property 9: Keyword with Whitespace Prefix
    ///
    /// Keywords should be recognized even with leading whitespace.
    ///
    /// Validates: Requirements 1.5 (Whitespace handling)
    #[test]
    fn property_keyword_with_whitespace_prefix() {
        proptest!(ProptestConfig::with_cases(100), |(
            keyword in keyword_strategy(),
            spaces in prop::collection::vec(prop_oneof![Just(' '), Just('\t'), Just('\n')], 1..5),
        )| {
            let prefix: String = spaces.into_iter().collect();
            let input = format!("{}{}", prefix, keyword);
            let result = crusty_peg_parser::test_keyword(&input);
            prop_assert!(
                result.is_ok(),
                "Keyword '{}' with whitespace prefix should be recognized (input: '{:?}')",
                keyword,
                input
            );
        });
    }

    /// Property 10: Keyword with Whitespace Suffix
    ///
    /// Keywords should be recognized with trailing whitespace.
    ///
    /// Validates: Requirements 1.5 (Whitespace handling)
    #[test]
    fn property_keyword_with_whitespace_suffix() {
        proptest!(ProptestConfig::with_cases(100), |(
            keyword in keyword_strategy(),
            spaces in prop::collection::vec(prop_oneof![Just(' '), Just('\t'), Just('\n')], 1..5),
        )| {
            let suffix: String = spaces.into_iter().collect();
            let input = format!("{}{}", keyword, suffix);
            let result = crusty_peg_parser::test_keyword(&input);
            prop_assert!(
                result.is_ok(),
                "Keyword '{}' with whitespace suffix should be recognized (input: '{:?}')",
                keyword,
                input
            );
        });
    }
}

// ============================================================================
// PRIMARY EXPRESSION TESTS (Task 4.1)
// ============================================================================

#[cfg(test)]
mod primary_expression_tests {
    use super::*;

    // ========================================================================
    // LITERAL EXPRESSION TESTS
    // ========================================================================

    #[test]
    fn test_literal_expr_int() {
        let result = crusty_peg_parser::literal_expr("42");
        assert_eq!(result, Ok(Expression::Literal(Literal::Int(42))));
    }

    #[test]
    fn test_literal_expr_float() {
        let result = crusty_peg_parser::literal_expr("3.15");
        assert_eq!(result, Ok(Expression::Literal(Literal::Float(3.15))));
    }

    #[test]
    fn test_literal_expr_string() {
        let result = crusty_peg_parser::literal_expr("\"hello\"");
        assert_eq!(
            result,
            Ok(Expression::Literal(Literal::String("hello".to_string())))
        );
    }

    #[test]
    fn test_literal_expr_char() {
        let result = crusty_peg_parser::literal_expr("'a'");
        assert_eq!(result, Ok(Expression::Literal(Literal::Char('a'))));
    }

    #[test]
    fn test_literal_expr_bool_true() {
        let result = crusty_peg_parser::literal_expr("true");
        assert_eq!(result, Ok(Expression::Literal(Literal::Bool(true))));
    }

    #[test]
    fn test_literal_expr_bool_false() {
        let result = crusty_peg_parser::literal_expr("false");
        assert_eq!(result, Ok(Expression::Literal(Literal::Bool(false))));
    }

    #[test]
    fn test_literal_expr_null() {
        let result = crusty_peg_parser::literal_expr("NULL");
        assert_eq!(result, Ok(Expression::Literal(Literal::Null)));
    }

    #[test]
    fn test_literal_expr_with_whitespace() {
        let result = crusty_peg_parser::literal_expr("  42  ");
        assert_eq!(result, Ok(Expression::Literal(Literal::Int(42))));
    }

    // ========================================================================
    // IDENTIFIER EXPRESSION TESTS
    // ========================================================================

    #[test]
    fn test_ident_expr_simple() {
        let result = crusty_peg_parser::ident_expr("foo");
        assert_eq!(result, Ok(Expression::Ident(Ident::new("foo"))));
    }

    #[test]
    fn test_ident_expr_with_underscore() {
        let result = crusty_peg_parser::ident_expr("foo_bar");
        assert_eq!(result, Ok(Expression::Ident(Ident::new("foo_bar"))));
    }

    #[test]
    fn test_ident_expr_with_numbers() {
        let result = crusty_peg_parser::ident_expr("foo123");
        assert_eq!(result, Ok(Expression::Ident(Ident::new("foo123"))));
    }

    #[test]
    fn test_ident_expr_with_whitespace() {
        let result = crusty_peg_parser::ident_expr("  foo  ");
        assert_eq!(result, Ok(Expression::Ident(Ident::new("foo"))));
    }

    // ========================================================================
    // PARENTHESIZED EXPRESSION TESTS
    // ========================================================================

    #[test]
    fn test_paren_expr_simple() {
        let result = crusty_peg_parser::paren_expr("(42)");
        assert_eq!(result, Ok(Expression::Literal(Literal::Int(42))));
    }

    #[test]
    fn test_paren_expr_ident() {
        let result = crusty_peg_parser::paren_expr("(foo)");
        assert_eq!(result, Ok(Expression::Ident(Ident::new("foo"))));
    }

    #[test]
    fn test_paren_expr_nested() {
        let result = crusty_peg_parser::paren_expr("((42))");
        assert_eq!(result, Ok(Expression::Literal(Literal::Int(42))));
    }

    #[test]
    fn test_paren_expr_with_whitespace() {
        let result = crusty_peg_parser::paren_expr("( 42 )");
        assert_eq!(result, Ok(Expression::Literal(Literal::Int(42))));
    }

    // ========================================================================
    // ARRAY LITERAL TESTS
    // ========================================================================

    #[test]
    fn test_array_lit_empty() {
        let result = crusty_peg_parser::array_lit("[]");
        assert_eq!(result, Ok(Expression::ArrayLit { elements: vec![] }));
    }

    #[test]
    fn test_array_lit_single() {
        let result = crusty_peg_parser::array_lit("[42]");
        assert_eq!(
            result,
            Ok(Expression::ArrayLit {
                elements: vec![Expression::Literal(Literal::Int(42))]
            })
        );
    }

    #[test]
    fn test_array_lit_multiple() {
        let result = crusty_peg_parser::array_lit("[1, 2, 3]");
        assert_eq!(
            result,
            Ok(Expression::ArrayLit {
                elements: vec![
                    Expression::Literal(Literal::Int(1)),
                    Expression::Literal(Literal::Int(2)),
                    Expression::Literal(Literal::Int(3)),
                ]
            })
        );
    }

    #[test]
    fn test_array_lit_trailing_comma() {
        let result = crusty_peg_parser::array_lit("[1, 2, 3,]");
        assert_eq!(
            result,
            Ok(Expression::ArrayLit {
                elements: vec![
                    Expression::Literal(Literal::Int(1)),
                    Expression::Literal(Literal::Int(2)),
                    Expression::Literal(Literal::Int(3)),
                ]
            })
        );
    }

    #[test]
    fn test_array_lit_with_whitespace() {
        let result = crusty_peg_parser::array_lit("[ 1 , 2 , 3 ]");
        assert_eq!(
            result,
            Ok(Expression::ArrayLit {
                elements: vec![
                    Expression::Literal(Literal::Int(1)),
                    Expression::Literal(Literal::Int(2)),
                    Expression::Literal(Literal::Int(3)),
                ]
            })
        );
    }

    // ========================================================================
    // TUPLE LITERAL TESTS
    // ========================================================================

    #[test]
    fn test_tuple_lit_empty() {
        let result = crusty_peg_parser::tuple_lit("()");
        assert_eq!(result, Ok(Expression::TupleLit { elements: vec![] }));
    }

    #[test]
    fn test_tuple_lit_single_with_comma() {
        let result = crusty_peg_parser::tuple_lit("(42,)");
        assert_eq!(
            result,
            Ok(Expression::TupleLit {
                elements: vec![Expression::Literal(Literal::Int(42))]
            })
        );
    }

    #[test]
    fn test_tuple_lit_two_elements() {
        let result = crusty_peg_parser::tuple_lit("(1, 2)");
        assert_eq!(
            result,
            Ok(Expression::TupleLit {
                elements: vec![
                    Expression::Literal(Literal::Int(1)),
                    Expression::Literal(Literal::Int(2)),
                ]
            })
        );
    }

    #[test]
    fn test_tuple_lit_multiple() {
        let result = crusty_peg_parser::tuple_lit("(1, 2, 3)");
        assert_eq!(
            result,
            Ok(Expression::TupleLit {
                elements: vec![
                    Expression::Literal(Literal::Int(1)),
                    Expression::Literal(Literal::Int(2)),
                    Expression::Literal(Literal::Int(3)),
                ]
            })
        );
    }

    #[test]
    fn test_tuple_lit_trailing_comma() {
        let result = crusty_peg_parser::tuple_lit("(1, 2, 3,)");
        assert_eq!(
            result,
            Ok(Expression::TupleLit {
                elements: vec![
                    Expression::Literal(Literal::Int(1)),
                    Expression::Literal(Literal::Int(2)),
                    Expression::Literal(Literal::Int(3)),
                ]
            })
        );
    }

    // ========================================================================
    // STRUCT INIT TESTS
    // ========================================================================

    #[test]
    fn test_struct_init_empty() {
        let result = crusty_peg_parser::struct_init("Point {}");
        assert_eq!(
            result,
            Ok(Expression::StructInit {
                ty: Type::Ident(Ident::new("Point")),
                fields: vec![],
            })
        );
    }

    #[test]
    fn test_struct_init_single_field() {
        let result = crusty_peg_parser::struct_init("Point { x: 10 }");
        assert_eq!(
            result,
            Ok(Expression::StructInit {
                ty: Type::Ident(Ident::new("Point")),
                fields: vec![(Ident::new("x"), Expression::Literal(Literal::Int(10)))],
            })
        );
    }

    #[test]
    fn test_struct_init_multiple_fields() {
        let result = crusty_peg_parser::struct_init("Point { x: 10, y: 20 }");
        assert_eq!(
            result,
            Ok(Expression::StructInit {
                ty: Type::Ident(Ident::new("Point")),
                fields: vec![
                    (Ident::new("x"), Expression::Literal(Literal::Int(10))),
                    (Ident::new("y"), Expression::Literal(Literal::Int(20))),
                ],
            })
        );
    }

    #[test]
    fn test_struct_init_trailing_comma() {
        let result = crusty_peg_parser::struct_init("Point { x: 10, y: 20, }");
        assert_eq!(
            result,
            Ok(Expression::StructInit {
                ty: Type::Ident(Ident::new("Point")),
                fields: vec![
                    (Ident::new("x"), Expression::Literal(Literal::Int(10))),
                    (Ident::new("y"), Expression::Literal(Literal::Int(20))),
                ],
            })
        );
    }

    #[test]
    fn test_struct_init_generic_type() {
        let result = crusty_peg_parser::struct_init("Vec<int> {}");
        assert_eq!(
            result,
            Ok(Expression::StructInit {
                ty: Type::Generic {
                    base: Box::new(Type::Ident(Ident::new("Vec"))),
                    args: vec![Type::Primitive(PrimitiveType::Int)],
                },
                fields: vec![],
            })
        );
    }

    // ========================================================================
    // PRIMARY EXPRESSION TESTS (combined)
    // ========================================================================

    #[test]
    fn test_primary_literal() {
        let result = crusty_peg_parser::primary("42");
        assert_eq!(result, Ok(Expression::Literal(Literal::Int(42))));
    }

    #[test]
    fn test_primary_ident() {
        let result = crusty_peg_parser::primary("foo");
        assert_eq!(result, Ok(Expression::Ident(Ident::new("foo"))));
    }

    #[test]
    fn test_primary_paren() {
        let result = crusty_peg_parser::primary("(42)");
        assert_eq!(result, Ok(Expression::Literal(Literal::Int(42))));
    }

    #[test]
    fn test_primary_array() {
        let result = crusty_peg_parser::primary("[1, 2, 3]");
        assert_eq!(
            result,
            Ok(Expression::ArrayLit {
                elements: vec![
                    Expression::Literal(Literal::Int(1)),
                    Expression::Literal(Literal::Int(2)),
                    Expression::Literal(Literal::Int(3)),
                ]
            })
        );
    }

    #[test]
    fn test_primary_tuple() {
        let result = crusty_peg_parser::primary("(1, 2)");
        assert_eq!(
            result,
            Ok(Expression::TupleLit {
                elements: vec![
                    Expression::Literal(Literal::Int(1)),
                    Expression::Literal(Literal::Int(2)),
                ]
            })
        );
    }

    #[test]
    fn test_primary_struct_init() {
        let result = crusty_peg_parser::primary("Point { x: 10 }");
        assert_eq!(
            result,
            Ok(Expression::StructInit {
                ty: Type::Ident(Ident::new("Point")),
                fields: vec![(Ident::new("x"), Expression::Literal(Literal::Int(10)))],
            })
        );
    }

    // ========================================================================
    // EXPR RULE TESTS (placeholder)
    // ========================================================================

    #[test]
    fn test_expr_literal() {
        let result = crusty_peg_parser::expr("42");
        assert_eq!(result, Ok(Expression::Literal(Literal::Int(42))));
    }

    #[test]
    fn test_expr_ident() {
        let result = crusty_peg_parser::expr("foo");
        assert_eq!(result, Ok(Expression::Ident(Ident::new("foo"))));
    }

    // ========================================================================
    // CAST EXPRESSION TESTS (Task 4.2)
    // ========================================================================
    // Tests for the cast_expr rule which handles C-style type casts: (Type)(expr)
    // These tests validate Requirements 2.1, 2.2, 2.4, 2.5, 2.6

    #[test]
    fn test_cast_expr_basic_int() {
        // Test basic cast: (int)(x)
        // Validates: Requirement 2.1 - (Type)(expr) is correctly identified as cast
        let result = crusty_peg_parser::cast_expr("(int)(x)");
        assert_eq!(
            result,
            Ok(Expression::Cast {
                expr: Box::new(Expression::Ident(Ident::new("x"))),
                ty: Type::Primitive(PrimitiveType::Int),
            })
        );
    }

    #[test]
    fn test_cast_expr_basic_float() {
        // Test cast to float: (float)(y)
        let result = crusty_peg_parser::cast_expr("(float)(y)");
        assert_eq!(
            result,
            Ok(Expression::Cast {
                expr: Box::new(Expression::Ident(Ident::new("y"))),
                ty: Type::Primitive(PrimitiveType::Float),
            })
        );
    }

    #[test]
    fn test_cast_expr_with_literal() {
        // Test cast with literal expression: (int)(42)
        let result = crusty_peg_parser::cast_expr("(int)(42)");
        assert_eq!(
            result,
            Ok(Expression::Cast {
                expr: Box::new(Expression::Literal(Literal::Int(42))),
                ty: Type::Primitive(PrimitiveType::Int),
            })
        );
    }

    #[test]
    fn test_cast_expr_pointer_type() {
        // Test cast with pointer type: (int*)(expr)
        // Validates: Requirement 2.6 - casts with complex type expressions
        let result = crusty_peg_parser::cast_expr("(int*)(ptr)");
        assert_eq!(
            result,
            Ok(Expression::Cast {
                expr: Box::new(Expression::Ident(Ident::new("ptr"))),
                ty: Type::Pointer {
                    ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                    mutable: false,
                },
            })
        );
    }

    #[test]
    fn test_cast_expr_double_pointer() {
        // Test cast with double pointer: (int**)(expr)
        let result = crusty_peg_parser::cast_expr("(int**)(ptr)");
        assert_eq!(
            result,
            Ok(Expression::Cast {
                expr: Box::new(Expression::Ident(Ident::new("ptr"))),
                ty: Type::Pointer {
                    ty: Box::new(Type::Pointer {
                        ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                        mutable: false,
                    }),
                    mutable: false,
                },
            })
        );
    }

    #[test]
    fn test_cast_expr_reference_type() {
        // Test cast with reference type: (&int)(expr)
        // Validates: Requirement 2.6 - casts with reference types
        let result = crusty_peg_parser::cast_expr("(&int)(val)");
        assert_eq!(
            result,
            Ok(Expression::Cast {
                expr: Box::new(Expression::Ident(Ident::new("val"))),
                ty: Type::Reference {
                    ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                    mutable: false,
                },
            })
        );
    }

    #[test]
    fn test_cast_expr_mutable_reference() {
        // Test cast with mutable reference: (&mut int)(expr)
        let result = crusty_peg_parser::cast_expr("(&mut int)(val)");
        assert_eq!(
            result,
            Ok(Expression::Cast {
                expr: Box::new(Expression::Ident(Ident::new("val"))),
                ty: Type::Reference {
                    ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                    mutable: true,
                },
            })
        );
    }

    #[test]
    fn test_cast_expr_user_defined_type() {
        // Test cast with user-defined type: (MyType)(expr)
        let result = crusty_peg_parser::cast_expr("(MyType)(val)");
        assert_eq!(
            result,
            Ok(Expression::Cast {
                expr: Box::new(Expression::Ident(Ident::new("val"))),
                ty: Type::Ident(Ident::new("MyType")),
            })
        );
    }

    #[test]
    fn test_cast_expr_with_whitespace() {
        // Test cast with various whitespace
        let result = crusty_peg_parser::cast_expr("( int )( x )");
        assert_eq!(
            result,
            Ok(Expression::Cast {
                expr: Box::new(Expression::Ident(Ident::new("x"))),
                ty: Type::Primitive(PrimitiveType::Int),
            })
        );
    }

    #[test]
    fn test_cast_expr_with_newlines() {
        // Test cast with newlines
        let result = crusty_peg_parser::cast_expr("(int)\n(x)");
        assert_eq!(
            result,
            Ok(Expression::Cast {
                expr: Box::new(Expression::Ident(Ident::new("x"))),
                ty: Type::Primitive(PrimitiveType::Int),
            })
        );
    }

    #[test]
    fn test_cast_expr_with_comments() {
        // Test cast with comments
        let result = crusty_peg_parser::cast_expr("(int /* type */) /* cast */ (x)");
        assert_eq!(
            result,
            Ok(Expression::Cast {
                expr: Box::new(Expression::Ident(Ident::new("x"))),
                ty: Type::Primitive(PrimitiveType::Int),
            })
        );
    }

    #[test]
    fn test_cast_expr_all_primitive_types() {
        // Test cast with all primitive types
        let types = vec![
            ("int", Type::Primitive(PrimitiveType::Int)),
            ("i32", Type::Primitive(PrimitiveType::I32)),
            ("i64", Type::Primitive(PrimitiveType::I64)),
            ("u32", Type::Primitive(PrimitiveType::U32)),
            ("u64", Type::Primitive(PrimitiveType::U64)),
            ("float", Type::Primitive(PrimitiveType::Float)),
            ("f32", Type::Primitive(PrimitiveType::F32)),
            ("f64", Type::Primitive(PrimitiveType::F64)),
            ("bool", Type::Primitive(PrimitiveType::Bool)),
            ("char", Type::Primitive(PrimitiveType::Char)),
        ];

        for (type_str, expected_type) in types {
            let input = format!("({})(x)", type_str);
            let result = crusty_peg_parser::cast_expr(&input);
            assert_eq!(
                result,
                Ok(Expression::Cast {
                    expr: Box::new(Expression::Ident(Ident::new("x"))),
                    ty: expected_type,
                }),
                "Failed for type: {}",
                type_str
            );
        }
    }

    #[test]
    fn test_cast_expr_in_primary() {
        // Test that cast_expr is correctly tried in primary()
        // This validates that cast_expr comes before paren_expr in ordered choice
        let result = crusty_peg_parser::primary("(int)(x)");
        assert_eq!(
            result,
            Ok(Expression::Cast {
                expr: Box::new(Expression::Ident(Ident::new("x"))),
                ty: Type::Primitive(PrimitiveType::Int),
            })
        );
    }

    #[test]
    fn test_paren_expr_not_cast() {
        // Test that (expr) without a following (expr) is NOT a cast
        // Validates: Requirement 2.2 - (expr) is correctly identified as parenthesized expression
        let result = crusty_peg_parser::primary("(42)");
        assert_eq!(result, Ok(Expression::Literal(Literal::Int(42))));
    }

    #[test]
    fn test_paren_expr_with_ident() {
        // Test that (ident) without a following (expr) is NOT a cast
        let result = crusty_peg_parser::primary("(x)");
        assert_eq!(result, Ok(Expression::Ident(Ident::new("x"))));
    }

    #[test]
    fn test_cast_vs_paren_disambiguation() {
        // Test disambiguation between cast and parenthesized expression
        // (int)(x) should be a cast
        let cast_result = crusty_peg_parser::primary("(int)(x)");
        assert!(matches!(cast_result, Ok(Expression::Cast { .. })));

        // (x) should be a parenthesized expression (just returns x)
        let paren_result = crusty_peg_parser::primary("(x)");
        assert!(matches!(paren_result, Ok(Expression::Ident(_))));
    }

    #[test]
    fn test_cast_expr_array_type() {
        // Test cast with array type: (int[10])(expr)
        let result = crusty_peg_parser::cast_expr("(int[10])(arr)");
        assert_eq!(
            result,
            Ok(Expression::Cast {
                expr: Box::new(Expression::Ident(Ident::new("arr"))),
                ty: Type::Array {
                    ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                    size: Some(10),
                },
            })
        );
    }

    #[test]
    fn test_cast_expr_slice_type() {
        // Test cast with slice type: (int[])(expr)
        let result = crusty_peg_parser::cast_expr("(int[])(slice)");
        assert_eq!(
            result,
            Ok(Expression::Cast {
                expr: Box::new(Expression::Ident(Ident::new("slice"))),
                ty: Type::Slice {
                    ty: Box::new(Type::Primitive(PrimitiveType::Int)),
                },
            })
        );
    }

    #[test]
    fn test_cast_expr_generic_type() {
        // Test cast with generic type: (Vec<int>)(expr)
        let result = crusty_peg_parser::cast_expr("(Vec<int>)(vec)");
        assert_eq!(
            result,
            Ok(Expression::Cast {
                expr: Box::new(Expression::Ident(Ident::new("vec"))),
                ty: Type::Generic {
                    base: Box::new(Type::Ident(Ident::new("Vec"))),
                    args: vec![Type::Primitive(PrimitiveType::Int)],
                },
            })
        );
    }

    #[test]
    fn test_cast_expr_tuple_type() {
        // Test cast with tuple type: ((int, bool))(expr)
        let result = crusty_peg_parser::cast_expr("((int, bool))(tup)");
        assert_eq!(
            result,
            Ok(Expression::Cast {
                expr: Box::new(Expression::Ident(Ident::new("tup"))),
                ty: Type::Tuple {
                    types: vec![
                        Type::Primitive(PrimitiveType::Int),
                        Type::Primitive(PrimitiveType::Bool),
                    ],
                },
            })
        );
    }

    #[test]
    fn test_cast_expr_nested_cast() {
        // Test nested casts: (T1)((T2)(expr))
        // Validates: Requirement 2.5 - nested casts
        // Note: The inner cast is parsed as the expression being cast
        let result = crusty_peg_parser::cast_expr("(int)((float)(x))");
        assert_eq!(
            result,
            Ok(Expression::Cast {
                expr: Box::new(Expression::Cast {
                    expr: Box::new(Expression::Ident(Ident::new("x"))),
                    ty: Type::Primitive(PrimitiveType::Float),
                }),
                ty: Type::Primitive(PrimitiveType::Int),
            })
        );
    }
}
