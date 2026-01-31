# Postfix Operator Removal Summary

**Date**: 2026-01-31  
**Commit**: 7be6fb4

## Overview

Removed support for postfix increment and decrement operators (`i++`, `i--`) from Crusty's syntax specification. Only prefix operators (`++i`, `--i`) are now supported.

## Rationale

Postfix operators add implementation complexity without significant benefit for Crusty's goals:
- Require temporary variable generation in Rust code
- Add semantic complexity (return old value vs new value)
- Prefix operators are sufficient for all use cases
- Simplifies parser and code generator

## Changes Made

### Requirements Document

**File**: `.kiro/specs/crusty-compiler-phase1/requirements.md`

**Requirement 29: Support Operators**

Removed criteria:
- ❌ 7: Parser SHALL support C postfix increment/decrement (i++, i--)
- ❌ 9: Semantic analyzer SHALL verify postfix returns original value
- ❌ 11: Code generator SHALL translate postfix increment to `({ let tmp = i; i += 1; tmp })`
- ❌ 13: Code generator SHALL translate postfix decrement to `({ let tmp = i; i -= 1; tmp })`

Kept criteria (renumbered):
- ✅ 6: Parser SHALL support C prefix increment/decrement (++i, --i)
- ✅ 7: Semantic analyzer SHALL verify prefix increments before returning
- ✅ 8: Code generator SHALL translate prefix increment to `(i += 1; i)`
- ✅ 9: Code generator SHALL translate prefix decrement to `(i -= 1; i)`

Total criteria reduced from 21 to 17.

### Design Document

**File**: `.kiro/specs/crusty-compiler-phase1/design.md`

**Property 11**: Updated to validate only prefix operators
- Before: "prefix increment (++i) ... postfix increment (i++)"
- After: "prefix increment (++i) or prefix decrement (--i)"
- Validates: Requirements 29.8, 29.9 (was 29.10, 29.11)

**Property 18**: Updated for loop examples
- Before: `for(i=start; i<end; i++)`
- After: `for(i=start; i<end; ++i)`

### Property Test Catalog

**File**: `.kiro/specs/crusty-compiler-phase1/PROPERTY_TEST_CATALOG.md`

Updated Property 11:
- Statement: Now covers only prefix operators
- Validates: Requirements 29.8-29.9 (was 29.10-29.11)
- Status: ✅ Implemented

Updated Property 18:
- Example pattern: `for(i=start; i<end; ++i)` (was `i++`)

### Tasks Document

**File**: `.kiro/specs/crusty-compiler-phase1/tasks.md`

Task 16.8: Updated Property 11 validation
- Validates: Requirements 29.8, 29.9 (was 29.10, 29.11)

### Tasks Review

**File**: `.kiro/specs/crusty-compiler-phase1/TASKS_REVIEW.md`

Updated Property 11 mapping:
- Requirements: 29.8, 29.9 (was 29.10, 29.11)

### Documentation Updates

**Files Updated**:
- `README.md`: For loop examples use `++i`
- `SYNTAX_REFERENCE.md`: For loop examples use `++i`
- `.kiro/specs/crusty-compiler-phase1/SYNTAX_PHILOSOPHY.md`: For loop examples use `++i`

All examples changed from:
```c
for (int i = 0; i < 100; i++) { ... }
```

To:
```c
for (int i = 0; i < 100; ++i) { ... }
```

## Impact Assessment

### ✅ No Code Changes Required

The parser and code generator already support prefix operators. No implementation changes needed.

### ✅ All Tests Pass

- 428 tests passing (100%)
- 3 tests ignored (unrelated)
- No test failures

### ✅ Documentation Consistency

All documentation now consistently uses prefix operators in examples.

## Supported Syntax

### ✅ Prefix Operators (Supported)

```c
int x = 5;
int y = ++x;  // x becomes 6, y is 6
int z = --x;  // x becomes 5, z is 5
```

### ❌ Postfix Operators (NOT Supported)

```c
int x = 5;
int y = x++;  // ❌ NOT SUPPORTED
int z = x--;  // ❌ NOT SUPPORTED
```

### ✅ For Loops (Use Prefix)

```c
// Correct
for (int i = 0; i < 10; ++i) {
    // ...
}

// Incorrect
for (int i = 0; i < 10; i++) {  // ❌ NOT SUPPORTED
    // ...
}
```

## Migration Guide

If you have existing Crusty code using postfix operators:

**Before**:
```c
for (int i = 0; i < n; i++) {
    arr[i] = i;
}

int x = 5;
int y = x++;
```

**After**:
```c
for (int i = 0; i < n; ++i) {
    arr[i] = i;
}

int x = 5;
int y = x;
++x;  // Separate increment
```

## Benefits

1. **Simpler Implementation**: No need for temporary variable generation
2. **Clearer Semantics**: Prefix operators have straightforward meaning
3. **Consistent Style**: Encourages consistent use of prefix operators
4. **Reduced Complexity**: Fewer edge cases in parser and code generator

## Related Requirements

- Requirement 29: Support Operators (updated)
- Property 11: Increment/decrement operators (updated)
- Property 18: For loops (updated)

---

**Status**: ✅ Complete  
**Tests**: ✅ All Passing (428/428)  
**Documentation**: ✅ Updated
