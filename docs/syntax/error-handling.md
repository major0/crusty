# Error Handling

## Introduction

Crusty uses `?` suffix syntax for error handling, mapping to Rust's `Result` type and `?` operator. Return types use `Type?` to indicate fallible functions, and the `?` operator propagates errors.

## Rationale

Rust already uses `?` for error propagation, so `expr?` in Crusty passes through unchanged. Using `Type?` for return types provides a concise way to express `Result<Type, E>` without Rust-specific syntax. Method names like `.is_err()`, `.unwrap()`, `Ok()`, and `Err()` pass through unchanged — Crusty is a syntax layer, not a semantic transformation.

## Examples

### Fallible Function
```c
int? parse_number(char* str) {
    let num = str.parse()?;
    return Ok(num);
}
```
Translates to:
```rust
pub fn parse_number(str: *const char) -> Result<i32, Box<dyn std::error::Error>> {
    let num = str.parse()?;
    return Ok(num);
}
```

> Note: `char*` lowers to the raw pointer `*const char`, not `&str` — Crusty is
> a syntax layer and does not infer Rust's string-slice type from a C `char*`.
> The explicit `return` is preserved verbatim (Crusty does not rewrite a tail
> `return` into a trailing expression).

### Error Propagation
```c
int? read_config() {
    let content = read_file("config.txt")?;
    let value = parse_number(content)?;
    return Ok(value);
}
```

### Error Handling Methods
```c
let result = parse_number("42");
if (result.is_err()) {
    // handle error
}
let value = result.unwrap();
```

All error handling methods (`.is_err()`, `.is_ok()`, `.unwrap()`, `.unwrap_or()`) pass through to Rust unchanged.

## Formal Grammar

```ebnf
fallible_type = type_expr "?" ;
error_prop    = expr "?" ;
```

## Syntax Transformations

| Crusty | Rust | Type |
|--------|------|------|
| `Type?` | `Result<Type, Box<dyn std::error::Error>>` | Syntax transform |
| `expr?` | `expr?` | Pass through |
| `Ok(val)` | `Ok(val)` | Pass through |
| `Err(val)` | `Err(val)` | Pass through |
| `.is_err()` | `.is_err()` | Pass through |
| `.unwrap()` | `.unwrap()` | Pass through |
