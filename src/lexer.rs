// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Lexical analysis module for tokenizing Crusty source code.

use crate::error::{LexError, Position, Span};

/// Token kinds in Crusty
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TokenKind {
    // Keywords
    Let,
    Var,
    Const,
    Static,
    Mut,
    Define,
    If,
    Else,
    While,
    For,
    In,
    Return,
    Break,
    Continue,
    Struct,
    Enum,
    Typedef,
    Namespace,
    Extern,
    Unsafe,
    Loop,
    Match,
    Switch,
    Case,
    Default,
    Auto,

    // Types
    Int,
    I32,
    I64,
    U32,
    U64,
    Float,
    F32,
    F64,
    Bool,
    Char,
    Void,

    // Operators
    Plus,        // +
    Minus,       // -
    Star,        // *
    Slash,       // /
    Percent,     // %
    Eq,          // ==
    Ne,          // !=
    Lt,          // <
    Gt,          // >
    Le,          // <=
    Ge,          // >=
    And,         // &&
    Or,          // ||
    Not,         // !
    BitAnd,      // &
    BitOr,       // |
    BitXor,      // ^
    BitNot,      // ~
    Shl,         // <<
    Shr,         // >>
    Assign,      // =
    PlusEq,      // +=
    MinusEq,     // -=
    StarEq,      // *=
    SlashEq,     // /=
    PercentEq,   // %=
    AndEq,       // &=
    OrEq,        // |=
    XorEq,       // ^=
    ShlEq,       // <<=
    ShrEq,       // >>=
    Inc,         // ++
    Dec,         // --
    Dot,         // .
    Arrow,       // ->
    DotDot,      // ..
    DotDotEq,    // ..=
    Question,    // ?
    Colon,       // :
    DoubleColon, // ::

    // Delimiters
    LParen,    // (
    RParen,    // )
    LBrace,    // {
    RBrace,    // }
    LBracket,  // [
    RBracket,  // ]
    Comma,     // ,
    Semicolon, // ;

    // Special
    Hash, // #
    Bang, // !
    At,   // @

    // Literals
    IntLiteral(String),
    FloatLiteral(String),
    StringLiteral(String),
    CharLiteral(char),
    BoolLiteral(bool),
    Null,

    // Identifiers
    Ident(String),

    // End of file
    Eof,
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Let => write!(f, "let"),
            TokenKind::Var => write!(f, "var"),
            TokenKind::Ident(s) => write!(f, "identifier '{}'", s),
            TokenKind::IntLiteral(s) => write!(f, "integer '{}'", s),
            TokenKind::Eof => write!(f, "end of file"),
            _ => write!(f, "{:?}", self),
        }
    }
}

/// A token with its kind, span, and text
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub text: String,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, text: String) -> Self {
        Self { kind, span, text }
    }
}

/// Lexer for tokenizing Crusty source code
pub struct Lexer<'a> {
    pub(crate) source: &'a str,
    pub(crate) chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    pub(crate) position: usize,
    pub(crate) line: usize,
    pub(crate) column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.char_indices().peekable(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    fn current_position(&self) -> Position {
        Position::new(self.line, self.column)
    }

    fn advance(&mut self) -> Option<char> {
        if let Some((pos, ch)) = self.chars.next() {
            self.position = pos + ch.len_utf8();
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, ch)| *ch)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        // Skip //
        self.advance();
        self.advance();

        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) -> Result<(), LexError> {
        let start = self.current_position();
        // Skip /*
        self.advance();
        self.advance();

        loop {
            match self.peek() {
                None => {
                    return Err(LexError::new(
                        Span::new(start, self.current_position()),
                        "unterminated block comment",
                    ));
                }
                Some('*') => {
                    self.advance();
                    if self.peek() == Some('/') {
                        self.advance();
                        break;
                    }
                }
                Some(_) => {
                    self.advance();
                }
            }
        }
        Ok(())
    }

    fn read_identifier(&mut self, start_pos: Position, first_char: char) -> Token {
        let start = self.position - first_char.len_utf8();

        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let text = &self.source[start..self.position];
        let kind = match text {
            "let" => TokenKind::Let,
            "var" => TokenKind::Var,
            "const" => TokenKind::Const,
            "static" => TokenKind::Static,
            "mut" => TokenKind::Mut,
            "define" => TokenKind::Define,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "return" => TokenKind::Return,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "struct" => TokenKind::Struct,
            "enum" => TokenKind::Enum,
            "typedef" => TokenKind::Typedef,
            "namespace" => TokenKind::Namespace,
            "extern" => TokenKind::Extern,
            "unsafe" => TokenKind::Unsafe,
            "loop" => TokenKind::Loop,
            "match" => TokenKind::Match,
            "switch" => TokenKind::Switch,
            "case" => TokenKind::Case,
            "default" => TokenKind::Default,
            "auto" => TokenKind::Auto,
            "int" => TokenKind::Int,
            "i32" => TokenKind::I32,
            "i64" => TokenKind::I64,
            "u32" => TokenKind::U32,
            "u64" => TokenKind::U64,
            "float" => TokenKind::Float,
            "f32" => TokenKind::F32,
            "f64" => TokenKind::F64,
            "bool" => TokenKind::Bool,
            "char" => TokenKind::Char,
            "void" => TokenKind::Void,
            "true" => TokenKind::BoolLiteral(true),
            "false" => TokenKind::BoolLiteral(false),
            "NULL" => TokenKind::Null,
            _ => TokenKind::Ident(text.to_string()),
        };

        Token::new(
            kind,
            Span::new(start_pos, self.current_position()),
            text.to_string(),
        )
    }

    fn read_number(&mut self, start_pos: Position, first_char: char) -> Result<Token, LexError> {
        let start = self.position - first_char.len_utf8();
        let mut is_float = false;

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.advance();
            } else if ch == '.' && !is_float {
                // Peek ahead to see if there's a digit after the dot
                // We need to look at the source directly without consuming
                let dot_pos = self.position;

                // Check if there's a character after the dot and if it's a digit
                if dot_pos + 1 < self.source.len() {
                    let next_char = self.source.as_bytes()[dot_pos + 1] as char;
                    if next_char.is_ascii_digit() {
                        // It's a float literal like 3.14
                        self.advance(); // consume the dot
                        is_float = true;
                    } else {
                        // Not a float - could be .. operator or method call
                        break;
                    }
                } else {
                    // End of source after dot
                    break;
                }
            } else {
                break;
            }
        }

        let text = &self.source[start..self.position];
        let kind = if is_float {
            TokenKind::FloatLiteral(text.to_string())
        } else {
            TokenKind::IntLiteral(text.to_string())
        };

        Ok(Token::new(
            kind,
            Span::new(start_pos, self.current_position()),
            text.to_string(),
        ))
    }

    fn read_string(&mut self, start_pos: Position) -> Result<Token, LexError> {
        // Skip opening "
        self.advance();
        let mut value = String::new();

        loop {
            match self.peek() {
                None | Some('\n') => {
                    return Err(LexError::new(
                        Span::new(start_pos, self.current_position()),
                        "unterminated string literal",
                    ));
                }
                Some('"') => {
                    self.advance();
                    break;
                }
                Some('\\') => {
                    self.advance();
                    match self.peek() {
                        Some('n') => {
                            value.push('\n');
                            self.advance();
                        }
                        Some('t') => {
                            value.push('\t');
                            self.advance();
                        }
                        Some('r') => {
                            value.push('\r');
                            self.advance();
                        }
                        Some('\\') => {
                            value.push('\\');
                            self.advance();
                        }
                        Some('"') => {
                            value.push('"');
                            self.advance();
                        }
                        _ => {
                            return Err(LexError::new(
                                Span::new(start_pos, self.current_position()),
                                "invalid escape sequence",
                            ));
                        }
                    }
                }
                Some(ch) => {
                    value.push(ch);
                    self.advance();
                }
            }
        }

        Ok(Token::new(
            TokenKind::StringLiteral(value.clone()),
            Span::new(start_pos, self.current_position()),
            format!("\"{}\"", value),
        ))
    }

    pub fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_whitespace();

        let start_pos = self.current_position();

        // Handle comments
        if self.peek() == Some('/') {
            let saved_pos = (self.position, self.line, self.column);
            self.advance();
            match self.peek() {
                Some('/') => {
                    self.skip_line_comment();
                    return self.next_token();
                }
                Some('*') => {
                    self.skip_block_comment()?;
                    return self.next_token();
                }
                _ => {
                    // Restore position, it's a division operator
                    self.position = saved_pos.0;
                    self.line = saved_pos.1;
                    self.column = saved_pos.2;
                    self.chars = self.source[self.position..].char_indices().peekable();
                }
            }
        }

        let ch = match self.advance() {
            Some(ch) => ch,
            None => {
                return Ok(Token::new(
                    TokenKind::Eof,
                    Span::new(start_pos, self.current_position()),
                    String::new(),
                ));
            }
        };

        let (kind, text) = match ch {
            // Single character tokens
            '(' => (TokenKind::LParen, "("),
            ')' => (TokenKind::RParen, ")"),
            '{' => (TokenKind::LBrace, "{"),
            '}' => (TokenKind::RBrace, "}"),
            '[' => (TokenKind::LBracket, "["),
            ']' => (TokenKind::RBracket, "]"),
            ',' => (TokenKind::Comma, ","),
            ';' => (TokenKind::Semicolon, ";"),
            '?' => (TokenKind::Question, "?"),
            '~' => (TokenKind::BitNot, "~"),
            '#' => (TokenKind::Hash, "#"),
            '@' => (TokenKind::At, "@"),

            // Multi-character operators
            '+' => match self.peek() {
                Some('+') => {
                    self.advance();
                    (TokenKind::Inc, "++")
                }
                Some('=') => {
                    self.advance();
                    (TokenKind::PlusEq, "+=")
                }
                _ => (TokenKind::Plus, "+"),
            },
            '-' => match self.peek() {
                Some('-') => {
                    self.advance();
                    (TokenKind::Dec, "--")
                }
                Some('=') => {
                    self.advance();
                    (TokenKind::MinusEq, "-=")
                }
                Some('>') => {
                    self.advance();
                    (TokenKind::Arrow, "->")
                }
                _ => (TokenKind::Minus, "-"),
            },
            '*' => match self.peek() {
                Some('=') => {
                    self.advance();
                    (TokenKind::StarEq, "*=")
                }
                _ => (TokenKind::Star, "*"),
            },
            '/' => match self.peek() {
                Some('=') => {
                    self.advance();
                    (TokenKind::SlashEq, "/=")
                }
                _ => (TokenKind::Slash, "/"),
            },
            '%' => match self.peek() {
                Some('=') => {
                    self.advance();
                    (TokenKind::PercentEq, "%=")
                }
                _ => (TokenKind::Percent, "%"),
            },
            '=' => match self.peek() {
                Some('=') => {
                    self.advance();
                    (TokenKind::Eq, "==")
                }
                _ => (TokenKind::Assign, "="),
            },
            '!' => match self.peek() {
                Some('=') => {
                    self.advance();
                    (TokenKind::Ne, "!=")
                }
                _ => (TokenKind::Bang, "!"),
            },
            '<' => match self.peek() {
                Some('=') => {
                    self.advance();
                    (TokenKind::Le, "<=")
                }
                Some('<') => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        (TokenKind::ShlEq, "<<=")
                    } else {
                        (TokenKind::Shl, "<<")
                    }
                }
                _ => (TokenKind::Lt, "<"),
            },
            '>' => match self.peek() {
                Some('=') => {
                    self.advance();
                    (TokenKind::Ge, ">=")
                }
                Some('>') => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        (TokenKind::ShrEq, ">>=")
                    } else {
                        (TokenKind::Shr, ">>")
                    }
                }
                _ => (TokenKind::Gt, ">"),
            },
            '&' => match self.peek() {
                Some('&') => {
                    self.advance();
                    (TokenKind::And, "&&")
                }
                Some('=') => {
                    self.advance();
                    (TokenKind::AndEq, "&=")
                }
                _ => (TokenKind::BitAnd, "&"),
            },
            '|' => match self.peek() {
                Some('|') => {
                    self.advance();
                    (TokenKind::Or, "||")
                }
                Some('=') => {
                    self.advance();
                    (TokenKind::OrEq, "|=")
                }
                _ => (TokenKind::BitOr, "|"),
            },
            '^' => match self.peek() {
                Some('=') => {
                    self.advance();
                    (TokenKind::XorEq, "^=")
                }
                _ => (TokenKind::BitXor, "^"),
            },
            '.' => match self.peek() {
                Some('.') => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        (TokenKind::DotDotEq, "..=")
                    } else {
                        (TokenKind::DotDot, "..")
                    }
                }
                _ => (TokenKind::Dot, "."),
            },
            ':' => match self.peek() {
                Some(':') => {
                    self.advance();
                    (TokenKind::DoubleColon, "::")
                }
                _ => (TokenKind::Colon, ":"),
            },

            // String literals
            '"' => {
                return self.read_string(start_pos);
            }

            // Identifiers and keywords
            ch if ch.is_alphabetic() || ch == '_' => {
                return Ok(self.read_identifier(start_pos, ch));
            }

            // Numbers
            ch if ch.is_ascii_digit() => {
                return self.read_number(start_pos, ch);
            }

            _ => {
                return Err(LexError::new(
                    Span::new(start_pos, self.current_position()),
                    format!("unexpected character: '{}'", ch),
                ));
            }
        };

        Ok(Token::new(
            kind,
            Span::new(start_pos, self.current_position()),
            text.to_string(),
        ))
    }

    #[allow(dead_code)]
    pub fn peek_token(&mut self) -> Result<Token, LexError> {
        let saved_state = (
            self.position,
            self.line,
            self.column,
            self.source[self.position..].to_string(),
        );

        let token = self.next_token()?;

        // Restore state
        self.position = saved_state.0;
        self.line = saved_state.1;
        self.column = saved_state.2;
        self.chars = self.source[self.position..].char_indices().peekable();

        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let source = "let var const if else while for return";
        let mut lexer = Lexer::new(source);

        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Let);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Var);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Const);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::If);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Else);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::While);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::For);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Return);
    }

    #[test]
    fn test_operators() {
        let source = "+ - * / == != < > <= >= && ||";
        let mut lexer = Lexer::new(source);

        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Plus);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Minus);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Star);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Slash);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Eq);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Ne);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Lt);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Gt);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Le);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Ge);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::And);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Or);
    }

    #[test]
    fn test_identifiers() {
        let source = "foo bar_baz _private";
        let mut lexer = Lexer::new(source);

        assert!(matches!(
            lexer.next_token().unwrap().kind,
            TokenKind::Ident(_)
        ));
        assert!(matches!(
            lexer.next_token().unwrap().kind,
            TokenKind::Ident(_)
        ));
        assert!(matches!(
            lexer.next_token().unwrap().kind,
            TokenKind::Ident(_)
        ));
    }

    #[test]
    fn test_numbers() {
        let source = "123 456.789";
        let mut lexer = Lexer::new(source);

        assert!(matches!(
            lexer.next_token().unwrap().kind,
            TokenKind::IntLiteral(_)
        ));
        assert!(matches!(
            lexer.next_token().unwrap().kind,
            TokenKind::FloatLiteral(_)
        ));
    }

    #[test]
    fn test_strings() {
        let source = r#""hello" "world\n""#;
        let mut lexer = Lexer::new(source);

        let token1 = lexer.next_token().unwrap();
        assert!(matches!(token1.kind, TokenKind::StringLiteral(_)));

        let token2 = lexer.next_token().unwrap();
        assert!(matches!(token2.kind, TokenKind::StringLiteral(_)));
    }

    #[test]
    fn test_comments() {
        let source = "// line comment\nint /* block comment */ main";
        let mut lexer = Lexer::new(source);

        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Int);
        assert_eq!(
            lexer.next_token().unwrap().kind,
            TokenKind::Ident("main".to_string())
        );
    }

    #[test]
    fn test_delimiters() {
        let source = "( ) { } [ ] , ;";
        let mut lexer = Lexer::new(source);

        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::LParen);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::RParen);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::LBrace);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::RBrace);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::LBracket);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::RBracket);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Comma);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Semicolon);
    }

    #[test]
    fn test_error_unterminated_string() {
        let source = r#""unterminated"#;
        let mut lexer = Lexer::new(source);

        assert!(lexer.next_token().is_err());
    }

    #[test]
    fn test_error_invalid_character() {
        let source = "$invalid";
        let mut lexer = Lexer::new(source);

        assert!(lexer.next_token().is_err());
    }
}
