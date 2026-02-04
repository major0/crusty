# Parser Rewrite: Choosing a Parser Generator

## Introduction

The original Crusty parser is hand-written recursive descent. While functional, it has limitations around ambiguity resolution — particularly with C-style cast expressions like `(Type)(expr)` vs parenthesized expressions `(expr)`. A parser generator provides formal grammar-driven parsing that handles these ambiguities cleanly.

## Rationale

Two parser generators were evaluated as viable options for Crusty:

- **rust-peg** — a PEG parser with left recursion support via `#[cache_left_rec]`
- **LALRPOP** — a traditional LR(1) parser generator with EBNF grammar

### Key Differences

| Aspect | rust-peg (PEG) | LALRPOP (LR) |
|--------|---------------|--------------|
| Left Recursion | Supported via `#[cache_left_rec]` | Native LR support |
| Ambiguity Resolution | Ordered choice (first match wins) | LR(1) lookahead + conflict resolution |
| Grammar Location | Inline in Rust code | Separate `.lalrpop` file |
| Learning Curve | Lower — PEG is intuitive | Higher — requires LR parsing theory |
| Debugging | Trace execution path | Shift/reduce conflict analysis |
| Precedence | `precedence!` macro | Grammar-level declarations |

### C-Like Language Considerations

C and its derivatives were historically designed alongside YACC, an LR parser generator. Left recursion is natural in C-like syntax (e.g., `expr + expr`), operator precedence aligns with LR grammar structure, and statement structure (semicolons, braces) is naturally LR-friendly.

Standard PEG parsers cannot handle left recursion, but rust-peg's `#[cache_left_rec]` attribute eliminates this limitation, allowing natural C-like grammars without transformation.

### Decision

The project chose rust-peg for the best balance of simplicity and capability:

- PEG ordered choice for natural ambiguity resolution (no shift/reduce conflicts)
- Left recursion support — write `expr = expr "+" term` directly
- Inline grammar in Rust code for better type integration
- Packrat parsing with memoization for performance
- `precedence!` macro for clean operator precedence handling
- Used by Python's new parser, proving viability for C-like languages

## Examples

### Cast Expression Resolution

```rust
rule primary() -> Expression
    = cast_expr()      // Try cast first
    / paren_expr()     // Then parenthesized expression
    / literal()
    / ident()

rule cast_expr() -> Expression
    = "(" t:type_expr() ")" "(" e:expr() ")" { Expression::Cast { ty: t, expr: Box::new(e) } }

rule paren_expr() -> Expression
    = "(" e:expr() ")" { e }
```

Ordered choice (`/`) tries alternatives in order. If `cast_expr` matches `(Type)(expr)`, it succeeds. Otherwise, rust-peg backtracks and tries `paren_expr`.

### Left-Recursive Expression Grammar

```rust
#[cache_left_rec]
rule expr() -> Expression = precedence! {
    l:(@) "+" r:@ { Expression::BinOp(Op::Add, Box::new(l), Box::new(r)) }
    l:(@) "-" r:@ { Expression::BinOp(Op::Sub, Box::new(l), Box::new(r)) }
    --
    l:(@) "*" r:@ { Expression::BinOp(Op::Mul, Box::new(l), Box::new(r)) }
    l:(@) "/" r:@ { Expression::BinOp(Op::Div, Box::new(l), Box::new(r)) }
    --
    n:number() { Expression::Number(n) }
    "(" e:expr() ")" { e }
}
```

The `precedence!` macro handles operator precedence levels separated by `--`, with left-associativity via `(@)` on the left side.

## Formal Grammar

rust-peg uses PEG notation where:
- Sequence: `a() b()` (space-separated)
- Ordered choice: `a() / b()` (try left first)
- Zero or more: `e()*`
- One or more: `e()+`
- Optional: `e()?`
- Binding: `name:rule()` (capture result)
