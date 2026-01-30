// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Parser module for building AST from token stream.

use crate::ast::*;
use crate::error::ParseError;
use crate::lexer::{Lexer, Token, TokenKind};

/// Parser for Crusty source code
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
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
        })
    }

    /// Advance to the next token
    fn advance(&mut self) -> Result<(), ParseError> {
        self.current_token = self
            .lexer
            .next_token()
            .map_err(|e| ParseError::new(e.span, e.message, vec![], "lexical error"))?;
        Ok(())
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
            TokenKind::Typedef => self.parse_typedef(),
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
                } else if self.check(&TokenKind::BitAnd) {
                    // &self or &var self
                    self.advance()?;

                    let mutable = if self.check(&TokenKind::Var) {
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
    fn parse_typedef(&mut self) -> Result<Item, ParseError> {
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
            visibility: Visibility::Public,
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
            _ => {
                // Try to parse as expression statement
                let expr = self.parse_expression_stub()?;
                self.expect(TokenKind::Semicolon)?;
                Ok(Statement::Expr(expr))
            }
        }
    }

    /// Parse a let statement
    fn parse_let_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(TokenKind::Let)?;

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
                    "expected variable name",
                    vec!["identifier".to_string()],
                    format!("{:?}", self.current_token.kind),
                ));
            }
        };

        // Parse optional type annotation
        let ty = if self.check(&TokenKind::Colon) {
            self.advance()?;
            Some(self.parse_type()?)
        } else {
            None
        };

        // Parse optional initializer
        let mut init = if self.check(&TokenKind::Assign) {
            self.advance()?;
            Some(self.parse_expression_stub()?)
        } else {
            None
        };

        // If the initializer is a struct initializer with Type::Auto,
        // and we have a type annotation, update the struct initializer type
        if let Some(ref var_type) = ty {
            if let Some(Expression::StructInit {
                ty: struct_ty,
                fields: _,
            }) = init.as_mut()
            {
                if matches!(struct_ty, Type::Auto) {
                    // Replace Type::Auto with the variable type
                    *struct_ty = var_type.clone();
                }
            }
        }

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
                    "expected variable name",
                    vec!["identifier".to_string()],
                    format!("{:?}", self.current_token.kind),
                ));
            }
        };

        // Parse optional type annotation
        let ty = if self.check(&TokenKind::Colon) {
            self.advance()?;
            Some(self.parse_type()?)
        } else {
            None
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

        // Parse type annotation (required for const)
        self.expect(TokenKind::Colon)?;
        let ty = self.parse_type()?;

        // Parse initializer (required for const)
        self.expect(TokenKind::Assign)?;
        let value = self.parse_expression_stub()?;

        self.expect(TokenKind::Semicolon)?;

        Ok(Statement::Const { name, ty, value })
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
                // Parenthesized expression or tuple literal
                self.advance()?;

                // Check for empty tuple ()
                if self.check(&TokenKind::RParen) {
                    self.advance()?;
                    return Ok(Expression::TupleLit {
                        elements: Vec::new(),
                    });
                }

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
        // Check for reference types (& and &var/&mut)
        if self.check(&TokenKind::BitAnd) {
            self.advance()?;

            // Check for mutable reference (&var or &mut)
            let mutable = if self.check(&TokenKind::Var) {
                self.advance()?;
                true
            } else {
                // Check for Rust-style &mut (not in lexer yet, but prepare for it)
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
        let source = "int main() { const x: int = 5; }";
        let mut parser = Parser::new(source).unwrap();

        let file = parser.parse_file().unwrap();
        match &file.items[0] {
            Item::Function(func) => {
                assert_eq!(func.body.statements.len(), 1);
                match &func.body.statements[0] {
                    Statement::Const { name, .. } => {
                        assert_eq!(name.name, "x");
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
    let source = "int foo(&var int x) {}";
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
    let source = "int main() { let p: Point = { .x = 10, .y = 20 }; }";
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();

    assert_eq!(file.items.len(), 1);
    match &file.items[0] {
        Item::Function(func) => {
            assert_eq!(func.body.statements.len(), 1);
            match &func.body.statements[0] {
                Statement::Let { init, ty, .. } => {
                    // Check that type is specified
                    assert!(ty.is_some());

                    if let Some(Expression::StructInit { ty, fields }) = init {
                        // Check type - should be Point (resolved from variable type)
                        match ty {
                            Type::Ident(ident) => assert_eq!(ident.name, "Point"),
                            _ => panic!("Expected Type::Ident for struct initializer"),
                        }
                        // Check fields
                        assert_eq!(fields.len(), 2);
                        assert_eq!(fields[0].0.name, "x");
                        assert_eq!(fields[1].0.name, "y");
                    } else {
                        panic!("Expected StructInit expression");
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
    let source = "int main() { let p: Point = { .x = 10 }; }";
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();

    assert_eq!(file.items.len(), 1);
    match &file.items[0] {
        Item::Function(func) => match &func.body.statements[0] {
            Statement::Let { init, .. } => {
                if let Some(Expression::StructInit { fields, .. }) = init {
                    assert_eq!(fields.len(), 1);
                    assert_eq!(fields[0].0.name, "x");
                } else {
                    panic!("Expected StructInit expression");
                }
            }
            _ => panic!("Expected Let statement"),
        },
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_parse_struct_initializer_trailing_comma() {
    let source = "int main() { let p: Point = { .x = 10, .y = 20, }; }";
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();

    assert_eq!(file.items.len(), 1);
    match &file.items[0] {
        Item::Function(func) => match &func.body.statements[0] {
            Statement::Let { init, .. } => {
                if let Some(Expression::StructInit { fields, .. }) = init {
                    assert_eq!(fields.len(), 2);
                } else {
                    panic!("Expected StructInit expression");
                }
            }
            _ => panic!("Expected Let statement"),
        },
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_parse_struct_initializer_nested() {
    let source = "int main() { let r: Rect = { .origin = { .x = 0, .y = 0 }, .size = { .w = 10, .h = 20 } }; }";
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();

    assert_eq!(file.items.len(), 1);
    match &file.items[0] {
        Item::Function(func) => {
            match &func.body.statements[0] {
                Statement::Let { init, .. } => {
                    if let Some(Expression::StructInit { fields, .. }) = init {
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
                        panic!("Expected StructInit expression");
                    }
                }
                _ => panic!("Expected Let statement"),
            }
        }
        _ => panic!("Expected Function"),
    }
}
