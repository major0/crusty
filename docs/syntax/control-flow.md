# Control Flow

## Introduction

Crusty provides C-like control flow statements including if, while, for, and labeled loops.

## Rationale

Familiar control flow syntax reduces the learning curve for C programmers while providing Rust's safety guarantees.

## Examples

### If Statement

```crusty
if (x > 0) {
    println("positive");
} else if (x < 0) {
    println("negative");
} else {
    println("zero");
}
```

### While Loop

```crusty
while (count < 10) {
    println("{}", count);
    count = count + 1;
}
```

### For Loop

```crusty
for (int i = 0; i < 10; i = i + 1) {
    println("{}", i);
}
```

### Labeled Loops

Crusty uses dot-prefixed labels for loop declarations, mimicking C/ASM identifier syntax. The dot is a syntactic prefix for declarations only - it is NOT part of the label name. When using `break` or `continue`, reference the label without the dot:

```crusty
.outer: loop {
    .inner: loop {
        if (condition) break outer;   // No dot in break
        continue inner;               // No dot in continue
    }
}
```

This translates to Rust's tick-prefixed labels:

```rust
'outer: loop {
    'inner: loop {
        if condition { break 'outer; }
        continue 'inner;
    }
}
```

**Label Syntax Rules:**
- Label declarations use dot prefix: `.label: loop { ... }`
- Label references do NOT use dot: `break label`, `continue label`
- The dot mimics C/ASM identifier syntax but is NOT part of the label name

### Break and Continue

```crusty
while (true) {
    if (done) break;
    if (skip) continue;
    // process
}
```

## Formal Grammar

```ebnf
if_stmt       ::= 'if' '(' expression ')' block ('else' 'if' '(' expression ')' block)* ('else' block)?
while_stmt    ::= 'while' '(' expression ')' block
for_stmt      ::= 'for' '(' init? ';' condition? ';' increment? ')' block
loop_stmt     ::= label_decl? 'loop' block
break_stmt    ::= 'break' label_ref? ';'
continue_stmt ::= 'continue' label_ref? ';'
label_decl    ::= '.' identifier ':'    // Declaration uses dot prefix
label_ref     ::= identifier            // Reference uses bare identifier (no dot)
```
