# Requirements: Typedef Type Alias Support

## Overview
Enhance Crusty's typedef implementation to properly support type aliases beyond just `typedef struct` patterns. The typedef feature should work as a general type alias mechanism in Rust, allowing users to create aliases for any type.

## Background
Currently, Crusty has a typedef implementation that:
- Parses typedef declarations (`typedef <type> <name>;`)
- Generates Rust `type` aliases in code generation
- Registers type aliases in the semantic analyzer's type environment

However, the semantic analyzer doesn't resolve type aliases during type checking, causing type mismatch errors when using typedef'd types.

## User Stories

### 1. Simple Type Aliases
**As a** Crusty developer  
**I want to** create simple type aliases like `typedef int MyInt;`  
**So that** I can use `MyInt` interchangeably with `int` throughout my code

**Acceptance Criteria:**
- 1.1: Parser accepts `typedef int MyInt;` syntax
- 1.2: Code generator produces `pub type MyInt = i32;`
- 1.3: Semantic analyzer treats `MyInt` and `int` as compatible types
- 1.4: Variables declared with `MyInt` type can be assigned `int` values
- 1.5: Functions returning `int` can return `MyInt` values and vice versa

### 2. Pointer Type Aliases
**As a** Crusty developer  
**I want to** create pointer type aliases like `typedef *int IntPtr;`  
**So that** I can use cleaner type names for pointer types

**Acceptance Criteria:**
- 2.1: Parser accepts `typedef *int IntPtr;` syntax (prefix pointer)
- 2.2: Code generator produces `pub type IntPtr = *mut i32;`
- 2.3: Semantic analyzer treats `IntPtr` and `*int` as compatible types
- 2.4: Variables declared with `IntPtr` can be assigned `*int` values

### 3. Custom Type Aliases
**As a** Crusty developer  
**I want to** create aliases for custom types like `typedef Point PointAlias;`  
**So that** I can provide alternative names for struct types

**Acceptance Criteria:**
- 3.1: Parser accepts `typedef CustomType AliasName;` syntax
- 3.2: Code generator produces `pub type AliasName = CustomType;`
- 3.3: Semantic analyzer treats `AliasName` and `CustomType` as compatible types
- 3.4: Struct initialization works with aliased type names

### 4. Reference Type Aliases
**As a** Crusty developer  
**I want to** create reference type aliases like `typedef &int IntRef;`  
**So that** I can simplify reference type declarations

**Acceptance Criteria:**
- 4.1: Parser accepts `typedef &int IntRef;` syntax
- 4.2: Code generator produces `pub type IntRef = &i32;`
- 4.3: Semantic analyzer treats `IntRef` and `&int` as compatible types

### 5. Generic Type Aliases
**As a** Crusty developer  
**I want to** create aliases for generic types like `typedef Vec[int] IntVec;`  
**So that** I can simplify complex generic type declarations

**Acceptance Criteria:**
- 5.1: Parser accepts `typedef Vec[int] IntVec;` syntax
- 5.2: Code generator produces `pub type IntVec = Vec<i32>;`
- 5.3: Semantic analyzer treats `IntVec` and `Vec[int]` as compatible types

## Non-Functional Requirements

### Performance
- Type alias resolution should not significantly impact compilation time
- Alias resolution should handle circular references gracefully (detect and report error)

### Compatibility
- Existing typedef struct patterns must continue to work
- All existing tests must pass
- Generated Rust code must compile without errors

### Error Handling
- Clear error messages when typedef references undefined types
- Detect and report circular type alias definitions
- Provide helpful suggestions when type mismatches occur with aliases

## Out of Scope
- Postfix pointer syntax (`int*` instead of `*int`) - this would require lexer/parser changes
- Type alias generics (e.g., `typedef T Vec[T];`) - Rust doesn't support this without trait bounds
- Conditional type aliases or type-level programming features

## Success Metrics
- All existing typedef tests pass
- New comprehensive typedef tests pass
- Type aliases work correctly in variable declarations, function parameters, and return types
- Generated Rust code compiles and runs correctly
- No performance regression in compilation time
