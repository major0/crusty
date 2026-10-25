# ADR-0020: Verbatim pass-through for `#[…]` and `#![…]` attributes

## Status
proposed

## Context

Crusty uses `#` for two unrelated things:

1. **Preprocessor directives** that Crusty *owns* and lowers itself — `#define`,
   `#macro`, `#ifdef`/`#ifndef`…`#endif`, `#import`, `#export` (and the
   explicitly-rejected `#include`/`#use`). These are Crusty surface forms with
   Crusty semantics.
2. **Rust attributes** `#[…]`, which Crusty does not interpret — they are Rust's,
   and under ADR-0001 (syntax layer; defer semantics to rustc) Crusty's only job
   is to carry them through to the generated Rust unchanged.

Today the second case is handled by *parsing the attribute structurally* into
`Attribute { name: Ident, args: Vec<AttributeArg> }`, where
`AttributeArg = Ident | Literal | NameValue { name, value }`. This models only a
flat argument list, which is strictly less than Rust's attribute grammar allows:

- **Nested meta fails**: `#[cfg(all(unix, feature = "x"))]`, `#[serde(rename_all = "camelCase", default)]`, `#[derive(Serialize)] #[serde(tag = "type")]` with nested parens — none parse, because `attribute_arg` admits no sub-parens and no token trees.
- **Crate-level inner attributes are entirely absent**: there is no `#![…]` rule.
  `#![no_std]`, `#![no_main]`, `#![feature(…)]`, `#![allow(…)]` cannot be
  expressed — and (unlike every other gap) they cannot be reached via `__rust__`
  either, because inner attributes must be the crate's *first tokens* and cannot
  be nested inside a `__rust__` item block. This is the one HARD gap in the
  language (gap analysis, ADR-0001 escape-hatch review).

The structured model buys nothing: **no downstream code inspects attribute
arguments**. Codegen's only use of `Attribute.args` is to print them straight
back out (three near-identical emission sites). Parsing them into a typed AST and
re-serialising them is smart-code over dumb-data — the opposite of the
`Literal::Raw` precedent (PS-05), where load-bearing source text is carried
verbatim and rustc arbitrates.

The user's framing: pass through *all* `#…` constructs that do not overlap with
the owned directive grammar. That is exactly the `#[…]` / `#![…]` family — and
the overlap test is trivial, because no directive keyword begins with `[` or `!`.

## Decision

We will carry Rust attributes through **verbatim** and drop the structured
attribute-argument model.

1. **`#[…]` outer attributes** parse as a single verbatim bracket-span. The
   `Attribute` AST becomes `Attribute { text: String }` holding the exact source
   between (and including) `#[` and the matching `]`. Codegen emits that text
   unchanged. The `AttributeArg` enum and its three-way codegen match are
   **removed**.

2. **`#![…]` inner attributes** become a new item form that captures the
   `#![ … ]` span verbatim and emits it unchanged at crate top level, so
   `#![no_std]`, `#![feature(…)]`, `#![allow(…)]` are expressible. Emission must
   place inner attributes ahead of other generated items (rustc requires inner
   attributes before any item), which the item ordering already preserves when
   they are written first in source.

3. **Disambiguation** is by the character after `#`: `[` → outer attribute,
   `![` → inner attribute, otherwise a directive keyword
   (`define`/`macro`/`ifdef`/`ifndef`/`import`/`export`, or the rejected
   `include`/`use`). These cannot collide — no directive keyword starts with `[`
   or `!`. The owned preprocessor directives are unchanged by this ADR.

The bracket span is captured with balanced-bracket matching so nested `[...]`
inside an attribute (rare but legal, e.g. in token-tree arguments) is carried
whole rather than terminating at the first `]`.

## Consequences

**Easier.**
- *Every* Rust attribute works immediately — nested meta, token-tree arguments,
  multiple derives, `cfg`/`serde`/`repr`/custom proc-macro attributes — with no
  per-attribute grammar work, now or ever.
- The one hard gap (crate-level `#![…]`) closes; nothing in the language is left
  unreachable by both the surface and `__rust__`.
- Codegen collapses three near-identical structured-emission sites into one
  verbatim write. Less code, the `Literal::Raw` pattern reused.

**Harder / accepted costs.**
- **Irreversible AST change**: `AttributeArg` is deleted and `Attribute` is
  reshaped from `{name, args}` to `{text}`. This ripples to ~39 references across
  5 files (`ast.rs`, `parser.rs`, `codegen.rs`, `codegen_unit_tests.rs`,
  `parser_coverage_tests.rs`). Tests that assert on structured args are rewritten
  to assert on the verbatim text. This is a one-way door — hence an ADR.
- Crusty no longer *validates* attribute syntax at all; a malformed attribute is
  rustc's error, not Crusty's (consistent with ADR-0001, and the same trade
  `Literal::Raw` already makes). Balanced-bracket capture still requires the
  brackets to balance, so `#[` with no closing `]` is still a Crusty parse error.
- Attribute text is carried byte-for-byte, so Crusty will not normalise spacing
  inside attributes (e.g. `#[ test ]` passes through as written). Acceptable —
  rustc does not care, and `rustfmt` is the formatter of record.

**Neutral.**
- The attribute *position* grammar is unchanged: attributes still attach to the
  same items/statements (`Statement::Attributed`, item-leading `attributes()`)
  they do now — only the *contents* become verbatim.

## Verification

- [ ] `Attribute` in `src/ast.rs` is `{ text: String }`; `AttributeArg` is removed; `grep -rn AttributeArg src/` returns nothing.
- [ ] `#[derive(Debug, Clone)]`, `#[cfg(all(unix, feature = "x"))]`, and `#[serde(rename_all = "camelCase", default)]` all parse and round-trip to byte-identical Rust.
- [ ] A new inner-attribute item parses `#![no_std]` / `#![feature(foo)]` / `#![allow(dead_code)]` and emits them verbatim at crate top level.
- [ ] Directive disambiguation unaffected: `#define`, `#macro`, `#ifdef…#endif`, `#import`, `#export` still parse; `#include`/`#use` still rejected with their hints.
- [ ] An unbalanced `#[` (no closing `]`) is still a Crusty parse error.
- [ ] End-to-end under `rustc -D unused_parens`: a file with a crate-level `#![allow(dead_code)]` plus a `#[derive(Debug)]` struct with a nested-`cfg` field attribute compiles.
- [ ] Full suite green (`cargo test`), clippy clean.
- [ ] `docs/syntax/` documents both attribute forms as verbatim pass-through and the directive-vs-attribute disambiguation.

## Alternatives considered

- **Keep the structured `Attribute` AST, widen `AttributeArg` to a token-tree
  string.** Rejected: it keeps a typed wrapper around data nothing inspects, and
  still forces a second rule for `#![…]`. The verbatim span subsumes both with
  less code — if the args are already an opaque string, the whole `#[…]` span may
  as well be.
- **Add only `#![…]` inner-attribute pass-through, leave `#[…]` structured.**
  Rejected (by the user): it closes the hard gap but leaves `#[…]` unable to
  express nested meta, so real-world attributes like `#[cfg(all(...))]` still
  fail. Doing both now is the coherent decision; splitting it would mean a second
  ADR superseding the attribute shape within the same session.
- **Interpret/validate attributes in Crusty.** Rejected under ADR-0001 — Crusty
  is a syntax layer; attribute semantics are rustc's.
