# Implementation Guide: Task 17.3 - Nested Function Type Checking

## Overview

This guide provides a detailed implementation plan for adding type checking support for nested functions in the Crusty compiler. Nested functions should be treated as first-class values that can be assigned to variables, passed as parameters, and returned from functions.

## Current State

**Already Implemented:**
- ✅ Task 17.1: Nested function parsing (`Statement::NestedFunction`)
- ✅ Task 17.2: Capture analysis (immutable/mutable detection)
- ✅ Partial Task 17.7: Unit tests for parsing and capture analysis

**Not Yet Implemented:**
- ❌ Type checking for nested functions as values
- ❌ Function pointer type support
- ❌ Type compatibility checking for nested functions

## Requirements (from tasks.md)

Task 17.3 must implement:
1. Verify nested functions can be assigned to variables
2. Verify nested functions can be passed as function parameters
3. Verify nested functions can be returned from functions
4. Support function pointer types for parameters accepting nested functions
5. Verify type compatibility when passing nested functions as arguments
6. Verify multiple nested functions can capture same variables

## Architecture Overview

### Key Components to Modify

1. **`src/semantic.rs`** - SemanticAnalyzer
   - Add nested function type inference
   - Add function type compatibility checking
   - Track nested function types in symbol table

2. **`src/ast.rs`** - Type system (already has `Type::Function`)
   - Verify `Type::Function` supports closure semantics
   - May need to add closure trait information

3. **Tests** - `src/nested_function_tests.rs`
   - Add type checking tests

## Implementation Plan

### Step 1: Understand Current Type System

The AST already has `Type::Function`:

```rust
pub enum Type {
    // ... other variants ...
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    // ... other variants ...
}
```

This is sufficient for representing nested function types.

### Step 2: Modify Semantic Analyzer

#### 2.1: Register Nested Functions as Callable Values

Currently in `src/semantic.rs`, nested functions are registered in the symbol table (around line 1281-1306). We need to ensure they're registered with the correct function type.

**Current code** (approximate):
```rust
Statement::NestedFunction {
    name,
    params,
    return_type,
    body,
} => {
    // Register the nested function in the current scope so it can be called
    let func_type = if let Some(ref ret_type) = return_type {
        Type::Function {
            params: params.iter().map(|p| p.ty.clone()).collect(),
            return_type: Box::new(ret_type.clone()),
        }
    } else {
        Type::Function {
            params: params.iter().map(|p| p.ty.clone()).collect(),
            return_type: Box::new(Type::Primitive(PrimitiveType::Void)),
        }
    };

    self.symbol_table.insert(
        name.name.clone(),
        Symbol::new(
            name.name.clone(),
            func_type,
            SymbolKind::Function,
            false, // nested functions are immutable
        ),
    );
    
    // ... rest of capture analysis ...
}
```

**Enhancement needed**: This code already registers the function type correctly. Verify it's working.

#### 2.2: Add Type Checking for Nested Function Assignment

When a nested function is assigned to a variable, we need to verify type compatibility.

**Location**: In `analyze_statement()` for `Statement::Let` and `Statement::Var`

**Add this logic**:

```rust
Statement::Let { name, ty, init, mutable } => {
    // ... existing code ...
    
    if let Some(init_expr) = init {
        let init_type = self.analyze_expression(init_expr)?;
        
        // If explicit type is provided, check compatibility
        if let Some(expected_type) = ty {
            if !self.type_env.is_compatible(&init_type, expected_type) {
                // Special case: check if init_type is a nested function
                if let Type::Function { .. } = init_type {
                    if let Type::Function { .. } = expected_type {
                        // Check function signature compatibility
                        if !self.check_function_type_compatibility(&init_type, expected_type) {
                            self.errors.push(SemanticError::new(
                                /* span */,
                                SemanticErrorKind::TypeMismatch,
                                format!(
                                    "nested function type mismatch: expected {}, found {}",
                                    expected_type, init_type
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }
    
    // ... rest of code ...
}
```

#### 2.3: Add Function Type Compatibility Checker

Add a new method to `SemanticAnalyzer`:

```rust
impl SemanticAnalyzer {
    /// Check if two function types are compatible
    /// This is used when assigning nested functions to variables or passing them as arguments
    fn check_function_type_compatibility(&self, actual: &Type, expected: &Type) -> bool {
        match (actual, expected) {
            (
                Type::Function {
                    params: actual_params,
                    return_type: actual_ret,
                },
                Type::Function {
                    params: expected_params,
                    return_type: expected_ret,
                },
            ) => {
                // Check parameter count
                if actual_params.len() != expected_params.len() {
                    return false;
                }
                
                // Check each parameter type
                for (actual_param, expected_param) in actual_params.iter().zip(expected_params.iter()) {
                    if !self.type_env.is_compatible(actual_param, expected_param) {
                        return false;
                    }
                }
                
                // Check return type
                self.type_env.is_compatible(actual_ret, expected_ret)
            }
            _ => false,
        }
    }
}
```

#### 2.4: Handle Nested Functions in Expression Analysis

When a nested function name is used in an expression, it should resolve to its function type.

**Location**: In `analyze_expression()` for `Expression::Ident`

**Current code** (approximate):
```rust
Expression::Ident(ident) => {
    if let Some(symbol) = self.symbol_table.lookup(&ident.name) {
        Ok(symbol.ty.clone())
    } else {
        Err(SemanticError::new(
            /* span */,
            SemanticErrorKind::UndefinedVariable,
            format!("undefined variable '{}'", ident.name),
        ))
    }
}
```

**This should already work** because nested functions are registered in the symbol table with their function type.

#### 2.5: Handle Nested Functions as Function Arguments

When analyzing function calls, check if arguments are nested functions and verify type compatibility.

**Location**: In `analyze_expression()` for `Expression::Call`

**Add this logic**:

```rust
Expression::Call { func, args } => {
    let func_type = self.analyze_expression(func)?;
    
    if let Type::Function { params, return_type } = func_type {
        // Check argument count
        if args.len() != params.len() {
            return Err(SemanticError::new(
                /* span */,
                SemanticErrorKind::ArgumentCountMismatch,
                format!("expected {} arguments, found {}", params.len(), args.len()),
            ));
        }
        
        // Check each argument type
        for (i, (arg, expected_param_type)) in args.iter().zip(params.iter()).enumerate() {
            let arg_type = self.analyze_expression(arg)?;
            
            // Special handling for function types (nested functions as arguments)
            if let Type::Function { .. } = arg_type {
                if let Type::Function { .. } = expected_param_type {
                    if !self.check_function_type_compatibility(&arg_type, expected_param_type) {
                        return Err(SemanticError::new(
                            /* span */,
                            SemanticErrorKind::TypeMismatch,
                            format!(
                                "argument {} type mismatch: expected {}, found {}",
                                i + 1,
                                expected_param_type,
                                arg_type
                            ),
                        ));
                    }
                    continue; // Skip regular compatibility check
                }
            }
            
            // Regular type compatibility check
            if !self.type_env.is_compatible(&arg_type, expected_param_type) {
                return Err(SemanticError::new(
                    /* span */,
                    SemanticErrorKind::TypeMismatch,
                    format!(
                        "argument {} type mismatch: expected {}, found {}",
                        i + 1,
                        expected_param_type,
                        arg_type
                    ),
                ));
            }
        }
        
        Ok(*return_type)
    } else {
        Err(SemanticError::new(
            /* span */,
            SemanticErrorKind::NotCallable,
            format!("expression is not callable"),
        ))
    }
}
```

#### 2.6: Handle Nested Functions as Return Values

When analyzing return statements, check if the returned value is a nested function.

**Location**: In `analyze_statement()` for `Statement::Return`

**Add this logic**:

```rust
Statement::Return(expr) => {
    if let Some(return_expr) = expr {
        let return_type = self.analyze_expression(return_expr)?;
        
        if let Some(expected_return_type) = &self.expected_return_type {
            // Special handling for function types
            if let Type::Function { .. } = return_type {
                if let Type::Function { .. } = expected_return_type {
                    if !self.check_function_type_compatibility(&return_type, expected_return_type) {
                        self.errors.push(SemanticError::new(
                            /* span */,
                            SemanticErrorKind::TypeMismatch,
                            format!(
                                "return type mismatch: expected {}, found {}",
                                expected_return_type, return_type
                            ),
                        ));
                    }
                    return;
                }
            }
            
            // Regular type compatibility check
            if !self.type_env.is_compatible(&return_type, expected_return_type) {
                self.errors.push(SemanticError::new(
                    /* span */,
                    SemanticErrorKind::TypeMismatch,
                    format!(
                        "return type mismatch: expected {}, found {}",
                        expected_return_type, return_type
                    ),
                ));
            }
        }
    }
}
```

### Step 3: Add Tests

Add comprehensive tests to `src/nested_function_tests.rs`:

```rust
// ============================================================================
// Test Category 4: Type Checking for Nested Functions
// ============================================================================

#[test]
fn test_nested_function_assigned_to_variable() {
    let source = r#"
void outer() {
    int add(int x, int y) {
        return x + y;
    }
    
    // Assign nested function to variable
    let adder = add;
}
"#;
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();
    
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&file);
    
    assert!(analyzer.errors().is_empty(), "Should allow assigning nested function to variable");
}

#[test]
fn test_nested_function_type_mismatch() {
    let source = r#"
void outer() {
    int add(int x, int y) {
        return x + y;
    }
    
    // Type mismatch: add takes 2 params, but variable expects 1
    int(int) wrong = add;
}
"#;
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();
    
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&file);
    
    assert!(!analyzer.errors().is_empty(), "Should detect type mismatch");
}

#[test]
fn test_nested_function_as_parameter() {
    let source = r#"
void apply(int(int) func, int value) {
    return func(value);
}

void outer() {
    int double(int x) {
        return x * 2;
    }
    
    // Pass nested function as argument
    int result = apply(double, 5);
}
"#;
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();
    
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&file);
    
    assert!(analyzer.errors().is_empty(), "Should allow passing nested function as parameter");
}

#[test]
fn test_nested_function_as_return_value() {
    let source = r#"
int(int) make_adder(int n) {
    int add_n(int x) {
        return x + n;
    }
    return add_n;
}
"#;
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();
    
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&file);
    
    assert!(analyzer.errors().is_empty(), "Should allow returning nested function");
}

#[test]
fn test_multiple_nested_functions_same_captures() {
    let source = r#"
void outer() {
    int x = 10;
    
    int add_x(int y) {
        return x + y;
    }
    
    int mul_x(int y) {
        return x * y;
    }
    
    // Both functions capture x
    int a = add_x(5);
    int b = mul_x(5);
}
"#;
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();
    
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&file);
    
    assert!(analyzer.errors().is_empty(), "Should allow multiple nested functions capturing same variable");
}

#[test]
fn test_nested_function_parameter_type_compatibility() {
    let source = r#"
void outer() {
    int add(int x, int y) {
        return x + y;
    }
    
    // Compatible: same signature
    int(int, int) func1 = add;
    
    // Incompatible: different parameter types
    float(float, float) func2 = add; // Should error
}
"#;
    let mut parser = Parser::new(source).unwrap();
    let file = parser.parse_file().unwrap();
    
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&file);
    
    // Should have one error for func2
    assert_eq!(analyzer.errors().len(), 1, "Should detect parameter type mismatch");
}
```

## Implementation Checklist

### Phase 1: Core Type Checking
- [ ] Verify nested functions are registered with correct `Type::Function` in symbol table
- [ ] Add `check_function_type_compatibility()` method to SemanticAnalyzer
- [ ] Test: Nested function assigned to variable with matching type
- [ ] Test: Nested function assigned to variable with mismatched type (should error)

### Phase 2: Function Parameters
- [ ] Enhance `Expression::Call` analysis to handle function-typed arguments
- [ ] Test: Nested function passed as parameter with matching type
- [ ] Test: Nested function passed as parameter with mismatched type (should error)

### Phase 3: Return Values
- [ ] Enhance `Statement::Return` analysis to handle function-typed returns
- [ ] Test: Nested function returned from function with matching type
- [ ] Test: Nested function returned from function with mismatched type (should error)

### Phase 4: Edge Cases
- [ ] Test: Multiple nested functions capturing same variables
- [ ] Test: Nested function with no captures
- [ ] Test: Complex function types (functions returning functions)

### Phase 5: Integration
- [ ] Run all existing tests to ensure no regressions
- [ ] Update documentation
- [ ] Commit with message: `feat(task-17.3): implement nested function type checking`

## Common Pitfalls

1. **Forgetting to check parameter count**: Always verify the number of parameters matches before checking types.

2. **Not handling void return types**: Ensure `Type::Primitive(PrimitiveType::Void)` is handled correctly.

3. **Capture analysis interference**: Type checking should not interfere with capture analysis (which is already implemented).

4. **Symbol table scope issues**: Nested functions should only be visible in their enclosing function's scope.

## Testing Strategy

1. **Unit tests**: Test each type checking scenario individually
2. **Integration tests**: Test nested functions in realistic code examples
3. **Error tests**: Verify appropriate errors for type mismatches
4. **Edge cases**: Test boundary conditions and complex scenarios

## Success Criteria

Task 17.3 is complete when:
- ✅ Nested functions can be assigned to variables with type checking
- ✅ Nested functions can be passed as function parameters with type checking
- ✅ Nested functions can be returned from functions with type checking
- ✅ Function pointer types work correctly for parameters
- ✅ Type compatibility is verified when passing nested functions
- ✅ Multiple nested functions can capture the same variables
- ✅ All tests pass
- ✅ No regressions in existing functionality

## Next Steps

After completing Task 17.3:
- **Task 17.4**: Implement code generation (translate to Rust closures)
- **Task 17.5**: Add validation rules (no static nested functions, no multi-level nesting)
- **Task 17.6**: Write property test for nested function translation

## References

- **Requirements**: Phase 1 requirements 59.8, 59.9, 59.10, 59.22, 59.23, 59.25
- **Related Code**: 
  - `src/semantic.rs` lines 1260-1385 (nested function analysis)
  - `src/ast.rs` lines 207-213 (NestedFunction AST)
  - `src/ast.rs` lines 325-328 (Function type)
- **Tests**: `src/nested_function_tests.rs`

---

**Created**: January 31, 2026  
**Task**: 17.3 - Add nested function type checking  
**Estimated Time**: 4-6 hours
