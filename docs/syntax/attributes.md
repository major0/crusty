# Attributes

Crusty carries Rust attributes through **verbatim** (ADR-0020). It does not
interpret their contents — an attribute's meaning is rustc's, consistent with
Crusty's role as a syntax layer (ADR-0001). This means *every* Rust attribute
works, including nested meta that no structured parser would need to model.

## Outer attributes — `#[ … ]`

An outer attribute attaches to the item or statement that follows it. The whole
`#[ … ]` span is captured as-is and emitted unchanged.

```c
#[derive(Debug, Clone)]
struct Config {
    int retries;
    bool verbose;
}

#[cfg(all(unix, target_pointer_width = "64"))]
void only_on_64bit_unix() {
    println!("64-bit unix only");
}

#[inline]
int add(int a, int b) {
    return a + b;
}
```

Transpiles to:

```rust
#[derive(Debug, Clone)]
pub struct Config {
    pub retries: i32,
    pub verbose: bool,
}

#[cfg(all(unix, target_pointer_width = "64"))]
pub fn only_on_64bit_unix() {
    println!("64-bit unix only");
}

#[inline]
pub fn add(a: i32, b: i32) -> i32 {
    return a + b;
}
```

Attributes also attach to statements inside a function body:

```c
#[allow(unused)]
let x = 1;
```

Because the span is carried verbatim, arbitrarily nested meta works with no
special handling — `#[cfg(all(unix, feature = "x"))]`,
`#[serde(rename_all = "camelCase", default)]`, multiple derives, and custom
proc-macro attributes all pass straight through. Inner `[ … ]` is handled by
bracket-balanced capture, so token-tree arguments do not terminate the attribute
early.

## Inner attributes — `#![ … ]`

An inner attribute applies to the enclosing crate. Place crate-level inner
attributes at the top of the file; they are emitted verbatim ahead of other
items, which is where rustc requires them.

```c
#![no_std]
#![feature(portable_simd)]
#![allow(dead_code)]

void main() {
    // …
}
```

Transpiles to:

```rust
#![no_std]
#![feature(portable_simd)]
#![allow(dead_code)]

pub fn main() {
    // …
}
```

Inner attributes are the one construct the `__rust__` escape hatch cannot
express — they must be the crate's first tokens and cannot be nested inside a
`__rust__ { … }` item block — so this surface form is how you reach them.

## Attributes vs `#` directives

Crusty owns a family of `#` preprocessor directives (`#define`, `#macro`,
`#ifdef`/`#ifndef`/`#endif`, `#import`, `#export`). Attributes never collide with
them: the character right after `#` decides.

| After `#` | Meaning |
|---|---|
| `[` | outer attribute — verbatim pass-through |
| `![` | inner attribute — verbatim pass-through |
| a directive keyword | a Crusty preprocessor directive |

No directive keyword begins with `[` or `!`, so the two surfaces are
unambiguous.

## Notes

- Crusty does not validate attribute contents; a malformed attribute is rustc's
  error, not Crusty's. The one exception is structural: the brackets must
  balance, so a `#[` with no closing `]` is a Crusty parse error.
- Attribute text is carried byte-for-byte, so inner spacing is preserved as
  written (`#[ test ]` passes through unchanged). `rustfmt` is the formatter of
  record.
