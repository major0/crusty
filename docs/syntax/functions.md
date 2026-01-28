# Functions

## Introduction

Functions in Crusty use C-style syntax with the return type before the function name.

## Rationale

C-style function declarations are familiar to most programmers and clearly indicate the return type at the start of the declaration.

## Examples

### Basic Function

```crusty
int add(int a, int b) {
    return a + b;
}
```

### Void Function

```crusty
void greet(char* name) {
    println("Hello, {}!", name);
}
```

### Static Function

Static functions are module-private:

```crusty
static int helper(int x) {
    return x * 2;
}
```

### Function with No Parameters

```crusty
int get_value() {
    return 42;
}
```

## Formal Grammar

```ebnf
function    ::= visibility? 'static'? type identifier '(' params? ')' block
visibility  ::= 'pub'
type        ::= primitive_type | identifier | pointer_type | array_type
params      ::= param (',' param)*
param       ::= type identifier
block       ::= '{' statement* '}'
```
