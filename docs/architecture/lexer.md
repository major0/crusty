# Lexer

## Introduction

The lexer (lexical analyzer) tokenizes Crusty source code into a stream of tokens for the parser.

## Rationale

Separating lexical analysis from parsing simplifies the parser and provides clear error locations.

## Token Types

### Keywords
```
fn, let, var, const, static, if, else, while, for, return,
break, continue, struct, enum, typedef, namespace, extern
```

### Type Keywords
```
int, i32, i64, u32, u64, float, f32, f64, bool, char, void
```

### Operators
```
+  -  *  /  %           // arithmetic
== != <  >  <= >=       // comparison
&& || !                 // logical
&  |  ^  ~  << >>       // bitwise
=  += -= *= /=          // assignment
.  ->  ..  ..=          // access and range
```

### Delimiters
```
( ) { } [ ]             // brackets
, ; : :: ?              // punctuation
```

### Literals
```
42                      // integer
3.14                    // float
"hello"                 // string
'a'                     // character
true false              // boolean
```

### Special Tokens
```
#                       // preprocessor directive
!                       // macro invocation / error propagation
@                       // type-scoped calls
```

## Interface

```rust
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub text: String,
}

pub struct Lexer<'a> {
    source: &'a str,
    position: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self;
    pub fn next_token(&mut self) -> Result<Token, LexError>;
    pub fn peek_token(&mut self) -> Result<Token, LexError>;
}
```
