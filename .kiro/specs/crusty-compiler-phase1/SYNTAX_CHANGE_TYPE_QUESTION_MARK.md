# Syntax Change: Type! → Type? and expr! → expr?

## Summary

Changed error handling syntax from `Type!` and `expr!` to `Type?` and `expr?` for better simplicity and alignment with Rust.

**Date**: 2026-01-29
**Rationale**: The `?` suffix is simpler to implement and more intuitive than `!`. Since Rust already uses `?` for error propagation, using `expr?` means it passes through unchanged to Rust.

## Syntax Changes

### Before (Type! and expr!)
```c
int! parse_number(char* str) {
    let num = str.parse()!;  // expr! → expr?
    return Ok(num);
}
```

### After (Type? and expr?)
```c
int? parse_number(char* str) {
    let num = str.parse()?;  // expr? → expr? (pass through)
    return Ok(num);
}
```

## Transformations

| Crusty Syntax | Rust Output | Transformation Type |
|---------------|-------------|---------------------|
| `Type?` | `Result<Type, Box<dyn std::error::Error>>` | Syntax transformation |
| `expr?` | `expr?` | Pass through (no transformation) |
| `.is_err()` | `.is_err()` | Pass through |
| `.is_ok()` | `.is_ok()` | Pass through |
| `.unwrap()` | `.unwrap()` | Pass through |
| `Ok()` | `Ok()` | Pass through |
| `Err()` | `Err()` | Pass through |

## Implementation Changes Required

### 1. Lexer Changes
**File**: `src/lexer.rs`

- Update token recognition to use `?` instead of `!` for:
  - Type suffix: `int?` instead of `int!`
  - Error propagation operator: `expr?` instead of `expr!`
- Note: The `?` character may already be tokenized; verify it doesn't conflict with ternary operator

### 2. Parser Changes
**File**: `src/parser.rs`

- Update return type parsing to recognize `?` suffix instead of `!`
- Update expression parsing to recognize `expr?` instead of `expr!`
- Ensure `?` in ternary operator context is still parsed correctly

### 3. AST Changes
**File**: `src/ast.rs`

- No changes needed if AST already represents fallible types and error propagation generically
- If AST nodes reference specific syntax, update documentation/comments

### 4. Code Generation Changes
**File**: `src/codegen.rs`

- Update fallible return type generation: `Type?` → `Result<Type, Box<dyn std::error::Error>>`
- Update error propagation: `expr?` → `expr?` (pass through unchanged)
- Verify that `?` operator is emitted correctly in Rust output

### 5. Semantic Analysis Changes
**File**: `src/semantic.rs`

- Update error messages to reference `Type?` instead of `Type!`
- Update validation logic if it checks for specific syntax

### 6. Test Updates
**Files**: `src/*_tests.rs`, `src/*_properties.rs`

- Update all test cases using `Type!` to use `Type?`
- Update all test cases using `expr!` to use `expr?`
- Update expected output in tests
- Update property-based test generators

### 7. Example Files
**Files**: `example/src/*.crst`

- Update any example files that demonstrate error handling
- Ensure examples use `Type?` and `expr?` syntax

## Documentation Updates

All documentation has been updated:

- ✅ `README.md` - Updated all examples and explanations
- ✅ `SYNTAX_PHILOSOPHY.md` - Updated syntax transformation rules
- ✅ `requirements.md` - Updated Requirement 49 with removed semantic transformations
- ✅ `design.md` - Updated Property 20 and design principles
- ✅ `tasks.md` - Updated task 16.7 and reverse transpilation tasks
- ✅ `DOCUMENTATION_REVIEW.md` - Updated all references

## Testing Strategy

1. **Unit Tests**: Update all unit tests to use new syntax
2. **Property Tests**: Update property-based tests to generate `Type?` and `expr?`
3. **Integration Tests**: Verify end-to-end transpilation works correctly
4. **Regression Tests**: Ensure existing functionality still works

## Migration Path

For existing Crusty code (if any):

1. Replace all `Type!` with `Type?` in return types
2. Replace all `expr!` with `expr?` in expressions
3. No other changes needed (method names already pass through)

## Benefits

1. **Simpler Implementation**: `expr?` passes through to Rust unchanged
2. **More Intuitive**: `?` conveys "maybe" or "optional" semantics
3. **Consistent**: Both use the same `?` symbol
4. **Less Novel**: Closer to Rust's existing syntax

## Risks

1. **Ternary Operator Conflict**: Need to ensure `?` in `condition ? true_val : false_val` doesn't conflict
2. **Existing Code**: Any existing Crusty code needs to be updated (likely minimal impact)

## Verification Checklist

- [ ] Lexer recognizes `Type?` syntax
- [ ] Lexer recognizes `expr?` syntax
- [ ] Parser correctly parses `Type?` in return types
- [ ] Parser correctly parses `expr?` in expressions
- [ ] Parser still handles ternary operator `? :` correctly
- [ ] Code generator emits `Result<Type, E>` for `Type?`
- [ ] Code generator passes through `expr?` unchanged
- [ ] All unit tests pass
- [ ] All property-based tests pass
- [ ] All integration tests pass
- [ ] Example programs compile and run correctly
- [ ] Documentation is consistent

## Related Files

- `src/lexer.rs` - Token recognition
- `src/parser.rs` - Syntax parsing
- `src/ast.rs` - AST representation
- `src/codegen.rs` - Code generation
- `src/semantic.rs` - Semantic analysis
- `src/*_tests.rs` - Unit tests
- `src/*_properties.rs` - Property-based tests
- `example/src/*.crst` - Example programs

## Notes

- This change aligns Crusty more closely with Rust's syntax
- The `?` operator is already well-understood by Rust developers
- Implementation should be straightforward since `expr?` passes through unchanged
- Main work is updating the lexer and parser to recognize `?` instead of `!`
