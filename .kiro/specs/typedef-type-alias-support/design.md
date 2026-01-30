# Design: Typedef Type Alias Support

## Overview
This design enhances Crusty's typedef implementation to properly resolve type aliases during semantic analysis, enabling typedef to work as a general type alias mechanism.

## Current Implementation Analysis

### Parser (`src/parser.rs`)
- **Function**: `parse_typedef()` (lines 972-1005)
- **Status**: ✅ Working correctly
- **Behavior**: Parses `typedef <type> <name>;` syntax and creates `Typedef` AST node
- **Supports**: All type expressions including primitives, pointers, references, custom types, generics

### Code Generator (`src/codegen.rs`)
- **Function**: `generate_typedef()` (lines 394-450)
- **Status**: ✅ Working correctly
- **Behavior**: Generates Rust `type` aliases with proper visibility
- **Output**: `pub type Name = TargetType;`

### Semantic Analyzer (`src/semantic.rs`)
- **Function**: `analyze_typedef()` (lines 586-617)
- **Status**: ⚠️ Partially working
- **Behavior**: Registers typedef in type environment and symbol table
- **Issue**: Type aliases are registered but never resolved during type checking

### Type Compatibility (`src/semantic.rs`)
- **Function**: `is_compatible()` (lines 244-350)
- **Status**: ❌ Missing alias resolution
- **Issue**: Compares `Type::Ident("MyInt")` with `Type::Primitive(I32)` without resolving aliases

## Root Cause
The `is_compatible()` function doesn't resolve type aliases before comparing types. When checking if `MyInt` (a typedef for `int`) is compatible with `int`, it compares the types directly without looking up the alias definition.

## Solution Design

### 1. Add Type Resolution Function
Add a new function to `TypeEnvironment` that resolves type aliases:

```rust
impl TypeEnvironment {
    /// Resolve a type by following type aliases
    /// Returns the resolved type, or the original type if it's not an alias
    pub fn resolve_type(&self, ty: &Type) -> Type {
        match ty {
            Type::Ident(ident) => {
                // Look up the type in the type environment
                if let Some(type_info) = self.types.get(&ident.name) {
                    match &type_info.kind {
                        TypeKind::Alias { target } => {
                            // Recursively resolve the target type
                            // This handles chains like: typedef A B; typedef B C;
                            self.resolve_type(target)
                        }
                        _ => ty.clone(),
                    }
                } else {
                    ty.clone()
                }
            }
            // For complex types, recursively resolve inner types
            Type::Pointer { ty: inner, mutable } => Type::Pointer {
                ty: Box::new(self.resolve_type(inner)),
                mutable: *mutable,
            },
            Type::Reference { ty: inner, mutable } => Type::Reference {
                ty: Box::new(self.resolve_type(inner)),
                mutable: *mutable,
            },
            Type::Array { ty: inner, size } => Type::Array {
                ty: Box::new(self.resolve_type(inner)),
                size: *size,
            },
            Type::Slice { ty: inner } => Type::Slice {
                ty: Box::new(self.resolve_type(inner)),
            },
            Type::Generic { base, args } => Type::Generic {
                base: Box::new(self.resolve_type(base)),
                args: args.iter().map(|t| self.resolve_type(t)).collect(),
            },
            Type::Tuple { types } => Type::Tuple {
                types: types.iter().map(|t| self.resolve_type(t)).collect(),
            },
            Type::Function { params, return_type } => Type::Function {
                params: params.iter().map(|t| self.resolve_type(t)).collect(),
                return_type: Box::new(self.resolve_type(return_type)),
            },
            Type::Fallible { ty: inner } => Type::Fallible {
                ty: Box::new(self.resolve_type(inner)),
            },
            // Primitives and Auto don't need resolution
            _ => ty.clone(),
        }
    }
}
```

### 2. Update is_compatible Function
Modify `is_compatible()` to resolve types before comparison:

```rust
pub fn is_compatible(&self, t1: &Type, t2: &Type) -> bool {
    // Resolve type aliases first
    let resolved_t1 = self.resolve_type(t1);
    let resolved_t2 = self.resolve_type(t2);
    
    // Then perform compatibility check on resolved types
    match (&resolved_t1, &resolved_t2) {
        // ... existing compatibility logic ...
    }
}
```

### 3. Add Circular Reference Detection
Add a helper function to detect circular type alias definitions:

```rust
impl TypeEnvironment {
    /// Check if a type alias chain contains a circular reference
    fn has_circular_reference(&self, ty: &Type, visited: &mut HashSet<String>) -> bool {
        match ty {
            Type::Ident(ident) => {
                if visited.contains(&ident.name) {
                    return true; // Circular reference detected
                }
                
                if let Some(type_info) = self.types.get(&ident.name) {
                    if let TypeKind::Alias { target } = &type_info.kind {
                        visited.insert(ident.name.clone());
                        let result = self.has_circular_reference(target, visited);
                        visited.remove(&ident.name);
                        return result;
                    }
                }
                false
            }
            // Check inner types for complex types
            Type::Pointer { ty, .. } | 
            Type::Reference { ty, .. } |
            Type::Array { ty, .. } |
            Type::Slice { ty } |
            Type::Fallible { ty } => self.has_circular_reference(ty, visited),
            
            Type::Generic { base, args } => {
                self.has_circular_reference(base, visited) ||
                args.iter().any(|t| self.has_circular_reference(t, visited))
            }
            _ => false,
        }
    }
}
```

### 4. Update analyze_typedef
Add circular reference check when analyzing typedef:

```rust
fn analyze_typedef(&mut self, typedef: &crate::ast::Typedef) {
    // Check for circular references
    let mut visited = HashSet::new();
    if self.type_env.has_circular_reference(&typedef.target, &mut visited) {
        self.errors.push(SemanticError::new(
            Span::new(
                crate::error::Position::new(0, 0),
                crate::error::Position::new(0, 0),
            ),
            SemanticErrorKind::TypeMismatch,
            format!("circular type alias definition for '{}'", typedef.name.name),
        ));
        return;
    }
    
    // ... existing registration logic ...
}
```

## Implementation Strategy

### Phase 1: Core Type Resolution
1. Add `resolve_type()` function to `TypeEnvironment`
2. Update `is_compatible()` to use `resolve_type()`
3. Add unit tests for type resolution

### Phase 2: Circular Reference Detection
1. Add `has_circular_reference()` function
2. Update `analyze_typedef()` to check for circular references
3. Add tests for circular reference detection

### Phase 3: Comprehensive Testing
1. Add integration tests for all typedef scenarios
2. Test typedef in variable declarations
3. Test typedef in function parameters and return types
4. Test typedef with complex types (pointers, references, generics)
5. Verify generated Rust code compiles

## Testing Strategy

### Unit Tests
- Test `resolve_type()` with simple aliases
- Test `resolve_type()` with chained aliases
- Test `resolve_type()` with complex types
- Test circular reference detection
- Test `is_compatible()` with resolved types

### Integration Tests
- Test typedef in variable declarations
- Test typedef in function signatures
- Test typedef with struct types
- Test typedef with generic types
- Test typedef with pointer and reference types

### Property-Based Tests
- Property: Type alias resolution is idempotent (resolving twice gives same result)
- Property: Circular references are always detected
- Property: Compatible types remain compatible after resolution

## Correctness Properties

**Validates: Requirements 1.3, 2.3, 3.3, 4.3, 5.3**

### Property 1: Type Alias Transitivity
```
For all type aliases A → B and B → C:
  resolve_type(A) == resolve_type(C)
```

**Validates: Requirements 1.3**

### Property 2: Compatibility Symmetry with Aliases
```
For all types T1, T2:
  is_compatible(T1, T2) == is_compatible(resolve_type(T1), resolve_type(T2))
```

**Validates: Requirements 1.3, 2.3, 3.3, 4.3, 5.3**

### Property 3: Circular Reference Detection
```
For all type alias chains with a cycle:
  has_circular_reference(T) == true
```

**Validates: Non-functional requirement for error handling**

### Property 4: Resolution Idempotence
```
For all types T:
  resolve_type(resolve_type(T)) == resolve_type(T)
```

**Validates: Requirements 1.3, 2.3, 3.3, 4.3, 5.3**

## Edge Cases

### 1. Undefined Type Aliases
- **Scenario**: `typedef UndefinedType MyType;`
- **Handling**: Parser accepts, semantic analyzer reports error during type checking
- **Error**: "undefined type 'UndefinedType'"

### 2. Circular Type Aliases
- **Scenario**: `typedef A B; typedef B A;`
- **Handling**: Detected during typedef analysis
- **Error**: "circular type alias definition for 'A'"

### 3. Self-Referential Aliases
- **Scenario**: `typedef MyType MyType;`
- **Handling**: Detected as circular reference
- **Error**: "circular type alias definition for 'MyType'"

### 4. Chained Aliases
- **Scenario**: `typedef int A; typedef A B; typedef B C;`
- **Handling**: Resolved recursively, all resolve to `int`
- **Result**: All three types are compatible with `int`

### 5. Complex Type Aliases
- **Scenario**: `typedef Vec[*int] IntPtrVec;`
- **Handling**: Resolved recursively, inner types resolved
- **Result**: Compatible with `Vec[*int]`

## Performance Considerations

### Type Resolution Caching
- Consider caching resolved types to avoid repeated lookups
- Use a `HashMap<String, Type>` to store resolved aliases
- Clear cache when type environment changes

### Circular Reference Detection
- Use visited set to avoid infinite loops
- Early termination when cycle detected
- O(n) complexity where n is chain length

## Backward Compatibility
- All existing typedef tests must pass
- Existing typedef struct patterns continue to work
- No changes to parser or code generator
- Only semantic analysis enhanced

## Success Criteria
- ✅ All existing tests pass
- ✅ New typedef tests pass
- ✅ Type aliases work in all contexts (variables, functions, parameters)
- ✅ Circular references detected and reported
- ✅ Generated Rust code compiles
- ✅ No performance regression
