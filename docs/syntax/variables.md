# Variables

## Introduction

Crusty supports variable declarations using C-style syntax with type before the variable name.

## Rationale

C-style declarations are familiar and clearly indicate the type at the start of the declaration.

## Examples

### Immutable Variable (let)

```crusty
int x = 42;
```

### Mutable Variable (var)

```crusty
var int count = 0;
count = count + 1;
```

### Constants

```crusty
const int MAX_SIZE = 100;
```

### Static Variables

```crusty
static int counter = 0;
```

## Formal Grammar

```ebnf
var_decl    ::= 'var'? type identifier ('=' expression)? ';'
const_decl  ::= 'const' type identifier '=' expression ';'
static_decl ::= 'static' 'var'? type identifier ('=' expression)? ';'
```
