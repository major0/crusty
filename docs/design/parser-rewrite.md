# Parser Rewrite: Choosing a Parser Generator

## Introduction

The original Crusty parser is hand-written recursive descent. While functional, it has limitations around ambiguity resolution — particularly with C-style cast expressions like `(Type)(expr)` vs parenthesized expressions `(expr)`. A parser generator provides formal grammar-driven parsing that handles these ambiguities cleanly.

## Rationale

Two parser generators were evaluated for the rewrite:

- **pest** — a PEG (Parsing Expression Grammar) parser using packrat parsing (top-down)
- **LALRPOP** — an EBNF parser using LR(1) parsing (bottom-up)

### Key Differences

| Aspect | pest (PEG) | LALRPOP (EBNF) |
|--------|-----------|----------------|
| Ambiguity Resolution | Ordered choice (first match wins) | LR(1) lookahead + conflict resolution |
| Learning Curve | Lower — reads like regex | Higher — requires LR parsing theory |
| Error Messages | Excellent defaults | Good, requires more configuration |
| AST Building | Separate builder functions | Inline actions in grammar |
| Debugging | Trace execution path | Shift/reduce conflict analysis |

### C-Like Language Considerations

C and its derivatives were historically designed alongside YACC, an LR parser generator. This has implications:

- Left recursion is natural in C-like syntax (e.g., `expr + expr`) — efficient in LR, problematic in PEG
- Operator precedence aligns with LR grammar structure
- Statement structure (semicolons, braces) is naturally LR-friendly

PEG workarounds for C-like syntax include eliminating left recursion (which changes the natural grammar structure), encoding precedence through rule ordering, and accepting backtracking overhead for ambiguous constructs.

### Decision

While pest offers a simpler learning curve, the technical fit of LR parsing for C-like languages is significant. The project initially chose pest for ease of use, then reconsidered in favor of rust-peg (a PEG parser with left-recursion support via the `precedence!` macro), which provides the best of both worlds — PEG simplicity with efficient precedence handling.

## Examples

### Cast Expression Resolution

```pest
primary = _{
    cast_expr      // Try cast first
    | paren_expr   // Then parenthesized expression
    | literal
    | ident
}

cast_expr = { "(" ~ type_expr ~ ")" ~ "(" ~ expr ~ ")" }
paren_expr = { "(" ~ expr ~ ")" }
```

The ordered choice operator (`/` in PEG, `|` in pest with `_{}`) tries alternatives in order. If `cast_expr` matches the `(Type)(expr)` pattern, it succeeds. Otherwise, pest backtracks and tries `paren_expr`.

### Grammar Structure

```pest
// Whitespace and comments handled automatically
WHITESPACE = _{ " " | "\t" | "\r" | "\n" }
COMMENT = _{ "//" ~ (!"\n" ~ ANY)* | "/*" ~ (!"*/" ~ ANY)* ~ "*/" }

// Type rules
type_expr = { primitive_type | array_type | pointer_type | ident }
primitive_type = { "int" | "float" | "bool" | "char" | "void" }
```

## Formal Grammar

The pest grammar uses PEG notation where:
- `~` is sequence (concatenation)
- `|` is ordered choice (try left first)
- `*` is zero or more
- `+` is one or more
- `?` is optional
- `_{ }` marks silent rules (not in parse tree)
