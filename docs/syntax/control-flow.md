# Control Flow

## Introduction

Crusty provides C-style control flow constructs including if/else, while loops, for loops, and labeled break/continue. These translate directly to their Rust equivalents.

## Rationale

C-style control flow is familiar to most developers. Crusty preserves this syntax while adding labeled loops using a dot-prefix notation (`.label:`) for declarations that translates to Rust's tick-prefix (`'label:`). When referencing labels in `break` and `continue` statements, the label name is used without any prefix.

## Examples

### If/Else
```c
if (x > 0) {
    println!("positive");
} else if (x < 0) {
    println!("negative");
} else {
    println!("zero");
}
```

### While Loop
```c
var i = 0;
while (i < 10) {
    println!("{}", i);
    i = i + 1;
}
```

### C-Style For Loop
```c
for (var i = 0; i < 10; i++) {
    println!("{}", i);
}
```

### For-In Loop
```c
for item in collection {
    println!("{}", item);
}
```

### Labeled Loops
```c
.outer: loop {
    .inner: loop {
        if (condition) break outer;
        continue inner;
    }
}
```
Translates to:
```rust
'outer: loop {
    'inner: loop {
        if condition { break 'outer; }
        continue 'inner;
    }
}
```

Note: The dot prefix (`.label:`) is used only in label declarations. When referencing labels in `break` and `continue`, use the bare name without any prefix.

## Formal Grammar

```ebnf
if_stmt       = "if" "(" expr ")" block ["else" (if_stmt | block)] ;
while_stmt    = "while" "(" expr ")" block ;
for_stmt      = "for" "(" [var_decl | expr_stmt] ";" [expr] ";" [expr] ")" block ;
for_in_stmt   = "for" IDENT "in" expr block ;
loop_stmt     = ["." IDENT ":"] "loop" block ;
break_stmt    = "break" [IDENT] ";" ;
continue_stmt = "continue" [IDENT] ";" ;
```
