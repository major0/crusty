// Copyright (c) 2026 Mark Ferrell
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
#[allow(dead_code)]
pub enum Item {
    Function(Function),
    Struct(Struct),
    Enum(Enum),
    Typedef(Typedef),
    Namespace(Namespace),
    Import(Import),
    Export(Export),
    Extern(Extern),
    Const(Const),
    Static(Static),
    MacroDefinition(MacroDefinition),
    ConditionalCompilation(ConditionalCompilation),
    /// An item-position macro invocation, e.g. `bitflags! { … }` or
    /// `lazy_static! { … }` at module level. The `args` hold the raw body span
    /// (passed to rustc verbatim, see ADR-0001) and `delimiter` records the
    /// invocation brackets so `name! { … }` is re-emitted with the same shape.
    /// Distinct from `Expression::MacroCall`, which is a macro used in value
    /// position inside a function body.
    MacroInvocation {
        name: Ident,
        args: Vec<Token>,
        delimiter: MacroDelimiter,
    },
    /// Raw Rust items emitted verbatim, from a `__rust__{ … }` escape hatch.
    /// `code` is the exact source between the braces (brace-balanced, trimmed),
    /// passed through to the generated Rust untouched. Used for signatures
    /// Crusty cannot express, e.g. inside an `extern` block.
    RawRust { code: String },
    /// A `typedef … @Type` implementation block (PS-09 Unit 2, ADR-0015).
    /// `typedef struct { methods } @Point;` → `impl Point { … }`;
    /// `typedef default { methods } @Point;` → `impl Default for Point`;
    /// `typedef struct { methods } @Point.name;` → a named organisational block
    /// merged into one `impl Point` at codegen. A dedicated item because the
    /// `@Type` target and the trait form do not fit the struct-definition shape.
    ImplBlock(ImplBlock),
    /// A `typedef trait { … } Name;` trait definition (PS-09 Unit 3 Part B,
    /// ADR-0016). Lowers to `pub trait Name<…> { … }`. A method with
    /// `body: None` is a required signature; `body: Some(_)` is a default
    /// method. Additive variant: the `@Type` impl form (`ImplBlock`) carries
    /// a trait NAME; this item DEFINES one.
    Trait(Trait),
}

/// A `typedef trait { … } Name;` trait definition (PS-09 Unit 3 Part B,
/// ADR-0016).
#[derive(Debug, Clone, PartialEq)]
pub struct Trait {
    pub visibility: Visibility,
    /// The trait's name (the `Name` in `typedef trait { … } Name;`).
    pub name: Ident,
    /// Generic parameters, e.g. `typedef trait Container<T> { … }`. Empty for
    /// a non-generic trait.
    pub generic_params: Vec<GenericParam>,
    /// Trait methods. A method with `body: None` is a required signature;
    /// `body: Some(_)` is a default method.
    pub methods: Vec<Function>,
    pub doc_comments: Vec<String>,
    pub attributes: Vec<Attribute>,
}

/// A `typedef … @Type` implementation block (PS-09 Unit 2, ADR-0015).
#[derive(Debug, Clone, PartialEq)]
pub struct ImplBlock {
    pub visibility: Visibility,
    /// The type the methods are implemented on (the `@Type` target).
    pub target: Ident,
    /// `Some("Default")` for the `typedef default { … } @Type;` form, which
    /// lowers to `impl Default for Type`; `None` for an inherent `impl Type`.
    pub trait_name: Option<Ident>,
    /// The optional `.name` on `@Type.name` — purely organisational; named
    /// blocks for one type merge into a single `impl` in codegen.
    pub name: Option<Ident>,
    pub methods: Vec<Function>,
    pub doc_comments: Vec<String>,
    pub attributes: Vec<Attribute>,
}

impl Item {
    /// Force this item's visibility to Public (PS-17 explicit `pub` modifier).
    /// Crusty items default to public already and `static` makes a function
    /// private, so `pub` is an explicit opt-in to the public default. The
    /// grammar forbids `pub static`, so a private item never reaches here.
    /// Variants without a visibility field are returned unchanged.
    pub fn into_public(self) -> Self {
        match self {
            Item::Function(mut f) => {
                f.visibility = Visibility::Public;
                Item::Function(f)
            }
            Item::Struct(mut s) => {
                s.visibility = Visibility::Public;
                Item::Struct(s)
            }
            Item::Enum(mut e) => {
                e.visibility = Visibility::Public;
                Item::Enum(e)
            }
            Item::Typedef(mut t) => {
                t.visibility = Visibility::Public;
                Item::Typedef(t)
            }
            Item::Const(mut c) => {
                c.visibility = Visibility::Public;
                Item::Const(c)
            }
            Item::Static(mut s) => {
                s.visibility = Visibility::Public;
                Item::Static(s)
            }
            Item::ImplBlock(mut i) => {
                i.visibility = Visibility::Public;
                Item::ImplBlock(i)
            }
            Item::Trait(mut t) => {
                t.visibility = Visibility::Public;
                Item::Trait(t)
            }
            other => other,
        }
    }
}

/// Conditional-compilation block (`#ifdef FOO` / `#ifndef FOO` … `#endif`).
///
/// Lowers to `#[cfg(FOO)]` (or `#[cfg(not(FOO))]` when negated) applied to each
/// contained item (see ADR-0001).
#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalCompilation {
    pub cfg: Ident,
    pub negated: bool,
    pub items: Vec<Item>,
}

/// Function declaration
/// A generic parameter on a function / struct / enum definition (PS-08,
/// ADR-0011). Bounds and const-generic types are carried; trait bounds are
/// RAW strings passed verbatim to rustc (Crusty does not model them).
#[derive(Debug, Clone, PartialEq)]
pub enum GenericParam {
    /// A type parameter, e.g. `T` or `T: Clone + Debug`. `bounds` holds each
    /// bound as a raw string (`"Clone"`, `"Debug"`); empty when unbounded.
    Type { name: Ident, bounds: Vec<String> },
    /// A lifetime parameter, e.g. `'a`. `name` excludes the leading `'`.
    Lifetime { name: Ident },
    /// A const generic, e.g. `const N: usize`.
    Const { name: Ident, ty: Type },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub visibility: Visibility,
    pub name: Ident,
    /// Generic parameter list (PS-08). Empty for a non-generic function.
    pub generic_params: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    /// `Some(block)` for a definition, `None` for a body-less declaration
    /// (an `extern` block function). A declaration emits a `;` instead of a
    /// `{ … }` and is not body-checked by the semantic analyzer.
    pub body: Option<Block>,
    /// `true` when the signature ends with C-style varargs (`...`). Only
    /// meaningful for body-less `extern` declarations — Rust forbids variadics
    /// on a defined or safe function — so it is `false` for every ordinary
    /// function. Codegen emits a trailing `, ...` after the named parameters.
    pub is_variadic: bool,
    /// `true` when the function is declared `unsafe` (PS-09 Unit 1, ADR-0015).
    /// Codegen emits `unsafe fn …`; Crusty does no unsafety analysis (ADR-0001).
    pub is_unsafe: bool,
    /// Raw `where` clause span (PS-08), e.g. `"T: Clone, U: Debug"`, carried
    /// verbatim to rustc. `None` when absent.
    pub where_clause: Option<String>,
    pub doc_comments: Vec<String>,
    pub attributes: Vec<Attribute>,
}

/// Struct definition
#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub visibility: Visibility,
    pub name: Ident,
    /// Generic parameter list (PS-08). Empty for a non-generic struct.
    pub generic_params: Vec<GenericParam>,
    pub fields: Vec<Field>,
    pub methods: Vec<Function>,
    /// Raw `where` clause span (PS-08), carried verbatim to rustc.
    pub where_clause: Option<String>,
    pub doc_comments: Vec<String>,
    pub attributes: Vec<Attribute>,
}

/// Enum definition
#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub visibility: Visibility,
    pub name: Ident,
    /// Generic parameter list (PS-08). Empty for a non-generic enum.
    pub generic_params: Vec<GenericParam>,
    pub variants: Vec<EnumVariant>,
    /// Raw `where` clause span (PS-08), carried verbatim to rustc.
    pub where_clause: Option<String>,
    pub doc_comments: Vec<String>,
    pub attributes: Vec<Attribute>,
}

/// Attribute (e.g., #[derive(Debug)], #[test])
#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: Ident,
    pub args: Vec<AttributeArg>,
}

/// Attribute argument
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeArg {
    Ident(Ident),
    Literal(Literal),
    NameValue { name: Ident, value: Literal },
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

/// Import directive (#import)
#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    pub path: Vec<Ident>,
    pub alias: Option<Ident>,
}

/// Export directive (#export)
#[derive(Debug, Clone, PartialEq)]
pub struct Export {
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

/// Parameterised macro definition (`#macro __NAME__(a, b) body`).
///
/// The body is captured as a raw source span and passed through to the
/// generated `macro_rules!` unchanged — Crusty does not tokenise or inspect it
/// (see ADR-0001).
#[derive(Debug, Clone, PartialEq)]
pub struct MacroDefinition {
    pub name: Ident,
    pub params: Vec<Ident>,
    pub body: String,
}

/// Macro invocation delimiter style. Preserved from the call site so a Crusty
/// `__vec__[1, 2]` lowers to `vec![1, 2]` rather than `vec!(1, 2)`.
#[derive(Debug, Clone, PartialEq)]
pub enum MacroDelimiter {
    Parens,   // name!(args)
    Brackets, // name![args]
    Braces,   // name!{args}
}

/// Statement types
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
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
    /// Unconditional infinite loop: `loop { … }`. Lowers to Rust `loop`.
    /// Exited with `break` (optionally labeled); a `break <expr>` can yield a
    /// value, matching Rust's loop-as-expression.
    Loop {
        label: Option<Ident>,
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
    /// `break [.label] [value];`. `value` lowers to Rust `break <expr>`, which
    /// yields a value from an enclosing `loop` expression. A labeled break is
    /// written `break .label;` (the label wears the dot, as at its declaration).
    Break {
        label: Option<Ident>,
        value: Option<Expression>,
    },
    Continue(Option<Ident>),
    NestedFunction {
        name: Ident,
        params: Vec<Param>,
        return_type: Option<Type>,
        body: Block,
    },
    /// A statement carrying one or more leading attributes (PS-17), e.g.
    /// `#[cfg(test)] let x = 1;`. The attributes are emitted before the inner
    /// statement; this wrapper keeps the other variants free of an attribute
    /// field that is empty for the overwhelming majority of statements.
    Attributed {
        attributes: Vec<Attribute>,
        stmt: Box<Statement>,
    },
    /// `unsafe { … }` block (PS-09 Unit 1, ADR-0015). Lowers verbatim to a Rust
    /// `unsafe { … }` block; Crusty does no unsafety analysis — rustc owns it
    /// (ADR-0001). A passthrough marker, not a checker.
    Unsafe(Block),
}

/// Expression types
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Expression {
    Literal(Literal),
    Ident(Ident),
    /// `loop { … }` in expression position — yields the value given to
    /// `break <value>` inside it. The statement form is `Statement::Loop`;
    /// this variant exists so `let x = loop { … break v; };` parses.
    Loop {
        label: Option<Ident>,
        body: Box<Block>,
    },
    /// `__thread__ { … }` (PS-09 Unit 4, ADR-0018): spawns the body on a new
    /// OS thread. A value-producing expression whose value is the spawn's
    /// `JoinHandle<T>` — binding it (`let h = __thread__ { … };`) gives the
    /// handle form, dropping it as an expression statement gives
    /// fire-and-forget. Lowers to `std::thread::spawn(move || { body })`.
    Thread {
        body: Box<Block>,
    },
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
        /// Functional-update base (`..base`), e.g. `Point { x: 1, ..p }`.
        /// `None` when the initializer lists every field directly.
        base: Option<Box<Expression>>,
    },
    ArrayLit {
        elements: Vec<Expression>,
    },
    /// Array-repeat literal `[value; count]`, e.g. `[0; 16]`.
    ArrayRepeat {
        value: Box<Expression>,
        count: Box<Expression>,
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
        delimiter: MacroDelimiter,
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
    /// Type-scoped associated access with no call (PS-15), e.g. `@Option.None`
    /// → `Option::None`. Carries no args and no parens; used for associated
    /// constants and unit enum variants.
    TypeScopedPath {
        ty: Type,
        member: Ident,
    },
    ExplicitGenericCall {
        ty: Type,
        generics: Vec<Type>,
        method: Ident,
        args: Vec<Expression>,
    },
    /// Rust-style turbofish call (PS-14), e.g. `Type::method::<G1, G2>(args)`
    /// → emitted verbatim. The type arguments bind to the METHOD, not the
    /// type, so this is distinct from ExplicitGenericCall's `Type::<G>::method`.
    TurbofishCall {
        ty: Type,
        method: Ident,
        generics: Vec<Type>,
        args: Vec<Expression>,
    },
    /// Comma expression: evaluates left, discards result, evaluates and returns right
    /// Used in for-loop increments like: i++, j--
    Comma {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    /// `match` expression (PS-06, ADR-0012). Yields the value of the matched
    /// arm's body, so it is legal in expression position
    /// (`let x = match e { … };`); in statement position it is a
    /// `Statement::Expr` wrapping this. Distinct from the C-style
    /// `Statement::Switch`, which matches value expressions, not patterns.
    Match {
        scrutinee: Box<Expression>,
        arms: Vec<MatchArm>,
    },
    /// Anonymous function (PS-07, ADR-0013/0014): `[ret] __func__(typed-params)
    /// { body }` in expression position. A callable value — lowered to a Rust
    /// closure `|params| -> ret { body }`, which captures enclosing locals.
    /// Distinct from a NAMED nested function, which lowers to a hoisted Rust
    /// `fn` item (ADR-0014). Reuses `Param` (typed, like every function) and
    /// `Block`; `return_type` is `None` for a `void`/unit-returning body.
    AnonFunction {
        params: Vec<Param>,
        return_type: Option<Type>,
        body: Box<Block>,
    },
}

/// One arm of a `match` expression (PS-06). The body is an expression; a
/// `{ … }` block is itself an expression, so both `p => expr` and `p => { … }`
/// use this one shape. `guard` is the optional `if <cond>` after the pattern.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>,
    pub body: Box<Expression>,
}

/// A `match`-arm pattern (PS-06, ADR-0012). Covers the decidable, portable
/// subset structurally; anything Crusty does not model (payload/tuple/struct
/// destructuring) rides `Pattern::Raw` verbatim and is left to rustc, the same
/// escape-hatch move PS-08 uses for generic bounds and `where` clauses.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Pattern {
    /// `_`
    Wildcard,
    /// A literal pattern: `0`, `"x"`, `'a'`, `true`, a radix/suffix `Raw`.
    Literal(Literal),
    /// A bare name that binds the scrutinee, e.g. `n`.
    Binding(Ident),
    /// A range pattern: `1..=5` (inclusive) or `1..5` (exclusive).
    Range {
        lo: Box<Expression>,
        hi: Box<Expression>,
        inclusive: bool,
    },
    /// Alternation: `a | b | c`.
    Or(Vec<Pattern>),
    /// A path pattern naming a unit variant or const, e.g. `Color::Red`.
    Path(Vec<Ident>),
    /// Any Rust pattern Crusty does not model structurally, captured verbatim
    /// and emitted unchanged (e.g. `Some(x)`, `(a, b)`, `Point { x, y }`).
    Raw(String),
}

/// Type expressions
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
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
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    /// Pointer-width unsigned / signed integers (Rust `usize` / `isize`).
    Usize,
    Isize,
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
#[allow(dead_code)]
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
    pub attributes: Vec<Attribute>,
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
#[allow(dead_code)]
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
    ModAssign,
    BitAndAssign,
    BitOrAssign,
    BitXorAssign,
    ShlAssign,
    ShrAssign,
}

impl BinaryOp {
    /// Whether this operator is an assignment or compound-assignment
    /// (`=`, `+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `<<=`, `>>=`).
    /// Assignments are statement-level place-expressions whose value is `()`,
    /// so codegen must not wrap them in parens the way it does value operators.
    pub fn is_assignment(&self) -> bool {
        matches!(
            self,
            BinaryOp::Assign
                | BinaryOp::AddAssign
                | BinaryOp::SubAssign
                | BinaryOp::MulAssign
                | BinaryOp::DivAssign
                | BinaryOp::ModAssign
                | BinaryOp::BitAndAssign
                | BinaryOp::BitOrAssign
                | BinaryOp::BitXorAssign
                | BinaryOp::ShlAssign
                | BinaryOp::ShrAssign
        )
    }

    /// Binary-operator precedence for minimal-parenthesization codegen (CG-1),
    /// matching Rust's precedence. Higher binds tighter. Assignment is lowest
    /// (0); it is emitted without parens via the dedicated path, so its value
    /// here only needs to sit below every value operator.
    ///
    /// Rust order (tight -> loose): `* / %` > `+ -` > `<< >>` > `&` > `^` >
    /// `|` > comparison (`== != < > <= >=`, non-associative) > `&&` > `||`.
    pub fn precedence(&self) -> u8 {
        use BinaryOp::*;
        match self {
            Mul | Div | Mod => 10,
            Add | Sub => 9,
            Shl | Shr => 8,
            BitAnd => 7,
            BitXor => 6,
            BitOr => 5,
            Eq | Ne | Lt | Gt | Le | Ge => 4,
            And => 3,
            Or => 2,
            // Assignment family (emitted unparenthesised through its own path).
            _ => 0,
        }
    }

    /// Whether this is a comparison operator. Rust's comparisons are
    /// non-associative, so a comparison nested directly under another needs
    /// parentheses regardless of side.
    pub fn is_comparison(&self) -> bool {
        use BinaryOp::*;
        matches!(self, Eq | Ne | Lt | Gt | Le | Ge)
    }
}
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
    Null,
    /// A literal whose exact source text must survive to rustc unchanged:
    /// non-decimal integers (`0xFF`, `0o17`, `0b1010`), digit separators
    /// (`1_000`), numeric type suffixes (`0u8`, `2.5f32`), and raw / byte
    /// strings (`r"..."`, `b"..."`). Crusty is a syntax layer (ADR-0001): these
    /// forms are valid Rust as written, so the text is passed through verbatim
    /// rather than round-tripped through i64/f64 (which would drop the radix,
    /// the separators, and — critically — the type suffix). Codegen emits the
    /// stored string as-is.
    Raw(String),
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

    #[allow(dead_code)]
    pub fn empty() -> Self {
        Self {
            statements: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_ident() {
        let ident = Ident::new("test_var");
        assert_eq!(ident.name, "test_var");
    }

    #[test]
    fn test_create_block() {
        let block = Block::new(vec![]);
        assert_eq!(block.statements.len(), 0);

        let empty_block = Block::empty();
        assert_eq!(empty_block.statements.len(), 0);
    }

    #[test]
    fn test_create_function() {
        let func = Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("main"),
            params: vec![],
            return_type: None,
            body: Some(Block::empty()),
            is_variadic: false,
            is_unsafe: false,
            doc_comments: vec![],
            attributes: vec![],
        };

        assert_eq!(func.name.name, "main");
        assert_eq!(func.params.len(), 0);
        assert!(func.return_type.is_none());
    }

    #[test]
    fn test_create_function_with_params() {
        let func = Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("add"),
            params: vec![
                Param {
                    name: Ident::new("a"),
                    ty: Type::Primitive(PrimitiveType::I32),
                },
                Param {
                    name: Ident::new("b"),
                    ty: Type::Primitive(PrimitiveType::I32),
                },
            ],
            return_type: Some(Type::Primitive(PrimitiveType::I32)),
            body: Some(Block::empty()),
            is_variadic: false,
            is_unsafe: false,
            doc_comments: vec![],
            attributes: vec![],
        };

        assert_eq!(func.params.len(), 2);
        assert_eq!(func.params[0].name.name, "a");
        assert_eq!(func.params[1].name.name, "b");
        assert!(func.return_type.is_some());
    }

    #[test]
    fn test_create_struct() {
        let struct_def = Struct {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("Point"),
            fields: vec![
                Field {
                    visibility: Visibility::Public,
                    name: Ident::new("x"),
                    ty: Type::Primitive(PrimitiveType::I32),
                    doc_comments: vec![],
                    attributes: vec![],
                },
                Field {
                    visibility: Visibility::Public,
                    name: Ident::new("y"),
                    ty: Type::Primitive(PrimitiveType::I32),
                    doc_comments: vec![],
                    attributes: vec![],
                },
            ],
            methods: vec![],
            doc_comments: vec![],
            attributes: vec![],
        };

        assert_eq!(struct_def.name.name, "Point");
        assert_eq!(struct_def.fields.len(), 2);
        assert_eq!(struct_def.fields[0].name.name, "x");
        assert_eq!(struct_def.fields[1].name.name, "y");
    }

    #[test]
    fn test_create_enum() {
        let enum_def = Enum {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("Color"),
            variants: vec![
                EnumVariant {
                    name: Ident::new("Red"),
                    value: Some(0),
                },
                EnumVariant {
                    name: Ident::new("Green"),
                    value: Some(1),
                },
                EnumVariant {
                    name: Ident::new("Blue"),
                    value: Some(2),
                },
            ],
            doc_comments: vec![],
            attributes: vec![],
        };

        assert_eq!(enum_def.name.name, "Color");
        assert_eq!(enum_def.variants.len(), 3);
        assert_eq!(enum_def.variants[0].name.name, "Red");
        assert_eq!(enum_def.variants[0].value, Some(0));
    }

    #[test]
    fn test_create_let_statement() {
        let stmt = Statement::Let {
            name: Ident::new("x"),
            ty: Some(Type::Primitive(PrimitiveType::I32)),
            init: Some(Expression::Literal(Literal::Int(42))),
            mutable: false,
        };

        match stmt {
            Statement::Let {
                name,
                ty,
                init,
                mutable,
            } => {
                assert_eq!(name.name, "x");
                assert!(ty.is_some());
                assert!(init.is_some());
                assert!(!mutable);
            }
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_create_if_statement() {
        let stmt = Statement::If {
            condition: Expression::Literal(Literal::Bool(true)),
            then_block: Block::empty(),
            else_block: None,
        };

        match stmt {
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                assert!(matches!(
                    condition,
                    Expression::Literal(Literal::Bool(true))
                ));
                assert_eq!(then_block.statements.len(), 0);
                assert!(else_block.is_none());
            }
            _ => panic!("Expected If statement"),
        }
    }

    #[test]
    fn test_create_while_statement() {
        let stmt = Statement::While {
            label: None,
            condition: Expression::Literal(Literal::Bool(true)),
            body: Block::empty(),
        };

        match stmt {
            Statement::While {
                label,
                condition,
                body,
            } => {
                assert!(label.is_none());
                assert!(matches!(
                    condition,
                    Expression::Literal(Literal::Bool(true))
                ));
                assert_eq!(body.statements.len(), 0);
            }
            _ => panic!("Expected While statement"),
        }
    }

    #[test]
    fn test_create_binary_expression() {
        let expr = Expression::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expression::Literal(Literal::Int(1))),
            right: Box::new(Expression::Literal(Literal::Int(2))),
        };

        match expr {
            Expression::Binary { op, left, right } => {
                assert!(matches!(op, BinaryOp::Add));
                assert!(matches!(*left, Expression::Literal(Literal::Int(1))));
                assert!(matches!(*right, Expression::Literal(Literal::Int(2))));
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_create_unary_expression() {
        let expr = Expression::Unary {
            op: UnaryOp::Neg,
            expr: Box::new(Expression::Literal(Literal::Int(42))),
        };

        match expr {
            Expression::Unary { op, expr } => {
                assert!(matches!(op, UnaryOp::Neg));
                assert!(matches!(*expr, Expression::Literal(Literal::Int(42))));
            }
            _ => panic!("Expected Unary expression"),
        }
    }

    #[test]
    fn test_create_call_expression() {
        let expr = Expression::Call {
            func: Box::new(Expression::Ident(Ident::new("foo"))),
            args: vec![
                Expression::Literal(Literal::Int(1)),
                Expression::Literal(Literal::Int(2)),
            ],
        };

        match expr {
            Expression::Call { func, args } => {
                assert!(matches!(*func, Expression::Ident(_)));
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected Call expression"),
        }
    }

    #[test]
    fn test_create_array_literal() {
        let expr = Expression::ArrayLit {
            elements: vec![
                Expression::Literal(Literal::Int(1)),
                Expression::Literal(Literal::Int(2)),
                Expression::Literal(Literal::Int(3)),
            ],
        };

        match expr {
            Expression::ArrayLit { elements } => {
                assert_eq!(elements.len(), 3);
            }
            _ => panic!("Expected ArrayLit expression"),
        }
    }

    #[test]
    fn test_create_tuple_literal() {
        let expr = Expression::TupleLit {
            elements: vec![
                Expression::Literal(Literal::Int(1)),
                Expression::Literal(Literal::String("test".to_string())),
            ],
        };

        match expr {
            Expression::TupleLit { elements } => {
                assert_eq!(elements.len(), 2);
            }
            _ => panic!("Expected TupleLit expression"),
        }
    }

    #[test]
    fn test_create_primitive_types() {
        let types = vec![
            Type::Primitive(PrimitiveType::Int),
            Type::Primitive(PrimitiveType::I32),
            Type::Primitive(PrimitiveType::I64),
            Type::Primitive(PrimitiveType::U32),
            Type::Primitive(PrimitiveType::U64),
            Type::Primitive(PrimitiveType::Float),
            Type::Primitive(PrimitiveType::F32),
            Type::Primitive(PrimitiveType::F64),
            Type::Primitive(PrimitiveType::Bool),
            Type::Primitive(PrimitiveType::Char),
            Type::Primitive(PrimitiveType::Void),
        ];

        assert_eq!(types.len(), 11);
    }

    #[test]
    fn test_create_pointer_type() {
        let ty = Type::Pointer {
            ty: Box::new(Type::Primitive(PrimitiveType::I32)),
            mutable: true,
        };

        match ty {
            Type::Pointer { ty, mutable } => {
                assert!(matches!(*ty, Type::Primitive(PrimitiveType::I32)));
                assert!(mutable);
            }
            _ => panic!("Expected Pointer type"),
        }
    }

    #[test]
    fn test_create_reference_type() {
        let ty = Type::Reference {
            ty: Box::new(Type::Primitive(PrimitiveType::I32)),
            mutable: false,
        };

        match ty {
            Type::Reference { ty, mutable } => {
                assert!(matches!(*ty, Type::Primitive(PrimitiveType::I32)));
                assert!(!mutable);
            }
            _ => panic!("Expected Reference type"),
        }
    }

    #[test]
    fn test_create_array_type() {
        let ty = Type::Array {
            ty: Box::new(Type::Primitive(PrimitiveType::I32)),
            size: Some(10),
        };

        match ty {
            Type::Array { ty, size } => {
                assert!(matches!(*ty, Type::Primitive(PrimitiveType::I32)));
                assert_eq!(size, Some(10));
            }
            _ => panic!("Expected Array type"),
        }
    }

    #[test]
    fn test_create_tuple_type() {
        let ty = Type::Tuple {
            types: vec![
                Type::Primitive(PrimitiveType::I32),
                Type::Primitive(PrimitiveType::Bool),
            ],
        };

        match ty {
            Type::Tuple { types } => {
                assert_eq!(types.len(), 2);
            }
            _ => panic!("Expected Tuple type"),
        }
    }

    #[test]
    fn test_ast_node_equality() {
        let ident1 = Ident::new("test");
        let ident2 = Ident::new("test");
        let ident3 = Ident::new("other");

        assert_eq!(ident1, ident2);
        assert_ne!(ident1, ident3);
    }

    #[test]
    fn test_ast_node_cloning() {
        let original = Function {
            generic_params: vec![],
            where_clause: None,
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Some(Block::empty()),
            is_variadic: false,
            is_unsafe: false,
            doc_comments: vec![],
            attributes: vec![],
        };

        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_literal_types() {
        let literals = [
            Literal::Int(42),
            Literal::Float(2.5),
            Literal::String("hello".to_string()),
            Literal::Char('a'),
            Literal::Bool(true),
        ];

        assert_eq!(literals.len(), 5);
    }

    #[test]
    fn test_visibility_variants() {
        let public = Visibility::Public;
        let private = Visibility::Private;

        assert_ne!(public, private);
    }

    #[test]
    fn test_binary_operators() {
        let ops = vec![
            BinaryOp::Add,
            BinaryOp::Sub,
            BinaryOp::Mul,
            BinaryOp::Div,
            BinaryOp::Mod,
            BinaryOp::Eq,
            BinaryOp::Ne,
            BinaryOp::Lt,
            BinaryOp::Gt,
            BinaryOp::Le,
            BinaryOp::Ge,
            BinaryOp::And,
            BinaryOp::Or,
            BinaryOp::BitAnd,
            BinaryOp::BitOr,
            BinaryOp::BitXor,
            BinaryOp::Shl,
            BinaryOp::Shr,
        ];

        assert_eq!(ops.len(), 18);
    }

    #[test]
    fn test_unary_operators() {
        let ops = [
            UnaryOp::Not,
            UnaryOp::Neg,
            UnaryOp::Ref,
            UnaryOp::Deref,
            UnaryOp::PreInc,
            UnaryOp::PreDec,
            UnaryOp::PostInc,
            UnaryOp::PostDec,
        ];

        assert_eq!(ops.len(), 8);
    }
}
