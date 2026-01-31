# Design Document: Crusty Compiler Phase 3

## Overview

Phase 3 implements **bidirectional transpilation** between Crusty and Rust, enabling:
1. Rust → Crusty transpilation (new)
2. Crusty → Rust transpilation (Phase 1, existing)
3. Round-trip validation (Crusty → Rust → Crusty)

This proves Crusty is a true syntactic layer over Rust with stable, complete syntax.

## Architecture

### High-Level Flow

```
┌─────────────────────────────────────────────────────────────┐
│                  Bidirectional Transpilation                 │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  Crusty Source          Unified AST           Rust Source    │
│      (.crst)                                     (.rs)       │
│        │                    │                      │         │
│        │                    │                      │         │
│        ▼                    │                      ▼         │
│   Crusty Parser             │                 Rust Parser    │
│   (Phase 1)                 │                 (syn crate)    │
│        │                    │                      │         │
│        └────────────────────┼──────────────────────┘         │
│                             │                                │
│                             ▼                                │
│                      Unified AST                             │
│                             │                                │
│                ┌────────────┴────────────┐                   │
│                │                         │                   │
│                ▼                         ▼                   │
│         Crusty CodeGen              Rust CodeGen             │
│         (Phase 1 + new)             (Phase 1)                │
│                │                         │                   │
│                ▼                         ▼                   │
│         Crusty Source               Rust Source              │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

## Component 1: Rust Parser Integration

### Purpose
Parse Rust source code into the unified AST representation.

### Design Approach

**Leverage syn crate**: Use Rust's official parsing library
- Mature, well-tested, tracks Rust syntax changes
- Provides complete Rust AST (syn::File, syn::Item, etc.)
- Handles all Rust syntax correctly

**Convert to Unified AST**: Map syn types to our AST types
- syn::File → ast::File
- syn::Item → ast::Item
- syn::Expr → ast::Expression
- syn::Type → ast::Type
- syn::Stmt → ast::Statement


### Implementation Strategy

```rust
// src/rust_parser.rs

pub struct RustParser {
    // Uses syn crate internally
}

impl RustParser {
    pub fn parse_file(source: &str) -> Result<ast::File, ParseError> {
        // 1. Parse with syn
        let syn_file = syn::parse_file(source)?;
        
        // 2. Convert to unified AST
        let ast_file = convert_syn_file(syn_file)?;
        
        Ok(ast_file)
    }
}

fn convert_syn_file(syn_file: syn::File) -> Result<ast::File> {
    // Convert items
    let items = syn_file.items
        .into_iter()
        .map(convert_syn_item)
        .collect::<Result<Vec<_>>>()?;
    
    Ok(ast::File { items })
}
```

### Conversion Rules

| Rust (syn)           | Crusty (unified AST) |
|---------------------|---------------------|
| syn::ItemFn         | ast::Function       |
| syn::ItemStruct     | ast::Struct         |
| syn::ItemEnum       | ast::Enum           |
| syn::ItemType       | ast::Typedef        |
| syn::ItemMod        | ast::Namespace      |
| syn::ItemUse        | ast::Use            |
| syn::ItemImpl       | ast::Impl           |
| syn::ItemTrait      | ast::Trait (new)    |

## Component 2: Crusty Code Generator (Rust → Crusty)

### Purpose
Generate Crusty source code from unified AST (reverse of Phase 1).

### Translation Rules

#### Functions
```rust
// Rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Crusty
int add(int a, int b) {
    return a + b;
}
```

#### Match → Switch
```rust
// Rust
match value {
    1 => println!("one"),
    2 => println!("two"),
    _ => println!("other"),
}

// Crusty
switch (value) {
    case 1: __println__("one"); break;
    case 2: __println__("two"); break;
    default: __println__("other"); break;
}
```

#### Impl Blocks → Struct Methods
```rust
// Rust
impl Point {
    fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }
}

// Crusty
struct Point {
    int x;
    int y;
    
    static Point new(int x, int y) {
        return Point { x, y };
    }
}
```

#### Type::method() → @Type.method()
```rust
// Rust
Option::None
Vec::new()

// Crusty
@Option.None
@Vec.new()
```

#### Turbofish → Parentheses
```rust
// Rust
Vec::<i32>::new()
collect::<Vec<_>>()

// Crusty
@Vec(i32).new()
collect(@Vec(_))
```

#### Macros
```rust
// Rust
println!("hello")
vec![1, 2, 3]

// Crusty
__println__("hello")
__vec__[1, 2, 3]
```

#### Labels
```rust
// Rust
'outer: loop {
    break 'outer;
}

// Crusty
.outer: loop {
    break outer;
}
```


## Component 3: Round-Trip Validation

### Purpose
Validate that transpilation preserves semantic meaning in both directions.

### Validation Strategy

```
Crusty Source
     │
     ▼
Parse (Phase 1)
     │
     ▼
  AST₁
     │
     ▼
Generate Rust (Phase 1)
     │
     ▼
Rust Source
     │
     ▼
Parse Rust (Phase 3)
     │
     ▼
  AST₂
     │
     ▼
Generate Crusty (Phase 3)
     │
     ▼
Crusty Source'
     │
     ▼
Parse (Phase 1)
     │
     ▼
  AST₃

Validate: AST₁ ≈ AST₃ (semantically equivalent)
```

### Semantic Equivalence

Two ASTs are semantically equivalent if:
1. Same structure (same items, statements, expressions)
2. Same types
3. Same control flow
4. Comments may differ in position but must be preserved

### Property-Based Testing

**Property 1: Round-trip preserves semantics**
```
For all valid Crusty programs P:
  compile(P) == compile(crusty(rust(P)))
```

**Property 2: Bidirectional consistency**
```
For all valid Rust programs R:
  compile(R) == compile(rust(crusty(R)))
```

## Component 4: Unified AST Enhancements

### New AST Nodes for Rust Features

```rust
// src/ast.rs additions

pub enum Item {
    // ... existing variants ...
    Trait(Trait),        // New: Rust trait
    Impl(Impl),          // Enhanced: impl blocks
}

pub struct Trait {
    pub name: Ident,
    pub generics: Vec<GenericParam>,
    pub methods: Vec<Function>,
    pub doc_comments: Vec<String>,
}

pub struct Impl {
    pub trait_name: Option<Ident>,  // None for inherent impl
    pub self_type: Type,
    pub methods: Vec<Function>,
}
```

### AST Normalization

Before comparison, normalize AST:
1. Remove whitespace differences
2. Normalize equivalent constructs
3. Normalize type representations
4. Preserve semantic meaning

## Implementation Phases

### Phase 3.1: Rust Parser Integration
1. Add syn dependency
2. Create rust_parser.rs module
3. Implement syn → unified AST conversion
4. Test with simple Rust programs

### Phase 3.2: Crusty Code Generation
1. Extend codegen.rs for Crusty target
2. Implement Rust → Crusty translation rules
3. Test with simple Rust programs
4. Verify generated Crusty compiles

### Phase 3.3: Round-Trip Validation
1. Implement round-trip test framework
2. Add AST comparison logic
3. Add semantic equivalence checks
4. Test with Crusty programs

### Phase 3.4: Advanced Features
1. Handle traits (VTable translation)
2. Handle lifetimes
3. Handle advanced generics
4. Handle Rust std library

### Phase 3.5: Testing and Polish
1. Property-based tests
2. Corpus testing with real Rust code
3. Performance optimization
4. Documentation

## Testing Strategy

### Unit Tests
- Test each syn → AST conversion
- Test each Rust → Crusty translation rule
- Test AST normalization
- Test semantic equivalence checks

### Integration Tests
- Test end-to-end Rust → Crusty transpilation
- Test round-trip with various programs
- Test with Rust std library code
- Test error handling

### Property-Based Tests
- Round-trip preserves semantics
- Bidirectional consistency
- Generated code compiles
- AST equivalence is transitive

### Corpus Testing
- Test with crates.io packages
- Test with Rust std library
- Test with real-world projects
- Measure success rate

## Correctness Properties

**Property 1: Round-Trip Semantic Preservation**
```
For all Crusty programs C:
  semantics(C) == semantics(crusty(rust(C)))
```
**Validates: Requirements 5.1, 5.2**

**Property 2: Bidirectional Consistency**
```
For all Rust programs R:
  semantics(R) == semantics(rust(crusty(R)))
```
**Validates: Requirements 5.2**

**Property 3: Structure Preservation**
```
For all programs P:
  structure(P) ≈ structure(round_trip(P))
```
**Validates: Requirements 5.3**

**Property 4: Comment Preservation**
```
For all programs P with comments:
  comments(P) ⊆ comments(round_trip(P))
```
**Validates: Requirements 5.4, 5.5**

## Success Criteria

Phase 3 is successful when:
- ✅ 95%+ of Rust std library code transpiles to Crusty
- ✅ Round-trip tests pass for all Phase 1 examples
- ✅ Property-based tests pass with 1000+ iterations
- ✅ Generated Crusty code is readable and idiomatic
- ✅ Performance is acceptable (< 1s for typical files)

## Performance Targets

- Rust parsing: < 100ms per file
- AST conversion: < 50ms per file
- Crusty generation: < 100ms per file
- Round-trip validation: < 500ms per file

## Future Enhancements

Post-Phase 3 improvements:
- Async/await support
- Procedural macro translation
- Advanced trait features (GATs, etc.)
- Const generics
- Incremental transpilation
- Parallel processing
