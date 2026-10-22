# ADR-0001: Macro handling — pass through Rust's macro system, shim only common C patterns

## Status

accepted

## Context

Crusty is a syntax layer over Rust, not a semantic reimplementation (see
`docs/design/philosophy.md`). Macros are the one area where the shipped
implementation most violates that principle.

The current design emulates the C preprocessor: a single `#define` directive
serves both constants and function-like macros, and macro bodies are captured as
`Vec<crate::lexer::Token>` — a raw token stream produced by a bespoke 212-line
hand tokenizer (`parse_macro_body_tokens`) living inside the PEG parser. This
causes concrete, measurable friction:

- ~12 macro-specific grammar rules plus the 212-line tokenizer dominate the
  parser's complexity, while the core language grammar is comparatively small
  and clean.
- The macro body as untyped token soup is "dumb data forcing smart code": the
  semantic analyzer is "lenient … since macro bodies are token sequences" and
  code generation has to re-interpret the blob.
- Every test-vs-design contradiction found while wiring in the PEG parser was
  macro-related (`#define [a,b]` / `{a,b}` delimiters, bracket macro-calls, the
  never-constructed `MacroDelimiter::{Brackets, Braces}` variants). None of the
  core grammar features produced contradictions.
- The macro body type (`Vec<Token>`) is the only remaining reason the `Token` /
  `TokenKind` types — and transitively the now-unused `Lexer` tokenizer — are
  kept after the parser moved to a `&str`-based PEG grammar.

Rust's macro facilities (`macro_rules!`, derive, attribute, and procedural
macros) are far more expressive than the C preprocessor. Emulating a weaker CPP
model on top of a stronger host is effort spent making the language worse.

## Decision

We will pass Rust's macro system through unchanged and provide only a small,
unambiguous set of C-flavoured shims. Macro bodies are captured as a **raw source
span (`String`)**, never tokenized by Crusty.

The complete Crusty macro surface is:

| Crusty syntax                 | Emits                        | Notes                                   |
|-------------------------------|------------------------------|-----------------------------------------|
| `#define __NAME__ value`      | `const NAME: _ = value;`     | Constants only. No parameters. Typed by Rust, not text-substituted. |
| `#macro __NAME__(a, b) body`  | `macro_rules! NAME { … }`    | The function-like shim. Body is a raw span passed through. |
| `#ifdef FOO` / `#endif`       | `#[cfg(FOO)]`                | Bare cfg flag. `#ifndef FOO` → `#[cfg(not(FOO))]`. |
| `__name__(args)` / `name!(…)` | `name!(args)`                | Invocation. Delimiter (`()`/`[]`/`{}`) preserved; body passed through untouched. |
| anything else                 | passed through               | Raw `macro_rules!`, derives, proc-macros, attribute macros are NOT modelled — they already work via attributes and the `__rust__` escape hatch. |

Boundary rules:

- `#define` is **constants only** and lowers to a Rust `const`, not a macro. A
  `#define` with a parameter list is a parse error directing the author to
  `#macro`.
- `#macro` is the **only** parameterised-macro definition shim and always uses
  `(params)`. Bracket/brace delimiters are not accepted in a definition.
- Macro **invocation** accepts `()`, `[]`, and `{}` (matching Rust's
  `vec![]` / `quote!{}` conventions) and preserves the delimiter; the argument
  text is an opaque span handed to rustc.
- `#ifdef`/`#ifndef` map to bare `cfg` flags only. `#[cfg(feature = "…")]`,
  target predicates, and other cfg forms are out of scope — authors use a raw
  attribute or `__rust__` for those.

## Consequences

Easier:

- The macro grammar collapses to a handful of rules (`#define`, `#macro`,
  `#ifdef`/`#ifndef`/`#endif`, and invocation-naming), each unambiguous.
- Deletes the 212-line `parse_macro_body_tokens` hand tokenizer and the
  `macro_args` / `macro_token_stream` / `macro_content` / `macro_content_part`
  rule cluster.
- `MacroDefinition.body` and `MacroCall` bodies become `String` (raw span)
  instead of `Vec<Token>`: smart data, dumb code.
- Removes the last consumer of `Token` / `TokenKind` and the `Lexer` tokenizer,
  which can then be deleted (≈1,150 further lines), finishing the parser-rewrite
  subtraction.
- Crusty gains the full power of Rust's macro system for free, by not blocking it.

Harder / accepted costs:

- `#define` with parameters (a valid C idiom) no longer works; authors write
  `#macro` instead. This is a deliberate, breaking syntax change — acceptable
  under the project's "syntax is unstable and evolving" status.
- Crusty no longer inspects macro bodies, so it cannot report Crusty-level errors
  inside a macro body; errors surface from rustc on the generated Rust. This is
  consistent with "syntax layer, not semantic reimplementation".
- Bidirectional (Rust → Crusty) round-tripping of macros is limited to
  re-emitting the raw body span. Acceptable: bidirectional transpilation is
  aspirational, not a Phase-1 deliverable.

## Verification

- [ ] `#define __PI__ 3.14159` transpiles to a Rust `const` (e.g. `const PI: f64 = 3.14159;`), not a macro.
- [ ] `#define __MAX__(a, b) …` (a `#define` with parameters) is a parse error whose message points the author to `#macro`.
- [ ] `#macro __MAX__(a, b) a > b ? a : b` transpiles to a `macro_rules! MAX` definition with the body passed through.
- [ ] `#ifdef FOO` / `#endif` around an item emits `#[cfg(FOO)]`; `#ifndef FOO` emits `#[cfg(not(FOO))]`.
- [ ] A macro invocation `__vec__[1, 2, 3]` transpiles to `vec![1, 2, 3]` with the bracket delimiter preserved.
- [ ] `src/ast.rs` no longer stores any macro body as `Vec<Token>`; macro bodies are `String`.
- [ ] `src/parser.rs` contains no `parse_macro_body_tokens` and no `macro_token_stream` / `macro_content` rules.
- [ ] `crate::lexer::Lexer` and (if no other consumer remains) `Token` / `TokenKind` are removed; `cargo build` is warning-clean.
- [ ] `cargo test` and `cargo clippy --all-targets` pass with zero failures and zero warnings.
