// Copyright (c) 2026 Crusty Programming Language
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Abstract Syntax Tree definitions for Crusty and Rust programs.

/// Represents a complete source file
#[derive(Debug, Clone, PartialEq)]
pub struct File {
    pub items: Vec<Item>,
    pub doc_comments: Vec<String>,
}

/// Top-level items in a program
#[derive(Debug, Clone, PartialEq)]
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

/// Function declaration
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub visibility: Visibility,
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Block,
    pub doc_comments: Vec<String>,
}

/// Struct definition
#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub visibility: Visibility,
    pub name: Ident,
    pub fields: Vec<Field>,
    pub methods: Vec<Function>,
    pub doc_comments: Vec<String>,
}

/// Enum definition
#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub visibility: Visibility,
    pub name: Ident,
    pub variants: Vec<EnumVariant>,
    pub doc_comments: Vec<String>,
}

/// Type alias (typedef)
#[derive(Debug, Clone, PartialEq)]
pub struct Typedef {
    pub visibility: Visibility,
    pub name: Ident,
    pub target: Type,
    pub doc_comments: Vec<String>,
}

/// Namespace declaration
#[derive(Debug, Clone, PartialEq)]
pub struct Namespace {
    pub name: Ident,
    pub items: Vec<Item>,
    pub doc_comments: Vec<String>,
}

/// Use/import directive
#[derive(Debug, Clone, PartialEq)]
pub struct Use {
    pub path: Vec<Ident>,
    pub alias: Option<Ident>,
}

/// Extern block
#[derive(Debug, Clone, PartialEq)]
pub struct Extern {
    pub abi: Option<String>,
    pub items: Vec<Item>,
}

/// Constant declaration
#[derive(Debug, Clone, PartialEq)]
pub struct Const {
    pub visibility: Visibility,
    pub name: Ident,
    pub ty: Type,
    pub value: Expression,
    pub doc_comments: Vec<String>,
}

/// Static variable declaration
#[derive(Debug, Clone, PartialEq)]
pub struct Static {
    pub visibility: Visibility,
    pub name: Ident,
    pub ty: Type,
    pub value: Expression,
    pub mutable: bool,
    pub doc_comments: Vec<String>,
}

/// Macro definition (#define)
#[derive(Debug, Clone, PartialEq)]
pub struct MacroDefinition {
    pub name: Ident,
    pub params: Vec<Ident>,
    pub body: Vec<Token>,
}

/// Statement types
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let {
        name: Ident,
        ty: Option<Type>,
        init: Option<Expression>,
        mutable: bool,
    },
    Var {
        name: Ident,
        ty: Option<Type>,
        init: Option<Expression>,
    },
    Const {
        name: Ident,
        ty: Type,
        value: Expression,
    },
    Expr(Expression),
    Return(Option<Expression>),
    If {
        condition: Expression,
        then_block: Block,
        else_block: Option<Block>,
    },
    While {
        label: Option<Ident>,
        condition: Expression,
        body: Block,
    },
    For {
        label: Option<Ident>,
        init: Box<Statement>,
        condition: Expression,
        increment: Expression,
        body: Block,
    },
    ForIn {
        label: Option<Ident>,
        var: Ident,
        iter: Expression,
        body: Block,
    },
    Switch {
        expr: Expression,
        cases: Vec<SwitchCase>,
        default: Option<Block>,
    },
    Break(Option<Ident>),
    Continue(Option<Ident>),
}

/// Expression types
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Literal),
    Ident(Ident),
    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expression>,
    },
    Call {
        func: Box<Expression>,
        args: Vec<Expression>,
    },
    FieldAccess {
        expr: Box<Expression>,
        field: Ident,
    },
    Index {
        expr: Box<Expression>,
        index: Box<Expression>,
    },
    Cast {
        expr: Box<Expression>,
        ty: Type,
    },
    Sizeof {
        ty: Type,
    },
    Ternary {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
    },
    StructInit {
        ty: Type,
        fields: Vec<(Ident, Expression)>,
    },
    ArrayLit {
        elements: Vec<Expression>,
    },
    TupleLit {
        elements: Vec<Expression>,
    },
    Range {
        start: Option<Box<Expression>>,
        end: Option<Box<Expression>>,
        inclusive: bool,
    },
    MacroCall {
        name: Ident,
        args: Vec<Token>,
    },
    RustBlock {
        tokens: Vec<Token>,
    },
    ErrorProp {
        expr: Box<Expression>,
    },
    MethodCall {
        receiver: Box<Expression>,
        method: Ident,
        args: Vec<Expression>,
    },
    TypeScopedCall {
        ty: Type,
        method: Ident,
        args: Vec<Expression>,
    },
    ExplicitGenericCall {
        ty: Type,
        generics: Vec<Type>,
        method: Ident,
        args: Vec<Expression>,
    },
}

/// Type expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Primitive(PrimitiveType),
    Ident(Ident),
    Pointer {
        ty: Box<Type>,
        mutable: bool,
    },
    Reference {
        ty: Box<Type>,
        mutable: bool,
    },
    Array {
        ty: Box<Type>,
        size: Option<usize>,
    },
    Slice {
        ty: Box<Type>,
    },
    Tuple {
        types: Vec<Type>,
    },
    Generic {
        base: Box<Type>,
        args: Vec<Type>,
    },
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    Fallible {
        ty: Box<Type>,
    },
    Auto,
}

/// Primitive types
#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveType {
    Int,
    I32,
    I64,
    U32,
    U64,
    Float,
    F32,
    F64,
    Bool,
    Char,
    Void,
}

/// Placeholder for token stream (will be properly defined in lexer module)
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Ident,
    Literal,
    Operator,
    Delimiter,
    Keyword,
    Other,
}

/// Function parameter
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: Ident,
    pub ty: Type,
}

/// Struct field
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub visibility: Visibility,
    pub name: Ident,
    pub ty: Type,
    pub doc_comments: Vec<String>,
}

/// Enum variant
#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: Ident,
    pub value: Option<i64>,
}

/// Switch case
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase {
    pub values: Vec<Expression>,
    pub body: Block,
}

/// Visibility modifier
#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Private,
}

/// Binary operators
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Comparison
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    // Logical
    And,
    Or,
    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    // Assignment
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
}

/// Unary operators
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Not,
    Neg,
    Ref,
    Deref,
    PreInc,
    PreDec,
    PostInc,
    PostDec,
}

/// Literal values
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),
}

/// Identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident {
    pub name: String,
}

impl Ident {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Block of statements
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
}

impl Block {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }
    
    pub fn empty() -> Self {
        Self { statements: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn placeholder() {
        // Placeholder test
    }
}
