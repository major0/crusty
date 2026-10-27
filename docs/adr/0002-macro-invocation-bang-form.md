# ADR-0002: Macro invocation uses the bang form `name!(…)` only

## Status
accepted (supersedes the invocation-form row of ADR-0001)

## Context

ADR-0001 established the Crusty macro surface. Its decision table listed macro
**invocation** as accepting two forms:

| `__name__(args)` / `name!(…)` | `name!(args)` | Invocation. Delimiter preserved. |

The `__name__(args)` form was never implemented, and the attempt to use it
during an end-to-end transpile revealed why it cannot be: at a call site
`__name__(args)` is **lexically identical** to an ordinary function call
`identifier(args)`. The PEG grammar's `macro_call` rule keys on a literal `!`;
without it, `__square__(side)` parses as a plain `Expression::Call` on an
identifier named `__square__` and is rejected by semantic analysis as an
undefined, non-callable symbol.

This is not a missing feature — it is a syntactic ambiguity with no principled
resolution under Crusty's design:

- **Vague rules.** `__name__` is a legal identifier. The only thing separating a
  macro invocation from a function call would be a naming convention, not the
  grammar. The parser would have to special-case a `__`-prefix (making
  `__`-named functions ambiguous or uncallable) or defer the decision to
  semantic analysis, pushing the vagueness downstream.
- **Shadow declarations.** Nothing prevents a real binding named `__square__`
  (a `let`, a parameter, a function) from coexisting with a `#macro __SQUARE__`.
  At `__square__(x)` there is no defined winner. C makes the bare-call form
  well-defined only because its preprocessor performs textual substitution in a
  phase *before* name resolution — the exact phase-ordering machinery ADR-0001
  deliberately refused to reimplement.

You cannot have the bare-identifier call syntax without the substitution model
that disambiguates it, and that model is out of scope by prior decision.

Definitions do not have this problem: `#define __NAME__` and `#macro __NAME__`
are declaration sites where the sigil adds no ambiguity. The ambiguity is
specific to the call site.

## Decision

We will support exactly one macro invocation form: the Rust-native bang form
`name!(…)`. The `__name__(…)` invocation form is removed from the language.

The invocation row of ADR-0001's decision table is superseded by:

| Crusty syntax | Emits        | Notes                                                         |
|---------------|--------------|---------------------------------------------------------------|
| `name!(args)` | `name!(args)` | Invocation. Delimiter (`()`/`[]`/`{}`) preserved; body passed through untouched. |

Every other decision in ADR-0001 (the `#define` → `const` shim, the `#macro` →
`macro_rules!` shim, `#ifdef`/`#ifndef` → `#[cfg(…)]`, raw-span bodies) stands
unchanged.

The asymmetry is intentional and mirrors C: **define** with a sigil
(`#define` / `#macro __NAME__`), **invoke** with the native bang form. The `!`
restores grammatical disambiguation for free, exactly as it does in Rust — a
macro call can never be confused with a function call or shadowed by a binding.

## Consequences

Easier:

- Macro invocation is grammatically unambiguous: the `!` is a hard syntactic
  marker, so no naming-convention rule and no semantic-analysis fallback are
  needed to tell a macro call from a function call.
- Rust macro calls (`vec![…]`, `println!(…)`, `quote!{…}`) pass through the
  grammar untouched — the thin-syntax-layer goal of ADR-0001 holds for calls as
  well as definitions.

Harder / accepted costs:

- The `__name__` symmetry between definition and invocation is lost; authors
  define with `#macro __MAX__(…)` but call with `max!(…)`. This is a deliberate
  breaking change, acceptable under the project's "syntax is unstable and
  evolving" status.
- Bidirectional (Rust → Crusty) mapping of a macro call is now the identity
  (the call passes through unchanged), rather than rewriting to `__name__(…)`.

## Verification

- [ ] A macro invocation `square!(side)` transpiles to `square!(side)` with the delimiter preserved.
- [ ] A bracket invocation `vec![1, 2, 3]` transpiles to `vec![1, 2, 3]`.
- [ ] A bare `__square__(side)` does NOT parse as a macro call (it is a plain function call and is rejected by semantic analysis as undefined).
- [ ] No committed doc (`README.md`, `docs/syntax/**`, `docs/design/**`) documents `__name__(…)` as a macro invocation form; `__rust__{ … }` (the raw-Rust escape hatch) is unaffected.
- [ ] ADR-0001's `Status` records that its invocation-form row is superseded by this ADR.
