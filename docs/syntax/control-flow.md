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

### Conditional binding — `if` / `while (decl; cond)`

`if` and `while` accept an **optional** C99-`for`-header-style declaration before
the condition, separated by `;`. The binding is scoped to the whole statement
(the condition and the branch(es)) and dropped after — the same scope-tightening
C99 introduced for `for (int i = 0; …)`. No `let` keyword is required: the init
clause is any declaration (`int i = e`, `let i = e`, `var i = e`).

```c
if (int i = compute(); i > 10) {
    use(i);            // i is in scope here
} else {
    recover(i);        // and here
}                      // i is gone after the statement

while (var i = seed(); i > 0) {   // bind once, then loop; var i is mutable
    i = i - 1;
}
```

lowers to a block wrapping the binding and a plain conditional:

```rust
{ let i = compute(); if i > 10 { use(i); } else { recover(i); } }
{ let mut i = seed(); while i > 0 { i = i - 1; } }
```

The prefix is optional sugar — a bare `if (cond)` / `while (cond)` is unchanged.
The header is exactly two clauses: a single declaration (no declarator list) and a
single boolean (no comma operator). It is bind-then-test, not pattern matching —
use `switch` to destructure.

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

## Switch

`switch` is Crusty's one multi-way construct (ADR-0032): a value-yielding
expression that matches a value against a series of **patterns** and takes the
first case that matches. Because it yields a value it can appear in value
position (a `let` initializer); used as a statement, its value is simply
unused. It replaces the Rust-style `match` (retired) — the pattern power is the
same, worn in C/Go-style `case … :` syntax.

```c
let grade = switch (score) {
    case 0: 100;
    case 1 ... 5: 50;
    case n if n > 6: 10;
    default: 0;
};
```

Lowers to a Rust `match` (Rust's scrutinee is bare; the Crusty parens are a
surface wrapper and are not re-emitted; each case becomes an arm):
```rust
let grade = match score {
    0 => 100,
    1..=5 => 50,
    n if n > 6 => 10,
    _ => 0,
};
```

### Case patterns

The case patterns Crusty understands structurally:

| Pattern | Example | Meaning |
|---|---|---|
| Wildcard | `default:` | the catch-all case (lowers to Rust `_`) |
| Literal | `case 0:`, `case 'a':`, `case true:` | matches that exact value |
| Binding | `case n:` | matches anything and binds it to `n` |
| Range | `case 1 ... 5:`, `case 'a' ... 'z':` | inclusive range (GNU-C style) |
| Comma-list | `case 1, 2, 3:` | matches any of the listed patterns (Go style) |
| Path | `case Color::Red:` | matches a unit enum variant or a constant |

The comma-list is Crusty's alternation surface; `|` is **not** accepted in a
case (it keeps its sole meaning, bitwise-OR). A **guard** (`if <cond>`) after
the pattern adds a boolean condition the case must also satisfy. A pattern
binding is in scope in the guard and the case body, and does not leak past the
case:
```c
let doubled = switch (n) {
    case x if x > 6: x + x;
    case y: y;
};
```

### Raw pass-through patterns

Any pattern Crusty does not model structurally — payload destructuring
(`Some(v)`), tuples (`(a, b)`), struct patterns (`Point { x, y }`), slice
patterns (`[first, .., last]`) — is carried **verbatim** to Rust and left to the
Rust compiler, the same way generic bounds and `where` clauses pass through.
Crusty does not analyse such a case's body (the names a raw pattern binds live
in text Crusty did not parse), so those cases rely on `rustc` for their
diagnostics.

```c
let v = switch (opt) {
    case Some(x): x;   // `Some(x)` is passed through verbatim
    case None: 0;
};
```

### Case bodies

A case body is a bare expression terminated by `;` (`case n: expr;`, whose value
is `expr` — the `;` is the case terminator) or a brace block (`case n: { … }`,
whose value is the block's trailing semicolon-less expression, ADR-0029):
```c
let label = switch (n) {
    case 0: "zero";                     // bare-expression body
    case 1: { let d = n * 2; "one" }    // block body; value is the trailing expr
    default: "other";
};
```

### Boundaries

- **A `switch` is not a function boundary.** A `return` inside a case body exits
  the enclosing function (not the switch); `break`/`continue [.label] [value]`
  target the enclosing loop. This fixes C's footgun where `break` inside a
  `switch` escapes only the switch.
- **Native enum payloads destructure; foreign ones need `__rust__`.** A Crusty
  `enum` variant may be a unit, a discriminant (`= n`), a tuple payload
  (`Variant(T, …)`), or a struct payload (`Variant { T x; … }`), so a pattern
  like `Shape.Circle(r)` destructures a Crusty-declared variant directly. A value
  whose type is defined outside Crusty still needs a raw pass-through pattern.
- **No exhaustiveness or pattern/scrutinee type checking in Crusty.** Whether the
  cases cover every value, whether any is unreachable, and whether a pattern
  matches the scrutinee's type are all decided by `rustc` (ADR-0001).

The C-style fallthrough-statement `switch` is gone: cases do not fall through
(each lowers to an independent `match` arm), there is no `case:`-stacking, and
there is no `fallthrough` keyword (Rust has none to lower to).

## Value-blocks and `if` as a value

A brace block can appear where a value is expected — a `let`/`var` initializer,
a `switch` case body, an `if`/`else` branch. In that position it is a
**value-block**: its value is its final semicolon-less **trailing expression**.

```c
let x = { let t = expensive(); t + 1 };   // x = t + 1
```

An `if`/`else` is itself an expression and can be used directly as a value; each
branch is a value-block, so a branch's value is its trailing expression:

```c
let grade = if (score > 90) { 1 } else { 2 };          // direct form
let grade = { if (score > 90) { 1 } else { 2 } };      // redundant wrap, same value
let band  = if (s > 90) { 1 } else if (s > 50) { 2 } else { 3 };   // chain
```

The redundant `{ … }` wrap around an `if` is accepted and yields the same value;
the direct form is the idiomatic one.

### `return` has one meaning: leave the function

A `return` is **never** a value-block's yield — it always exits the nearest
enclosing function or closure. So a `return` inside a value-block punches out of
the *containing* function, not the block:

```c
int pick(int x) {
    let y = { if (x <= 5) { return 0; } x + 1 };   // x<=5: returns 0 from pick
    return y + 100;                                // otherwise y = x + 1
}
```

### Function bodies require `return`

A function or closure body (`fn`, `__func__`) is **not** a value-block. A
non-void body must yield its value with an explicit `return` — a bare trailing
value in a function body is an error, even in tail position (ADR-0029):

```c
int f(int x) { return x + 1; }   // OK
int f(int x) { x + 1 }           // ERROR — a function body requires `return`
```

This keeps the function-vs-value-block distinction visible: `return` leaves a
function, a trailing expression yields a value-block. A `__func__` lowers to a
Rust closure, so a value-block nested inside its body yields locally while a
`return` in that body exits the `__func__` (the closure), not the outer
function.

## Formal Grammar

```ebnf
if_stmt       = "if" "(" expr ")" block ["else" (if_stmt | block)] ;
while_stmt    = "while" "(" expr ")" block ;
for_stmt      = "for" "(" (var_decl | expr_stmt) ";" expr ";" [expr] ")" block ;
for_in_stmt   = "for" IDENT "in" expr block ;
loop_stmt     = ["." IDENT ":"] "loop" block ;
break_stmt    = "break" ["." IDENT] [expr] ";" ;
continue_stmt = "continue" ["." IDENT] ";" ;
switch_stmt   = "switch" "(" expr ")" "{" {switch_case} [switch_default] "}" ;
switch_expr   = switch_stmt ;                        (* in expression position *)
switch_case   = "case" case_pattern ["if" expr_no_struct] ":" case_body ;
case_pattern  = pattern_no_or {"," pattern_no_or} ;  (* comma-list -> Pattern::Or *)
case_body     = block | expr ";" ;                   (* block tail, or bare value *)
switch_default = "default" ":" case_body ;
pattern_no_or = "_" | literal | IDENT
              | literal "..." literal                (* inclusive range *)
              | IDENT {"::" IDENT}                   (* path pattern, 2+ segments *)
              | raw_pattern ;                        (* verbatim pass-through *)

(* ADR-0029 value-blocks: a block in expression position yields its trailing,
   semicolon-less expression; an `if`/`else` is itself an expression. Both are
   primary expressions, so they appear as a `let` initializer, an arm body, or
   an `if`/`else` branch. In a function/closure body a block is NOT a value —
   a non-void body must yield via `return`. *)
block         = "{" {statement} [tail_expr] "}" ;
tail_expr     = expr ;                             (* no trailing ";" *)
value_block   = block ;                            (* block in expression position *)
if_expr       = "if" "(" expr ")" block ["else" (if_expr | block)] ;
```

Note: the `switch` scrutinee is parenthesised — `switch (p) { … }`, like
`if (cond)` — so the full expression grammar is available inside the parens (a
struct-literal scrutinee `switch (Point { x: 1 })` works). A case value element
is parsed through a primary/postfix tier, so `|` (bitwise-OR) is not a case
surface and the comma-list is the alternation form.
