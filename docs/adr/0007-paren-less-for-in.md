# ADR-0007: Paren-less for-in with a struct-literal-restricted iterable

## Status
accepted

## Context
Crusty's for-in loop was grammared as `for ( IDENT in expr ) block`, mirroring
the parenthesized C-style `for (init; cond; incr)`, `while (cond)`, `if (cond)`,
and `switch (expr)` forms. The reasoning was surface consistency with C.

That reasoning was wrong. C's `for` parens delimit a three-part clause
(`init; cond; incr`) — they wrap an *expression region* with internal
semicolons. A for-in has no such clause: it is a binding and an iterable, the
exact shape of Rust's `for x in iter`, which C does not have. Wrapping
`(i in 0..10)` in parens delimits nothing — `i in 0..10` is not an expression
Crusty has elsewhere — so the parens are noise that makes the form read as
*less* like the C/Rust family it belongs to, not more.

Dropping the parens exposes a real grammar ambiguity that the parens were
masking. In `for item in collection { }`, a PEG parser cannot tell whether
`collection { }` is "iterable `collection`, then loop body `{ }`" or a struct
literal `collection {}` (Crusty's `StructInit` is `Ident { fields }`). The
greedy parse takes the struct literal and then fails, wanting a loop body.

This is the same ambiguity Rust has, and Rust resolves it the same way: in the
iterable position of `for` (and in `if`/`while` conditions) a struct literal is
not allowed unless parenthesized — the "restricted expression" / no-struct-literal
rule. Crusty's `if`/`while`/`switch` conditions are already parenthesized
(`"(" expr ")"`), so the `)` terminates the expression and they are NOT exposed
to this ambiguity. Only the paren-less for-in iterable is.

## Decision
We will make for-in paren-less — `for IDENT in iter block` — and restrict the
`iter` position to a struct-literal-free expression. A bare struct literal in
the iterable position must be parenthesized: `for p in (Point { x: 1 }) { }`,
exactly as Rust requires. The restriction is confined to the for-in iterable;
the already-parenthesized `if`/`while`/`switch` conditions are left untouched.

## Consequences
- **Easier:** for-in reads as `for i in 0..10 { }`, matching the Rust it
  transpiles to; one fewer paren pair; the surface is honestly Rust-family.
  The iterable accepts the full expression grammar — ranges, binary
  arithmetic (`for x in a + b { }`), comparisons, calls, method chains — with
  no parens.
- **Harder:** iterating a *freshly-constructed* struct literal needs parens.
  This is vanishingly rare (you almost never iterate a struct literal built
  in place), and the diagnostic points at it.
- **Accepted cost:** the expression precedence table is parameterised by a
  single `allow_struct` flag threaded to its one `primary` base case, so the
  iterable reuses the whole table (no duplication) with only the bare
  struct-literal atom suppressed. It generalizes if conditions are ever
  de-parenthesized later.
- A surface-syntax change: every for-in in docs, tests, and example apps moves
  from `for (x in it)` to `for x in it`.

## Verification
- [x] `for_in_stmt` grammar is `for IDENT in iter block`, no parens around the clause.
- [x] `for i in 0..10 { }`, `for x in xs { }`, `for e in m.iter() { }`, and
      `for x in a + b { }` all parse.
- [x] `for p in Point { x: 1 } { }` is rejected; `for p in (Point { x: 1 }) { }` parses.
- [x] `cargo test --lib --bins` is green (all for-in tests converted to paren-less).
- [x] No `for (IDENT in ...)` paren form remains in `docs/` or `examples/`.
