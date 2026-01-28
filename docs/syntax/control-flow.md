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

Crusty uses dot-prefixed labels for loop control:

```crusty
.outer: loop {
    .inner: loop {
        if (condition) break .outer;
        continue .inner;
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
if_stmt     ::= 'if' '(' expression ')' block ('else' 'if' '(' expression ')' block)* ('else' block)?
while_stmt  ::= 'while' '(' expression ')' block
for_stmt    ::= 'for' '(' init? ';' condition? ';' increment? ')' block
loop_stmt   ::= label? 'loop' block
break_stmt  ::= 'break' label? ';'
continue_stmt ::= 'continue' label? ';'
label       ::= '.' identifier ':'
```
