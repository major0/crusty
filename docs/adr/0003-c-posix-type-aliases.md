# ADR-0003: C/POSIX type aliases

## Status
accepted

## Context
Crusty aims to accept basic C programs with little or no modification, excluding
the memory-unsafe features it deliberately drops (unions, unchecked casts, etc.).
A gap today: C code names its integer types with the `<stdint.h>` and POSIX
spellings (`uint8_t`, `size_t`, `ssize_t`, …) and the idiomatic `byte`. Crusty
has the underlying types (`uint8` → `u8`, etc.) but not these standard names, so
a C program using them fails to transpile for a purely cosmetic reason.

These names are *aliases* for types that already exist; they carry no new
semantics. They should therefore cost as little as possible to support, and must
not grow the primitive type surface (keywords, `PrimitiveType` variants, codegen
arms) for every name.

## Decision
We will recognise a fixed table of single-token C/POSIX type names as aliases
that lower to their Rust equivalents, resolved as `Type::Ident` names at code
generation time — NOT as new primitive keywords.

The alias table (Crusty name → emitted Rust type):

| Crusty (C/POSIX) | Rust  |
|------------------|-------|
| `int8_t`         | `i8`  |
| `int16_t`        | `i16` |
| `int32_t`        | `i32` |
| `int64_t`        | `i64` |
| `uint8_t`        | `u8`  |
| `uint16_t`       | `u16` |
| `uint32_t`       | `u32` |
| `uint64_t`       | `u64` |
| `size_t`         | `usize` |
| `uintptr_t`      | `usize` |
| `ssize_t`        | `isize` |
| `ptrdiff_t`      | `isize` |
| `intptr_t`       | `isize` |
| `byte`           | `u8`  |

A C alias is a plain identifier type (`Type::Ident`), so no keyword, AST variant,
or parser rule is added. Codegen's `Type::Ident` arm consults the table: a hit
emits the mapped Rust type; a miss emits the name verbatim (today's behaviour).

Precedence: a user `typedef` of the same name wins. User typedefs resolve through
the semantic type environment before codegen, so a program that defines its own
`size_t` is unaffected by the built-in alias.

Scope: ONLY single-token names. Multi-word C primitives (`unsigned int`,
`long long`, `unsigned char`) are a separate, larger parser decision and are NOT
covered here.

## Consequences
- Easier: idiomatic C integer-typed code transpiles unmodified; the table is one
  data structure, trivially extended with a new row.
- Harder / accepted: these names are not Crusty's canonical spelling (that remains
  `int8`/`uint8`/…), so there are now two spellings for the same type — accepted,
  because the alias exists precisely for C-source compatibility. The table is a
  second place (beside the primitive map) that names a type-to-Rust mapping;
  accepted as the cost of not growing the primitive surface.
- A user typedef shadowing a C alias silently changes what the name emits; this
  matches how a C program redefining a standard type behaves, so it is intended.

## Verification
- [ ] `uint8_t x = 0;` transpiles to `let x: u8 = 0;` (and the other 13 rows map
      per the table), confirmed by a transpile that `rustc` accepts.
- [ ] `byte b = 0;` transpiles with `b: u8`.
- [ ] A function `size_t len(uint8_t* p)` lowers to `len(p: *const u8) -> usize`.
- [ ] A user `typedef uint32 size_t;` makes `size_t` resolve to the user's target,
      not the built-in `usize` (user typedef wins).
- [ ] An unknown identifier type (e.g. `Widget`) still emits verbatim — the table
      is a lookup, not a rename of every ident.
- [ ] `docs/syntax/types.md` documents the alias table.
