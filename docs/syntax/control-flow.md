# Control Flow

## Introduction

Crusty provides C-style control flow constructs including if/else, while loops, for loops, and labeled break/continue. These translate directly to their Rust equivalents.

## Rationale

C-style control flow is familiar to most developers. Crusty preserves this syntax while adding labeled loops using a dot-prefix notation (`.label:`) that translates to Rust's tick-prefix (`'label:`). The label wears the dot at both its declaration and its reference (`break .label;`, `continue .label;`), which keeps a label distinct from a `break` value.

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
for (item in collection) {
    println!("{}", item);
}
```

### Infinite Loop
```c
loop {
    if (done) {
        break;
    }
    work();
}
```
Translates to Rust's `loop` unchanged. Exit with `break` (optionally labeled);
use it when there is no loop condition to evaluate up front.

A `loop` is also an expression: it can appear where a value is expected and
yields the value of a `break <value>` inside it.
```c
let first_even = loop {
    if (n % 2 == 0) {
        break n;
    }
    n = n + 1;
};
```
Only `loop` is value-producing this way; `while` and `for` are statements.

### Labeled Loops
```c
.outer: loop {
    .inner: loop {
        if (condition) {
            break .outer;
        }
        continue .inner;
    }
}
```
Translates to:
```rust
'outer: loop {
    'inner: loop {
        if condition {
            break 'outer;
        }
        continue 'inner;
    }
}
```

Note: A label wears the dot (`.label`) at BOTH its declaration (`.outer:`) and
its reference (`break .outer;`, `continue .outer;`). An undotted name is never a
label — see Break below.

### Break and Continue
`break` has three forms; `continue` takes an optional dotted label:
```c
break;              // leave the innermost loop
break value;        // leave a `loop`, yielding `value`
break .outer;       // leave the loop labeled `.outer`
break .outer value; // leave `.outer`, yielding `value`
continue;           // next iteration of the innermost loop
continue .outer;    // next iteration of the loop labeled `.outer`
```
The dot is what distinguishes a label from a value: an undotted name after
`break` is always a VALUE expression (`break x;` yields `x`), never a label, so
a labeled break cannot be written by accident. `continue` carries no value, so
an undotted name after it is a parse error.

## Formal Grammar

```ebnf
if_stmt       = "if" "(" expr ")" block ["else" (if_stmt | block)] ;
while_stmt    = "while" "(" expr ")" block ;
for_stmt      = "for" "(" [var_decl | expr_stmt] ";" [expr] ";" [expr] ")" block ;
for_in_stmt   = "for" "(" IDENT "in" expr ")" block ;
loop_stmt     = ["." IDENT ":"] "loop" block ;
break_stmt    = "break" ["." IDENT] [expr] ";" ;
continue_stmt = "continue" ["." IDENT] ";" ;
```
