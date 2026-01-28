# Parser

## Introduction

The parser is the second phase of the Crusty compiler. It consumes the token stream from the lexer and builds an Abstract Syntax Tree (AST) representing the program structure.

## Rationale

A recursive descent parser was chosen for its simplicity and ability to produce clear error messages. The parser uses operator precedence climbing for expression parsing, which handles C's complex operator precedence rules naturally.

## Examples

Input:
```c
int factorial(int n) {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}
```

Produces an AST with:
- A `Function` node with name "factorial", return type `int`, parameter `n: int`
- An `If` statement with condition `n <= 1` and body `return 1`
- A `Return` statement with expression `n * factorial(n - 1)`

## Formal Grammar

```ebnf
file       = {item} ;
item       = function_decl | struct_decl | enum_decl | typedef_decl
           | const_decl | use_decl | extern_decl ;
block      = "{" {statement} "}" ;
statement  = let_decl | var_decl | const_decl | if_stmt | while_stmt
           | for_stmt | return_stmt | break_stmt | continue_stmt
           | expr_stmt | block ;
```
