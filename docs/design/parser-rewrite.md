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

### Decision: pest

pest was chosen for Crusty because:

1. PEG ordered choice naturally resolves the cast expression ambiguity — try `(Type)(expr)` first, backtrack to `(expr)` if it fails
2. Grammar syntax is simpler and more maintainable
3. No shift/reduce conflicts to debug
4. Larger community and better documentation
5. Clean separation between grammar definition and AST construction

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
