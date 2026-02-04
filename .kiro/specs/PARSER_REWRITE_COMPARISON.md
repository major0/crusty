# Parser Rewrite Comparison: PEG vs EBNF/LR

## Executive Summary

Both PEG and EBNF/LR parser generators for Rust can solve the cast expression ambiguity bug in the current hand-written parser. This document compares the approaches to help you make an informed decision.

**IMPORTANT CONSIDERATION**: C-like languages (including Crusty) are naturally suited to LR parsing. The syntax was designed with LR grammars in mind, and many C-like constructs align well with bottom-up parsing. While PEG parsers traditionally struggle with left recursion, **there are now PEG alternatives that support left recursion**.

**CRITICAL UPDATE**: The **rust-peg** crate now supports left recursion via the `#[cache_left_rec]` attribute, addressing the main weakness of PEG parsers for C-like languages. This changes the comparison significantly.

**TL;DR**: The choice depends on your priorities:
- **rust-peg (PEG with left recursion)**: Best of both worlds - PEG simplicity with left recursion support
- **LALRPOP (EBNF/LR)**: Traditional LR approach, natural fit for C-like syntax
- **pest (PEG without left recursion)**: Simpler but requires workarounds for left recursion

## Parser Generator Options

### 1. rust-peg (PEG with Left Recursion Support)
- **URL**: https://github.com/kevinmehall/rust-peg
- **Grammar**: PEG with `#[cache_left_rec]` for left recursion
- **Approach**: Macro-based, inline grammar in Rust code
- **Left Recursion**: ✅ Supported via `#[cache_left_rec]`
- **Community**: Active, well-maintained

### 2. LALRPOP (EBNF/LR)
- **URL**: https://lalrpop.github.io/lalrpop/
- **Grammar**: EBNF with LR(1) parsing
- **Approach**: Separate `.lalrpop` file, build-time generation
- **Left Recursion**: ✅ Native support (LR parsing)
- **Community**: Active, solid

### 3. pest (PEG without Left Recursion)
- **URL**: https://pest.rs/
- **Grammar**: PEG without left recursion support
- **Approach**: Separate `.pest` file, build-time generation
- **Left Recursion**: ❌ Not supported (requires workarounds)
- **Community**: Large, very active

## Executive Summary

Both pest and LALRPOP are excellent parser generators for Rust that can solve the cast expression ambiguity bug in the current hand-written parser. This document compares the two approaches to help you make an informed decision.

**IMPORTANT CONSIDERATION**: C-like languages (including Crusty) are naturally suited to LR parsing. The syntax was designed with LR grammars in mind, and many C-like constructs align well with bottom-up parsing. While PEG parsers traditionally struggle with left recursion, **there are now PEG alternatives that support left recursion**.

**CRITICAL UPDATE**: The **rust-peg** crate now supports left recursion via the `#[cache_left_rec]` attribute, addressing the main weakness of PEG parsers for C-like languages. This changes the comparison significantly.

**TL;DR**: The choice depends on your priorities:
- **rust-peg (PEG with left recursion)**: Best of both worlds - PEG simplicity with left recursion support
- **LALRPOP (EBNF/LR)**: Traditional LR approach, natural fit for C-like syntax
- **pest (PEG without left recursion)**: Simpler but requires workarounds for left recursion

## Comparison Matrix

| Aspect | rust-peg (PEG+LR) | LALRPOP (EBNF) | pest (PEG) |
|--------|-------------------|----------------|------------|
| **Grammar Notation** | PEG (inline Rust macro) | EBNF (separate file) | PEG (separate file) |
| **Parsing Algorithm** | Packrat with left recursion | LR(1) (bottom-up) | Packrat (top-down) |
| **C-like Language Fit** | ✅ Excellent (left recursion support) | ✅ Excellent (native LR) | ⚠️ Requires workarounds |
| **Left Recursion** | ✅ `#[cache_left_rec]` attribute | ✅ Native support | ❌ Not supported |
| **Ambiguity Resolution** | Ordered choice | Lookahead + conflicts | Ordered choice |
| **Learning Curve** | Moderate (PEG + caching) | Steep (LR theory) | Easy (pure PEG) |
| **Error Messages** | Good | Good | Excellent |
| **Performance** | Very fast (memoization) | Very fast (table-driven) | Very fast (memoization) |
| **Grammar Location** | Inline in Rust code | Separate `.lalrpop` file | Separate `.pest` file |
| **AST Building** | Inline actions | Inline actions | Separate builder |
| **Community** | Active, solid | Active, solid | Large, very active |
| **Documentation** | Good | Good | Excellent |
| **Maintenance** | Moderate | Harder (LR conflicts) | Easier (no conflicts) |
| **Debugging** | Moderate | Harder (shift/reduce) | Easier (trace execution) |

## Detailed Comparison

### 1. Grammar Syntax

**pest (PEG)**:
```pest
// Simple and intuitive
primary = _{ 
    cast_expr      // Try cast first
    | paren_expr   // Then parenthesized
    | literal
    | ident
}

cast_expr = { "(" ~ type_expr ~ ")" ~ "(" ~ expr ~ ")" }
paren_expr = { "(" ~ expr ~ ")" }
```

**LALRPOP (EBNF)**:
```lalrpop
// More verbose, requires understanding precedence
PrimaryExpr: Expression = {
    <CastExpr> => <>,
    <ParenExpr> => <>,
    <Literal> => Expression::Literal(<>),
    <IDENT> => Expression::Ident(Ident::new(<>)),
};

CastExpr: Expression = {
    "(" <ty:Type> ")" "(" <expr:Expr> ")" => {
        Expression::Cast { ty, expr: Box::new(expr), ... }
    }
};
```

**Winner**: pest - simpler, more readable syntax

### 2. Ambiguity Resolution

**pest (PEG)**:
- Uses ordered choice: tries alternatives in order
- First match wins, no conflicts
- Natural for handling cast vs. parenthesized expression
- No need to understand parsing theory

**LALRPOP (EBNF)**:
- Uses LR(1) lookahead
- Can have shift/reduce or reduce/reduce conflicts
- Requires understanding of LR parsing to resolve conflicts
- May need to refactor grammar to eliminate conflicts

**Winner**: pest - ordered choice is simpler and more intuitive

### 3. Error Messages

**pest (PEG)**:
```
Error: expected one of: "int", "float", "bool", identifier
  --> file.crst:5:10
   |
 5 |     let x = (;
   |             ^
```

**LALRPOP (EBNF)**:
```
Error: Unexpected token `;`
Expected one of: type, identifier
at line 5, column 10
```

Both provide good error messages, but pest's are slightly better formatted out of the box.

**Winner**: pest - slightly better default error messages

### 4. Learning Curve

**pest (PEG)**:
- Grammar syntax is intuitive (reads like regex)
- Ordered choice is easy to understand
- No need to learn parsing theory
- Good documentation and examples

**LALRPOP (EBNF)**:
- Requires understanding LR parsing
- Need to understand shift/reduce conflicts
- Precedence declarations can be tricky
- Less documentation available

**Winner**: pest - much easier to learn

### 5. Performance

Both are very fast:
- pest uses packrat parsing with memoization
- LALRPOP uses table-driven LR(1) parsing

In practice, both will be faster than the current hand-written parser.

**Winner**: Tie - both are excellent

### 6. AST Building

**pest (PEG)**:
- Separate AST builder functions
- Clean separation of grammar and code
- More verbose but more maintainable

**LALRPOP (EBNF)**:
- Inline actions in grammar
- More concise
- Grammar and code are mixed

**Winner**: pest - better separation of concerns

### 7. Maintenance

**pest (PEG)**:
- Grammar is self-documenting
- Easy to add new rules
- No conflicts to resolve
- Community support is strong

**LALRPOP (EBNF)**:
- Grammar can have conflicts
- Adding rules may introduce conflicts
- Requires more expertise to maintain
- Smaller community

**Winner**: pest - easier to maintain

### 8. Debugging

**pest (PEG)**:
- Can trace execution path easily
- Ordered choice makes it clear what's happening
- Good error messages help debugging

**LALRPOP (EBNF)**:
- Shift/reduce conflicts can be hard to debug
- Need to understand LR parsing tables
- Less intuitive

**Winner**: pest - easier to debug

## C-Like Language Considerations

**CRITICAL INSIGHT**: This is the most important factor for Crusty.

### Why C-Like Languages Favor LR Parsing

C and its derivatives (C++, Java, Rust, and Crusty) were designed with LR parsing in mind:

1. **Historical Context**: C was developed alongside YACC (Yet Another Compiler Compiler), an LALR parser generator. The language syntax was explicitly designed to be parseable by LR grammars.

2. **Left Recursion**: C-like languages use left-recursive constructs naturally:
   ```c
   expr -> expr + term    // Left recursive
   expr -> expr * term    // Left recursive
   ```
   - LR parsers handle left recursion efficiently (O(n))
   - PEG parsers struggle with left recursion (requires workarounds or can cause infinite loops)

3. **Operator Precedence**: C-like operator precedence aligns perfectly with LR grammar structure:
   ```lalrpop
   Expr = AssignExpr;
   AssignExpr = OrExpr ("=" OrExpr)?;
   OrExpr = AndExpr ("||" AndExpr)*;
   // Natural bottom-up precedence
   ```

4. **Statement Structure**: C-like statement syntax is naturally LR:
   - Statements end with semicolons (clear boundaries)
   - Blocks use braces (clear nesting)
   - Declarations have predictable structure

### PEG Workarounds for C-Like Syntax

PEG parsers can handle C-like syntax, but require workarounds:

1. **Left Recursion Elimination**: Must rewrite left-recursive rules
   ```pest
   // Can't write: expr = expr "+" term
   // Must write: expr = term ("+" term)*
   ```
   This changes the natural structure and can complicate AST building.

2. **Precedence Through Ordering**: Must carefully order alternatives
   ```pest
   primary = _{ 
       cast_expr      // Order matters!
       | paren_expr   // Wrong order = wrong parse
       | literal
   }
   ```

3. **Backtracking Overhead**: PEG's ordered choice requires backtracking for ambiguous constructs, which can be less efficient than LR's lookahead.

### LALRPOP Advantages for Crusty

1. **Natural Grammar Structure**: Write grammar that matches language design
2. **Efficient Left Recursion**: No need to rewrite recursive rules
3. **Predictable Behavior**: LR(1) lookahead is deterministic
4. **Better Performance**: Table-driven parsing is faster for C-like syntax
5. **Alignment with Language Design**: Crusty inherits C's LR-friendly design

### PEG Advantages Despite C-Like Syntax

1. **Simpler Mental Model**: Ordered choice is easier to understand
2. **Better Error Messages**: More context in error reporting
3. **Easier Debugging**: Can trace execution path
4. **No Conflicts**: No shift/reduce or reduce/reduce conflicts to resolve
5. **Larger Community**: More help available

### The Trade-off

**LALRPOP**: Better technical fit for C-like languages, but steeper learning curve
**pest**: Easier to use and maintain, but requires workarounds for C-like constructs

### Recommendation Update

Given that Crusty is a C-like language:

**Choose LALRPOP (EBNF/LR) if**:
- ✅ You want the most natural fit for C-like syntax
- ✅ You want efficient handling of left recursion
- ✅ You want grammar that aligns with language design
- ✅ You're willing to learn LR parsing theory
- ✅ You want the best performance for C-like constructs

**Choose pest (PEG) if**:
- ✅ You prioritize ease of learning and maintenance
- ✅ You want better error messages and debugging
- ✅ You're okay with workarounds for left recursion
- ✅ You want a larger community and better documentation
- ✅ You prefer simpler grammar syntax

**Winner for C-like languages**: **LALRPOP** - better technical fit, despite higher complexity

## rust-peg: The Game Changer

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

**rust-peg is now the recommended choice for Crusty** because:

1. ✅ **Best of Both Worlds**: PEG simplicity + left recursion support
2. ✅ **Natural for C-like Languages**: Can write left-recursive rules naturally
3. ✅ **Simpler Than LR**: No shift/reduce conflicts to debug
4. ✅ **Inline Grammar**: Better integration with Rust code
5. ✅ **Ordered Choice**: Natural handling of ambiguity
6. ✅ **Performance**: Packrat parsing with memoization

**Trade-off**: Smaller community and less documentation than pest, but the left recursion support makes it worth it for C-like languages.

## Cast Expression Handling

Both approaches can handle the cast expression ambiguity:

**pest (PEG)**:
```pest
primary = _{ 
    cast_expr      // Try this first
    | paren_expr   // Then this
    | ...
}
```
- Ordered choice: tries `cast_expr` first
- If it matches `(Type)(expr)`, success
- If not, backtracks and tries `paren_expr`
- Simple and intuitive

**LALRPOP (EBNF)**:
```lalrpop
PrimaryExpr = {
    <CastExpr>,    // LR(1) lookahead
    <ParenExpr>,   // determines which
    ...
}
```
- LR(1) lookahead determines which rule to use
- May require grammar refactoring to avoid conflicts
- More complex to understand

**Winner**: pest - simpler approach

## Ecosystem and Community

**pest**:
- 2.7k+ stars on GitHub
- Active development
- Used by many projects (including tree-sitter)
- Excellent documentation
- Large community
- Many examples available

**LALRPOP**:
- 2.9k+ stars on GitHub
- Active but slower development
- Used by fewer projects
- Good documentation but less extensive
- Smaller community
- Fewer examples

**Winner**: pest - larger, more active community

## Migration Effort

Both require similar effort:
- pest: ~3-5 days (21 tasks, 80+ sub-tasks)
- LALRPOP: ~3-5 days (18 tasks, 70+ sub-tasks)

The main difference is in the learning curve and debugging time.

**Winner**: Tie - similar effort

## Final Recommendation

**The decision depends on your priorities:**

### Best Overall: rust-peg Wins

For a C-like language like Crusty, **rust-peg is the best choice**:
- ✅ PEG simplicity with left recursion support
- ✅ Natural grammar structure for C-like languages
- ✅ No LR conflicts to debug
- ✅ Inline grammar in Rust code
- ✅ Ordered choice for ambiguity resolution
- ✅ Good performance with memoization

### Alternative 1: LALRPOP (Traditional LR)

Choose **LALRPOP** if:
- ✅ You prefer separate grammar files
- ✅ You want traditional LR parsing
- ✅ You're comfortable with shift/reduce conflicts
- ✅ You want the most "correct" approach for C-like languages

### Alternative 2: pest (Simple PEG)

Choose **pest** if:
- ✅ You want the largest community and best documentation
- ✅ You're okay with eliminating left recursion manually
- ✅ You prefer separate grammar files
- ✅ You want the simplest possible approach

### Updated Recommendation

**rust-peg is recommended for Crusty** because:

1. **Best of Both Worlds**: Combines PEG simplicity with left recursion support
2. **Natural Fit**: Can write C-like left-recursive grammars naturally
3. **Simpler**: No shift/reduce conflicts like LALRPOP
4. **Efficient**: Packrat parsing with memoization
5. **Integrated**: Inline grammar in Rust code with type safety
6. **Proven**: Used by production parsers including Python's new parser

**Trade-off**: Smaller community than pest, but the left recursion support is crucial for C-like languages.

**Implementation Path**: Create a third spec for rust-peg, or adapt the pest spec to use rust-peg instead (they're both PEG-based, so the grammar structure is similar).

## Next Steps

1. **Review both specs**: Read through both `.kiro/specs/parser-pest-rewrite/` and `.kiro/specs/parser-ebnf-rewrite/`
2. **Make a decision**: Choose pest or LALRPOP based on your preferences
3. **Start implementation**: Begin with task 1 of your chosen spec
4. **Iterate**: Follow the incremental implementation plan

## References

- rust-peg: https://github.com/kevinmehall/rust-peg
- rust-peg docs: https://docs.rs/peg/latest/peg/
- LALRPOP: https://lalrpop.github.io/lalrpop/
- pest: https://pest.rs/
- PEG vs LR: https://en.wikipedia.org/wiki/Parsing_expression_grammar
- Left-recursive PEG: https://medium.com/@gvanrossum_83706/left-recursive-peg-grammars-65dab3c580e1
- Crusty specs:
  - pest: `.kiro/specs/parser-pest-rewrite/` (can be adapted for rust-peg)
  - LALRPOP: `.kiro/specs/parser-ebnf-rewrite/`
