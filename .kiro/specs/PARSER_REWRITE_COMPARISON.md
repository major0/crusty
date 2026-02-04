# Parser Rewrite Comparison: pest (PEG) vs LALRPOP (EBNF)

## Executive Summary

Both pest and LALRPOP are excellent parser generators for Rust that can solve the cast expression ambiguity bug in the current hand-written parser. This document compares the two approaches to help you make an informed decision.

**IMPORTANT CONSIDERATION**: C-like languages (including Crusty) are naturally suited to LR parsing. The syntax was designed with LR grammars in mind, and many C-like constructs align well with bottom-up parsing. While PEG parsers can handle C-like syntax through ordered choice and backtracking, LALRPOP's LR(1) approach may be more natural for this language family.

**TL;DR**: The choice depends on your priorities:
- **LALRPOP (EBNF/LR)**: Better fit for C-like syntax, more efficient for left-recursive grammars, natural alignment with language design
- **pest (PEG)**: Simpler to learn and maintain, better error messages, larger community, easier debugging

## Comparison Matrix

| Aspect | pest (PEG) | LALRPOP (EBNF) |
|--------|-----------|----------------|
| **Grammar Notation** | PEG (Parsing Expression Grammar) | EBNF (Extended Backus-Naur Form) |
| **Parsing Algorithm** | Packrat parsing (top-down) | LR(1) (bottom-up) |
| **C-like Language Fit** | Requires workarounds for left recursion | Natural fit (C designed for LR) |
| **Ambiguity Resolution** | Ordered choice (first match wins) | Lookahead + grammar conflicts |
| **Learning Curve** | Easier (more intuitive) | Steeper (requires understanding LR parsing) |
| **Error Messages** | Excellent (shows what was expected) | Good (but requires more work) |
| **Performance** | Very fast (memoization) | Very fast (table-driven) |
| **Grammar Complexity** | Simpler, more readable | More complex, requires precedence declarations |
| **Lexer Integration** | Separate (but integrated) | Integrated (regex in grammar) |
| **AST Building** | Separate builder functions | Inline actions in grammar |
| **Community** | Large, active | Smaller, but solid |
| **Documentation** | Excellent | Good |
| **Maintenance** | Easier (grammar is self-documenting) | Harder (need to understand LR conflicts) |
| **Debugging** | Easier (trace execution path) | Harder (shift/reduce conflicts) |

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

## Recommendation

**The decision depends on your priorities:**

### Technical Fit: LALRPOP Wins

For a C-like language like Crusty, **LALRPOP is the better technical fit**:
- ✅ C-like languages were designed for LR parsing
- ✅ Natural handling of left recursion
- ✅ Grammar structure aligns with language design
- ✅ More efficient for C-like operator precedence
- ✅ Predictable LR(1) behavior

### Ease of Use: pest Wins

For developer experience, **pest is easier**:
- ✅ Simpler grammar syntax
- ✅ Better error messages
- ✅ Easier to learn and debug
- ✅ Larger community
- ✅ No LR conflicts to resolve

### Final Recommendation

**LALRPOP is recommended for Crusty** because:

1. **Natural Fit**: C-like languages are designed for LR parsing
2. **Efficient**: Better performance for left-recursive constructs
3. **Aligned**: Grammar structure matches language design
4. **Predictable**: LR(1) lookahead is deterministic
5. **Professional**: Used by production compilers for C-like languages

**Trade-off**: You'll need to invest time learning LR parsing theory and debugging shift/reduce conflicts, but you'll get a parser that naturally fits the language design.

**Alternative**: If ease of use is more important than technical fit, choose pest. It can handle C-like syntax with workarounds, and the simpler mental model may be worth the trade-off.

## Next Steps

1. **Review both specs**: Read through both `.kiro/specs/parser-pest-rewrite/` and `.kiro/specs/parser-ebnf-rewrite/`
2. **Make a decision**: Choose pest or LALRPOP based on your preferences
3. **Start implementation**: Begin with task 1 of your chosen spec
4. **Iterate**: Follow the incremental implementation plan

## References

- pest: https://pest.rs/
- LALRPOP: https://lalrpop.github.io/lalrpop/
- PEG vs LR: https://en.wikipedia.org/wiki/Parsing_expression_grammar
- Crusty specs:
  - pest: `.kiro/specs/parser-pest-rewrite/`
  - LALRPOP: `.kiro/specs/parser-ebnf-rewrite/`
