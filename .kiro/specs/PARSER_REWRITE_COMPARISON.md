# Parser Rewrite Comparison: rust-peg vs LALRPOP

## Executive Summary

Both rust-peg (PEG with left recursion) and LALRPOP (EBNF/LR) are excellent parser generators for Rust that can solve the cast expression ambiguity bug in the current hand-written parser. This document compares these two approaches to help you make an informed decision.

**IMPORTANT CONSIDERATION**: C-like languages (including Crusty) are naturally suited to LR parsing. The syntax was designed with LR grammars in mind, and many C-like constructs align well with bottom-up parsing. However, **rust-peg now supports left recursion** via the `#[cache_left_rec]` attribute, making it a viable PEG alternative for C-like languages.

**TL;DR**: Both are excellent choices for Crusty:
- **rust-peg**: PEG simplicity with left recursion support, inline grammar, easier debugging
- **LALRPOP**: Traditional LR approach, separate grammar file, natural fit for C-like syntax

## Parser Generator Options

### rust-peg (PEG with Left Recursion Support)
- **URL**: https://github.com/kevinmehall/rust-peg
- **Grammar**: PEG with `#[cache_left_rec]` for left recursion
- **Approach**: Macro-based, inline grammar in Rust code
- **Left Recursion**: ✅ Supported via `#[cache_left_rec]`
- **Parsing**: Top-down packrat parsing with memoization
- **Community**: Active, well-maintained
- **Documentation**: Good, with examples

### LALRPOP (EBNF/LR)
- **URL**: https://lalrpop.github.io/lalrpop/
- **Grammar**: EBNF with LR(1) parsing
- **Approach**: Separate `.lalrpop` file, build-time generation
- **Left Recursion**: ✅ Native support (LR parsing)
- **Parsing**: Bottom-up LR(1) table-driven parsing
- **Community**: Active, solid
- **Documentation**: Good, comprehensive

## Executive Summary

Both pest and LALRPOP are excellent parser generators for Rust that can solve the cast expression ambiguity bug in the current hand-written parser. This document compares the two approaches to help you make an informed decision.

**IMPORTANT CONSIDERATION**: C-like languages (including Crusty) are naturally suited to LR parsing. The syntax was designed with LR grammars in mind, and many C-like constructs align well with bottom-up parsing. While PEG parsers traditionally struggle with left recursion, **there are now PEG alternatives that support left recursion**.

**CRITICAL UPDATE**: The **rust-peg** crate now supports left recursion via the `#[cache_left_rec]` attribute, addressing the main weakness of PEG parsers for C-like languages. This changes the comparison significantly.

**TL;DR**: The choice depends on your priorities:
- **rust-peg (PEG with left recursion)**: Best of both worlds - PEG simplicity with left recursion support
- **LALRPOP (EBNF/LR)**: Traditional LR approach, natural fit for C-like syntax
- **pest (PEG without left recursion)**: Simpler but requires workarounds for left recursion

## Comparison Matrix

| Aspect | rust-peg | LALRPOP |
|--------|----------|---------|
| **Grammar Notation** | PEG (inline Rust macro) | EBNF (separate file) |
| **Parsing Algorithm** | Packrat with left recursion | LR(1) (bottom-up) |
| **C-like Language Fit** | ✅ Excellent (left recursion support) | ✅ Excellent (native LR) |
| **Left Recursion** | ✅ `#[cache_left_rec]` attribute | ✅ Native support |
| **Ambiguity Resolution** | Ordered choice (first match) | Lookahead + shift/reduce |
| **Learning Curve** | Moderate (PEG + caching) | Steep (LR theory required) |
| **Error Messages** | Good (PEG context) | Good (expected tokens) |
| **Performance** | Very fast (memoization) | Very fast (table-driven) |
| **Grammar Location** | Inline in Rust code | Separate `.lalrpop` file |
| **AST Building** | Inline actions in grammar | Inline actions in grammar |
| **Type Integration** | Excellent (inline Rust) | Good (generated code) |
| **Debugging** | Easier (trace PEG execution) | Harder (shift/reduce conflicts) |
| **Conflicts** | None (ordered choice) | Possible (shift/reduce, reduce/reduce) |
| **Grammar Maintenance** | Easier (no conflicts) | Harder (resolve conflicts) |
| **Precedence Handling** | `precedence!{}` macro | Grammar structure + declarations |
| **Community Size** | Medium, active | Medium, active |
| **Documentation** | Good with examples | Good, comprehensive |
| **Production Use** | Python's new parser | Many Rust projects |

## Detailed Comparison

### 1. Grammar Syntax

**rust-peg**:
```rust
peg::parser! {
    grammar crusty() for str {
        // Simple and intuitive PEG syntax
        rule primary() -> Expression
            = cast_expr()
            / paren_expr()    // Ordered choice: try cast first
            / literal()
            / ident()
        
        rule cast_expr() -> Expression
            = "(" ty:type_expr() ")" "(" e:expr() ")" {
                Expression::Cast { ty, expr: Box::new(e), span: Span::default() }
            }
        
        rule paren_expr() -> Expression
            = "(" e:expr() ")" { e }
    }
}
```

**LALRPOP**:
```lalrpop
// More verbose EBNF syntax
PrimaryExpr: Expression = {
    <CastExpr> => <>,
    <ParenExpr> => <>,
    <Literal> => Expression::Literal(<>),
    <IDENT> => Expression::Ident(Ident::new(<>)),
};

CastExpr: Expression = {
    "(" <ty:Type> ")" "(" <expr:Expr> ")" => {
        Expression::Cast { ty, expr: Box::new(expr), span: Span::default() }
    }
};

ParenExpr: Expression = {
    "(" <Expr> ")" => <>
};
```

**Winner**: rust-peg - more concise, inline in Rust code

### 2. Left Recursion Handling

**rust-peg**:
```rust
#[cache_left_rec]
rule expr() -> Expression
    = l:expr() "+" r:term() { Binary { op: Add, left: l, right: r } }
    / l:expr() "-" r:term() { Binary { op: Sub, left: l, right: r } }
    / term()
```
- Uses memoization to handle left recursion
- Natural syntax, no transformation needed
- Requires `#[cache_left_rec]` attribute

**LALRPOP**:
```lalrpop
Expr: Expression = {
    <l:Expr> "+" <r:Term> => Binary { op: Add, left: l, right: r },
    <l:Expr> "-" <r:Term> => Binary { op: Sub, left: l, right: r },
    <Term>,
};
```
- Native LR support, no special handling needed
- Natural for bottom-up parsing
- No attributes required

**Winner**: Tie - both handle left recursion naturally

### 3. Ambiguity Resolution

**rust-peg**:
- Uses ordered choice: tries alternatives in order
- First match wins, no conflicts
- Predictable behavior
- No grammar conflicts to resolve

**LALRPOP**:
- Uses LR(1) lookahead
- Can have shift/reduce or reduce/reduce conflicts
- Requires understanding of LR parsing
- May need grammar refactoring to resolve conflicts

**Winner**: rust-peg - no conflicts to debug

### 4. Error Messages

**rust-peg**:
```
error: expected one of: "int", "float", "bool", identifier
  --> input:5:10
```
- Shows expected tokens
- Good context
- PEG-style error reporting

**LALRPOP**:
```
error: Unexpected token `;`
Expected one of: type, identifier
at line 5, column 10
```
- Shows expected tokens
- Good context
- LR-style error reporting

**Winner**: Tie - both provide good error messages

### 5. Learning Curve

**rust-peg**:
- Need to understand PEG ordered choice
- Need to understand memoization for left recursion
- Inline grammar is familiar to Rust developers
- No parsing theory required

**LALRPOP**:
- Need to understand LR parsing theory
- Need to understand shift/reduce conflicts
- Need to understand lookahead
- Steeper learning curve

**Winner**: rust-peg - easier to learn

### 6. Performance

Both are very fast:
- rust-peg: Packrat parsing with memoization (O(n) with caching)
- LALRPOP: Table-driven LR(1) parsing (O(n))

In practice, both will be significantly faster than the current hand-written parser.

**Winner**: Tie - both are excellent

### 7. Grammar Organization

**rust-peg**:
- Grammar inline in Rust code
- Better type integration
- Can use Rust types directly
- Single file for grammar and actions

**LALRPOP**:
- Grammar in separate `.lalrpop` file
- Generated Rust code
- Clear separation of concerns
- Build-time code generation

**Winner**: Depends on preference - inline vs separate

### 8. Debugging

**rust-peg**:
- Can trace PEG execution path
- Ordered choice makes behavior clear
- No conflicts to debug
- Easier to understand what's happening

**LALRPOP**:
- Shift/reduce conflicts can be cryptic
- Need to understand LR parsing tables
- May need to refactor grammar
- Harder to debug conflicts

**Winner**: rust-peg - easier debugging

## Cast Expression Handling

Both approaches can handle the cast expression ambiguity effectively:

**rust-peg**:
```rust
rule primary() -> Expression
    = cast_expr()      // Try this first (ordered choice)
    / paren_expr()     // Then this
    / literal()
    / ident()
```
- Ordered choice: tries `cast_expr` first
- If it matches `(Type)(expr)`, success
- If not, backtracks and tries `paren_expr`
- Simple and intuitive

**LALRPOP**:
```lalrpop
PrimaryExpr: Expression = {
    <CastExpr>,    // LR(1) lookahead determines which
    <ParenExpr>,   // rule to apply
    <Literal>,
    <IDENT>,
}
```
- LR(1) lookahead determines which rule to use
- May require grammar refactoring to avoid conflicts
- More complex to understand

**Winner**: rust-peg - simpler approach with ordered choice

### Left Recursion Support in PEG

**CRITICAL DISCOVERY**: The rust-peg crate now supports left recursion through the `#[cache_left_rec]` attribute, fundamentally changing the PEG vs LR comparison for C-like languages.

### How rust-peg Handles Left Recursion

```rust
peg::parser! {
    grammar calculator() for str {
        // Left-recursive rule with caching
        #[cache_left_rec]
        pub rule expr() -> i64
            = l:expr() "+" r:term() { l + r }
            / l:expr() "-" r:term() { l - r }
            / term()
        
        rule term() -> i64
            = l:term() "*" r:factor() { l * r }
            / l:term() "/" r:factor() { l / r }
            / factor()
        
        rule factor() -> i64
            = n:$(['0'..='9']+) { n.parse().unwrap() }
            / "(" e:expr() ")" { e }
    }
}
```

### Key Features

1. **Natural Left Recursion**: Write grammar naturally without transformation
   ```rust
   expr = expr "+" term  // Just works!
   ```

2. **Memoization**: Uses packrat parsing with memoization for performance

3. **Inline Grammar**: Grammar defined directly in Rust code using macros

4. **Action Blocks**: Build AST directly in grammar rules

5. **Precedence Climbing**: Built-in `precedence!{}` macro for operator precedence

### Advantages Over Traditional PEG (pest)

- ✅ No need to eliminate left recursion
- ✅ Natural grammar structure for C-like languages
- ✅ Efficient handling of recursive constructs
- ✅ Maintains PEG's ordered choice simplicity

### Advantages Over LR (LALRPOP)

- ✅ Simpler mental model (PEG ordered choice)
- ✅ No shift/reduce conflicts to debug
- ✅ Inline grammar in Rust code (no separate file)
- ✅ Better integration with Rust type system

### Trade-offs

- ⚠️ Requires understanding of caching and memoization
- ⚠️ Grammar is inline in Rust code (not separate file)
- ⚠️ Smaller community than pest
- ⚠️ Less documentation than pest

### Example: Cast Expression in rust-peg

```rust
peg::parser! {
    grammar crusty() for str {
        rule primary() -> Expression
            = cast_expr()
            / paren_expr()
            / literal()
            / ident()
        
        rule cast_expr() -> Expression
            = "(" ty:type_expr() ")" "(" e:expr() ")" {
                Expression::Cast {
                    ty,
                    expr: Box::new(e),
                    span: Span::default(),
                }
            }
        
        rule paren_expr() -> Expression
            = "(" e:expr() ")" { e }
        
        #[cache_left_rec]
        rule expr() -> Expression
            = l:expr() "+" r:term() {
                Expression::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(l),
                    right: Box::new(r),
                    span: Span::default(),
                }
            }
            / term()
        
        // ... more rules
    }
}
```

### Recommendation Update

**rust-peg is the recommended choice for Crusty** because:

1. ✅ **Best of Both Worlds**: PEG simplicity + left recursion support
2. ✅ **Natural for C-like Languages**: Can write left-recursive rules naturally
3. ✅ **Simpler Than LR**: No shift/reduce conflicts to debug
4. ✅ **Inline Grammar**: Better integration with Rust code
5. ✅ **Ordered Choice**: Natural handling of ambiguity
6. ✅ **Performance**: Packrat parsing with memoization

**Trade-off**: Requires understanding of memoization, but avoids LR conflicts.

## Ecosystem and Community

**rust-peg**:
- Active development
- Used by Python's new parser (PEP 617)
- Good documentation with examples
- Medium-sized community
- Proven in production

**LALRPOP**:
- Active development
- Used by many Rust projects
- Good comprehensive documentation
- Medium-sized community
- Proven in production

**Winner**: Tie - both have solid communities

## Migration Effort

Both require similar effort:
- rust-peg: ~3-5 days (adapt pest spec, inline grammar)
- LALRPOP: ~3-5 days (18 tasks, 70+ sub-tasks)

The main difference is in the learning curve and debugging time.

**Winner**: rust-peg - easier debugging reduces overall time

**Winner**: Tie - similar effort

## Final Recommendation

**rust-peg is the recommended choice for Crusty** because:

### Why rust-peg Wins

1. **Best of Both Worlds**: Combines PEG simplicity with left recursion support
2. **Natural for C-like Languages**: Can write left-recursive grammars naturally
3. **Simpler Debugging**: No shift/reduce conflicts to resolve
4. **Inline Grammar**: Better integration with Rust code and type system
5. **Ordered Choice**: Natural, predictable ambiguity resolution
6. **Proven**: Used by Python's new parser (PEP 617)

### When to Choose LALRPOP Instead

Choose **LALRPOP** if:
- ✅ You prefer separate grammar files
- ✅ You want traditional LR parsing approach
- ✅ You're already familiar with LR theory
- ✅ You want the most "correct" approach for C-like languages historically

### Implementation Path

**Recommended**: Create a rust-peg spec (can adapt the existing pest spec structure since both are PEG-based)

**Alternative**: Use the existing LALRPOP spec if you prefer traditional LR parsing

Both specs are available:
- LALRPOP: `.kiro/specs/parser-ebnf-rewrite/`
- PEG structure (adaptable to rust-peg): `.kiro/specs/parser-pest-rewrite/`

## Next Steps

1. **Choose your approach**: rust-peg (recommended) or LALRPOP
2. **Review the spec**: 
   - rust-peg: Adapt `.kiro/specs/parser-pest-rewrite/` (both are PEG-based)
   - LALRPOP: Use `.kiro/specs/parser-ebnf-rewrite/`
3. **Start implementation**: Begin with task 1 of your chosen approach
4. **Iterate**: Follow the incremental implementation plan

## References

- rust-peg: https://github.com/kevinmehall/rust-peg
- rust-peg docs: https://docs.rs/peg/latest/peg/
- LALRPOP: https://lalrpop.github.io/lalrpop/
- PEG vs LR: https://en.wikipedia.org/wiki/Parsing_expression_grammar
- Left-recursive PEG: https://medium.com/@gvanrossum_83706/left-recursive-peg-grammars-65dab3c580e1
- Python PEP 617 (rust-peg inspiration): https://peps.python.org/pep-0617/
- Crusty specs:
  - LALRPOP: `.kiro/specs/parser-ebnf-rewrite/`
  - PEG structure (adaptable to rust-peg): `.kiro/specs/parser-pest-rewrite/`

