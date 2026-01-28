# Lexer

## Introduction

The lexer (lexical analyzer) is the first phase of the Crusty compiler. It reads raw source text and produces a stream of tokens that the parser consumes.

## Rationale

Separating lexical analysis from parsing simplifies both components. The lexer handles low-level concerns like whitespace, comments, and character encoding, while the parser works with a clean token stream.

## Examples

Input:
```c
int add(int a, int b) {
    return a + b;
}
```

Token stream:
```
INT, IDENT("add"), LPAREN, INT, IDENT("a"), COMMA,
INT, IDENT("b"), RPAREN, LBRACE, RETURN, IDENT("a"),
PLUS, IDENT("b"), SEMICOLON, RBRACE
```

## Token Categories

- **Keywords**: `int`, `float`, `bool`, `char`, `void`, `let`, `var`, `const`, `if`, `else`, `while`, `for`, `return`, `break`, `continue`, `struct`, `enum`, `static`, `typedef`, `switch`, `case`, `default`, `loop`, `in`, `sizeof`, `true`, `false`, `NULL`
- **Operators**: `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`, `!`, `&`, `|`, `^`, `<<`, `>>`, `++`, `--`, `=`, `+=`, `-=`, `*=`, `/=`
- **Delimiters**: `(`, `)`, `{`, `}`, `[`, `]`, `,`, `;`, `.`, `->`, `::`, `?`, `:`
- **Literals**: Integer, float, string, character, boolean
- **Identifiers**: User-defined names

## Formal Grammar

```ebnf
token      = keyword | operator | delimiter | literal | identifier | comment ;
comment    = line_comment | block_comment ;
line_comment  = "//" {any_char} newline ;
block_comment = "/*" {any_char} "*/" ;
```
