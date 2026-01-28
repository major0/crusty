# Crusty Syntax Overview

## Introduction

Crusty is a C-like programming language that transpiles to Rust. It provides familiar C syntax while leveraging Rust's safety guarantees and standard library.

## Rationale

Many developers are familiar with C-style syntax. Crusty allows these developers to write code in a familiar style while benefiting from Rust's memory safety, type system, and ecosystem.

## Examples

### Hello World

```crusty
void main() {
    println("Hello, World!");
}
```

### Function with Parameters

```crusty
int add(int a, int b) {
    return a + b;
}
```

### Struct Definition

```crusty
struct Point {
    int x;
    int y;
}
```

### Control Flow

```crusty
void example(int n) {
    if (n > 0) {
        println("positive");
    } else if (n < 0) {
        println("negative");
    } else {
        println("zero");
    }
}
```

## Key Differences from C

- No manual memory management (uses Rust's ownership system)
- No null pointers (uses Rust's Option type)
- No undefined behavior
- No goto statements
- No unions (use Rust enums instead)
- No #include (use module system instead)

## Formal Grammar

The complete formal grammar is defined in the parser implementation. Key productions:

```ebnf
file        ::= item*
item        ::= function | struct | enum | typedef | const | static
function    ::= type? identifier '(' params? ')' block
params      ::= param (',' param)*
param       ::= type identifier
block       ::= '{' statement* '}'
```
