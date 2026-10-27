# ADR-0030: Switch case values — comma-list and `...` ranges, no `|`, no stacking

## Status
proposed

## Context

Crusty has two matching constructs (ADR-0012): the C-style `switch` **statement**
and the Rust-flavoured `match` **expression**. Both lower to a Rust `match`. The
`switch` statement's case-value surface has three problems, all confirmed against
the current grammar and codegen (code in hand, not speculation):

1. **`case 1: case 2:` stacking implies a fallthrough that does not exist.** The
   stacked form is C's "share a body by falling through," but Crusty's codegen
   emits Rust `1 | 2 => body` — Rust has **no fallthrough at all** (no keyword,
   no implicit fall-through; shared bodies are pattern alternation). So the
   surface advertises a semantics the emitted code never had.

2. **`|` in a case value is a silent overload.** `case 1 | 2:` parses today and
   emits Rust `1 | 2 =>` — i.e. it is read as *pattern alternation* (two values),
   not as C's *bitwise-OR* (the single value `3`). A C programmer writing
   `case 1 | 2:` for the value `3` silently gets two cases. One syntax, two
   readings, wrong-by-default for a C mindset.

3. **A comma-list `case 1, 2, 3:` miscompiles today.** The `case_values` rule
   parses a single `expr()`, so `1, 2, 3` is swallowed as a *comma expression*
   and emits garbage (`{ { 1; 2 }; 3 } =>`). And a GCC-style range
   `case 1 ... 3:` does not parse at all.

Two C-family languages already solve "several values, one case" without
fallthrough and without overloading `|`:

- **Go**: `case 1, 2, 3:` — a comma-separated value list; no implicit
  fallthrough (each case auto-breaks; explicit `fallthrough` opts in).
- **GNU C** (GCC/Clang/Sun extension): `case 1 ... 5:` — an inclusive range
  (spaces around `...` required), behaving as if every value in `[1,5]` were a
  label.

Both map exactly onto Rust `match` patterns: a comma-list → `1 | 2 | 3 =>`, a
range → `1..=5 =>`. The AST already fits: `SwitchCase { values: Vec<Expression> }`
holds the list, and `Expression::Range { start, end, inclusive }` already exists.

## Decision

Crusty's `switch` statement accepts **exactly two** case-value spellings, and the
`|` operator is **rejected** in a case value (never reinterpreted):

```c
case 1, 2, 3:      // Go-style comma-list  -> Rust  1 | 2 | 3 =>
case 1 ... 5:      // GNU-C inclusive range -> Rust  1..=5 =>
case 1 ... 5, 10:  // the two may be mixed  -> Rust  1..=5 | 10 =>
```

- Each case value is a **single non-comma expression** or a `lo ... hi`
  **inclusive range**; values are comma-separated. The comma is the list
  separator, never the comma operator.
- `|` is **not** a case-value surface. Writing `case 1 | 2:` is a parse error,
  not a silent alternation and not a bitwise-OR. `|` keeps its sole meaning —
  bitwise-OR, inside ordinary value expressions elsewhere.
- The `case 1: case 2:` **stacking form is removed.** Its only purpose was
  fallthrough grouping, which neither Crusty nor Rust implements.
- `switch` cases do not fall through; each lowers to an independent `match` arm.
  There is no `fallthrough` keyword (Rust has none to lower to).

`match` is unchanged: it remains the Rust-flavoured value-yielding expression
with `=>` arms and `|`-alternation in *pattern* position (where `|` is
unambiguous because a pattern is not a value expression).

## Consequences

Easier:
- The `switch` surface tells the truth: no fallthrough syntax for a no-fallthrough
  lowering, and no `|` overload. `case 1, 2, 3:` and `case 1 ... 5:` are both
  real, familiar C-family spellings (Go and GNU C).
- The latent comma-expression miscompile (`case 1, 2:` → garbage) is fixed by
  construction — the list is parsed element-wise.
- AST and codegen are unchanged: `SwitchCase.values` already holds the list and
  the emitter already writes `|`-alternation to Rust; a range element is an
  existing `Expression::Range` that already emits `lo..=hi`.

Harder / accepted:
- **Breaking surface change.** Existing Crusty source using `case 1: case 2:`
  stacking or `case 1 | 2:` stops parsing and must move to the comma-list. The
  blast radius is small: no example app or doc uses either form, and the test
  corpus asserts only emitter output, not stacked/`|` input.
- The parser's `case_values` rule is rewritten (parse a comma-list of
  single-value-or-range, reject the comma operator and `|`); this is the only
  code change beyond tests and docs.

## Rejected alternatives

- **Keep `case 1 | 2:` alternation (the Rust spelling).** Rejected: it overloads
  `|`, which in a C-family value position reads as bitwise-OR; the silent
  "1-or-2 vs 3" ambiguity is a footgun, and it is live today.
- **Keep `case 1: case 2:` stacking as a deprecated alias.** Rejected: it keeps
  advertising a fallthrough that never happens, which is the core dishonesty this
  ADR removes. One clear spelling beats two where one lies.
- **Add an explicit `fallthrough` keyword (full Go parity).** Rejected for now:
  Rust has no fallthrough to lower to, so `fallthrough` would require Crusty to
  synthesize goto-like control flow — out of scope for a syntax-layer transpiler
  (ADR-0001) and unwanted per the project's no-fallthrough stance.

## Verification

- [ ] `case 1, 2, 3:` parses as three values and emits Rust `1 | 2 | 3 => …`
      (not a comma expression).
- [ ] `case 1 ... 5:` parses and emits Rust `1..=5 => …`.
- [ ] `case 1 ... 5, 10, 20 ... 25:` emits `1..=5 | 10 | 20..=25 => …`.
- [ ] `case 1 | 2:` is a **parse error** (|  rejected in a case value).
- [ ] `case 1: case 2:` stacking is a **parse error** (stacking removed).
- [ ] A `switch` with comma-list and range cases compiles under
      `rustc -D unused_parens` and runs with the expected branch selected.
- [ ] `docs/syntax/control-flow.md` documents the two spellings and the
      no-fallthrough / no-`|` rules; the full existing suite stays green.
