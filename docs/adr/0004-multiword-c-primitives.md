# ADR-0004: Multi-word C primitive types

## Status
accepted

## Context
ADR-0003 added single-token C/POSIX type aliases (`uint8_t`, `size_t`, …). C
source also spells its built-in integers with MULTIPLE keywords — `unsigned
int`, `long long`, `signed char`, `unsigned long int` — which are not single
identifiers and so cannot be a `Type::Ident` alias. To accept basic C programs
with little modification, the type grammar must recognise these multi-word
spellings and lower them to a Rust fixed-width integer.

Two forces make this a decision, not a mechanical add:

1. **Width is platform-defined in C.** `long` is 32-bit on LLP64 (Windows) and
   64-bit on LP64 (Linux/macOS); `int` is almost always 32-bit; `char`
   signedness is implementation-defined. We must pick one model.
2. **The tokens combine.** `unsigned`, `signed`, `short`, `long`, `int`, `char`
   appear in ordered combinations (`unsigned long int`, `long long`), with
   optional and reorder-tolerant parts, so the grammar must accept a small
   language of type-specifier sequences, not a fixed list of strings.

## Decision
We will parse the common C integer type-specifier combinations as multi-word
primitive types and lower them with the **LP64** width model (the dominant
target for Rust: Linux and macOS). Each lowers to an existing Crusty primitive:

| C spelling (and equivalents)                         | Rust  |
|------------------------------------------------------|-------|
| `signed char`                                        | `i8`  |
| `unsigned char`                                      | `u8`  |
| `short`, `short int`, `signed short`, `signed short int` | `i16` |
| `unsigned short`, `unsigned short int`               | `u16` |
| `signed`, `signed int`                               | `i32` |
| `unsigned`, `unsigned int`                           | `u32` |
| `long`, `long int`, `signed long`, `signed long int` | `i64` |
| `unsigned long`, `unsigned long int`                 | `u64` |
| `long long`, `long long int`, `signed long long [int]` | `i64` |
| `unsigned long long [int]`                           | `u64` |

`int` and `char` on their own keep their existing Crusty meaning (`i32` and Rust
`char`); only combinations and the width keywords (`short`/`long`/`signed`/
`unsigned`) are newly recognised. A bare `signed`/`unsigned` means `signed int`/
`unsigned int`.

Grammar: a `c_multiword_type` rule is tried in the type-primary position BEFORE
`primitive_type` and `ident_type`, so `unsigned int` is consumed as a unit
rather than `unsigned` being read as an identifier. It returns a
`Type::Primitive`, so nothing downstream needs to know it came from a multi-word
spelling.

## Consequences
- Easier: idiomatic C integer declarations (`unsigned int`, `long`, `long long`)
  transpile unmodified.
- Accepted — platform width: `long` is 64-bit here (LP64). Code written for
  LLP64 (Windows, `long` = 32-bit) that relies on `long` being 32-bit will get a
  wider type. This is the correct default for the Rust-target platforms and is
  documented; a program needing an exact width should use the fixed-width names
  (`int32`/`int64`) or the `_t` aliases.
- Accepted — `char` signedness: bare `char` stays Rust `char` (a Unicode scalar),
  NOT an 8-bit integer. C code using `char` as a byte should use `signed char` /
  `unsigned char` (→ `i8`/`u8`) or `byte`. This is a known divergence.
- `long double` is NOT supported (Rust has no 80/128-bit float); it is out of
  scope and remains unrecognised.

## Verification
- [ ] `unsigned int x;` → `let x: u32;`, `long y;` → `let y: i64;`,
      `long long z;` → `let z: i64;`, `unsigned long long w;` → `let w: u64;`,
      confirmed by a transpile `rustc` accepts.
- [ ] `signed char` → `i8`, `unsigned char` → `u8`, `unsigned short` → `u16`.
- [ ] A function `unsigned long sum(unsigned int a, long b)` lowers to
      `sum(a: u32, b: i64) -> u64`.
- [ ] Bare `int` is still `i32` and bare `char` is still Rust `char` (the
      multi-word rule does not hijack them).
- [ ] An identifier that merely starts with a width word (e.g. `longitude`) is
      still parsed as an identifier, not `long` + `itude`.
- [ ] `docs/syntax/types.md` documents the multi-word spellings and the LP64 /
      `char` caveats.
