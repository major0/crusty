# Parser Rewrite Comparison: pest (PEG) vs LALRPOP (EBNF)

## Executive Summary

Both pest and LALRPOP are excellent parser generators for Rust that can solve the cast expression ambiguity bug in the current hand-written parser. This document compares the two approaches to help you make an informed decision.

**TL;DR Recommendation**: **pest** is recommended for Crusty because:
- Simpler grammar syntax (PEG is more intuitive than EBNF)
- Better error messages out of the box
- Easier to learn and maintain
- More active community and better documentation
- Ordered choice naturally handles ambiguity without conflicts

## Comparison Matrix

| Aspect | pest (PEG) | LALRPOP (EBNF) |
|--------|-----------|----------------|
| **Grammar Notation** | PEG (Parsing Expression Grammar) | EBNF (Extended Backus-Naur Form) |
| **Parsing Algorithm** | Packrat parsing (top-down) | LR(1) (bottom-up) |
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

**Choose pest (PEG) if**:
- ✅ You want simpler, more intuitive grammar syntax
- ✅ You want easier maintenance and debugging
- ✅ You want better error messages out of the box
- ✅ You want a larger community and better documentation
- ✅ You want to avoid learning LR parsing theory
- ✅ You prefer separation of grammar and AST building code

**Choose LALRPOP (EBNF) if**:
- ⚠️ You already know LR parsing theory
- ⚠️ You prefer inline actions in grammar
- ⚠️ You want to use EBNF notation specifically
- ⚠️ You need the absolute best performance (marginal difference)

## Final Recommendation

**pest is the better choice for Crusty** because:

1. **Simpler**: PEG ordered choice is more intuitive than LR conflicts
2. **Easier to maintain**: No shift/reduce conflicts to debug
3. **Better errors**: Excellent error messages out of the box
4. **Larger community**: More help available, more examples
5. **Better docs**: Comprehensive documentation and tutorials
6. **Natural fit**: Ordered choice naturally handles cast expression ambiguity

The only advantage of LALRPOP is if you already have deep knowledge of LR parsing and prefer EBNF notation. For most developers, pest will be easier to learn, use, and maintain.

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
