# Abstract Syntax Tree (AST)

## Introduction

The AST is a unified representation that can express both Crusty and Rust programs, enabling bidirectional transpilation.

## Rationale

A shared AST allows the same semantic analysis and code generation logic to work for both source languages.

## Core Types

### File
```rust
pub struct File {
    pub items: Vec<Item>,
    pub doc_comments: Vec<String>,
}
```

### Items
```rust
pub enum Item {
    Function(Function),
    Struct(Struct),
    Enum(Enum),
    Typedef(Typedef),
    Namespace(Namespace),
    Use(Use),
    Extern(Extern),
    Const(Const),
    Static(Static),
    MacroDefinition(MacroDefinition),
}
```

### Function
```rust
pub struct Function {
    pub visibility: Visibility,
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Block,
    pub doc_comments: Vec<String>,
}
```

### Statements
```rust
pub enum Statement {
    Let { name: Ident, ty: Option<Type>, init: Option<Expression>, mutable: bool },
    Var { name: Ident, ty: Option<Type>, init: Option<Expression> },
    Const { name: Ident, ty: Type, value: Expression },
    Expr(Expression),
    Return(Option<Expression>),
    If { condition: Expression, then_block: Block, else_block: Option<Block> },
    While { condition: Expression, body: Block },
    For { init: Box<Statement>, condition: Expression, increment: Expression, body: Block },
    Break,
    Continue,
}
```

### Expressions
```rust
pub enum Expression {
    Literal(Literal),
    Ident(Ident),
    Binary { op: BinaryOp, left: Box<Expression>, right: Box<Expression> },
    Unary { op: UnaryOp, expr: Box<Expression> },
    Call { func: Box<Expression>, args: Vec<Expression> },
    FieldAccess { expr: Box<Expression>, field: Ident },
    Index { expr: Box<Expression>, index: Box<Expression> },
    Cast { expr: Box<Expression>, ty: Type },
    StructInit { ty: Type, fields: Vec<(Ident, Expression)> },
    MacroCall { name: Ident, args: TokenStream },
}
```

### Types
```rust
pub enum Type {
    Primitive(PrimitiveType),
    Named(Ident),
    Pointer(Box<Type>),
    Reference { ty: Box<Type>, mutable: bool },
    Array { ty: Box<Type>, size: Option<usize> },
    Tuple(Vec<Type>),
    Function { params: Vec<Type>, return_type: Box<Type> },
}
```
