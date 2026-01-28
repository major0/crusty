# Semantic Analyzer

## Introduction

The semantic analyzer validates program correctness beyond syntax, including type checking and scope resolution.

## Rationale

Catching semantic errors before code generation provides better error messages and prevents invalid output.

## Responsibilities

### Type Checking
- Verify operand types match operator requirements
- Check function argument types match parameter types
- Validate return types match function declarations
- Ensure assignments have compatible types

### Scope Resolution
- Track variable declarations and their scopes
- Resolve identifier references to declarations
- Detect undefined variable usage
- Detect duplicate declarations in same scope

### Symbol Table
```rust
pub struct SymbolTable {
    scopes: Vec<Scope>,
}

pub struct Scope {
    symbols: HashMap<String, Symbol>,
    parent: Option<usize>,
}

pub struct Symbol {
    pub name: String,
    pub ty: Type,
    pub kind: SymbolKind,
}

pub enum SymbolKind {
    Variable { mutable: bool },
    Function,
    Type,
    Constant,
}
```

### Type Environment
```rust
pub struct TypeEnvironment {
    types: HashMap<String, TypeInfo>,
}

pub struct TypeInfo {
    pub name: String,
    pub kind: TypeKind,
    pub fields: Option<Vec<Field>>,
    pub methods: Vec<Function>,
}
```

## Error Types

- Type mismatch
- Undefined variable
- Undefined type
- Duplicate declaration
- Invalid operation for type
- Missing return value
- Unreachable code
