# Design Document: Crusty Transpiler (Compiler) Phase 1

## Overview

The Crusty transpiler (crustyc) is a bidirectional tool that translates between Crusty (a C-like language) and Rust source code. This phase 1 implementation establishes the core transpiler infrastructure and determines which C language features can be preserved while maintaining Rust compatibility.

The transpiler follows a traditional multi-phase architecture:
1. **Lexical Analysis**: Tokenize source code
2. **Parsing**: Build Abstract Syntax Tree (AST)
3. **Semantic Analysis**: Validate types, scopes, and language rules
4. **Code Generation**: Emit target language source code
5. **Compilation**: Invoke rustc to produce binaries (optional)

The design supports bidirectional transpilation, meaning the same AST representation can be generated from either Crusty or Rust source, and can be emitted as either Crusty or Rust code. This enables round-trip validation and interoperability.

### Key Design Principles

- **Shared AST**: Use a unified AST representation that can represent both Crusty and Rust constructs
- **C-like Function Syntax**: Crusty uses C-style function declarations with return types before function names (e.g., `int main()`, `void foo()`), NOT Rust's `fn` keyword syntax
- **Rust Standard Library**: Crusty programs use Rust's std library directly without wrappers
- **Safety First**: Reject C features that violate Rust's safety guarantees
- **Familiar Syntax**: Provide C-like syntax that maps cleanly to Rust semantics
- **Escape Hatch**: Support rust! macro for embedding raw Rust code when needed

## Development Workflow and Infrastructure

### CI/CD Pipeline Architecture

The project uses GitHub Actions for continuous integration and deployment. The CI pipeline ensures code quality through automated testing, linting, and formatting checks.

**Pipeline Configuration** (.github/workflows/ci.yml):

```yaml
name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    name: Test Suite
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable]
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: ${{ matrix.rust }}
          override: true
          components: rustfmt, clippy
      
      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Build
        run: cargo build --verbose
      
      - name: Run tests
        run: cargo test --verbose
      
      - name: Run clippy
        run: cargo clippy -- -D warnings
      
      - name: Check formatting
        run: cargo fmt -- --check
```

**Pipeline Stages**:
1. **Checkout**: Retrieve repository code
2. **Setup**: Install Rust toolchain with rustfmt and clippy
3. **Cache**: Cache Cargo dependencies for faster builds
4. **Build**: Compile all code with `cargo build`
5. **Test**: Run unit tests and property-based tests with `cargo test`
6. **Lint**: Check code quality with `cargo clippy`
7. **Format**: Verify code formatting with `cargo fmt --check`

**Cross-Platform Testing**: The pipeline runs on Linux, macOS, and Windows to ensure compatibility across all major platforms.

**Build Status**: The README includes build status badges showing the current CI status.

### Git Workflow and Commit Strategy

The project follows Conventional Commits specification for structured, machine-readable commit messages.

**Commit Message Format**:
```
type(scope): subject

body

footer
```

**Commit Types**:
- `feat`: New feature implementation
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Adding or updating tests
- `refactor`: Code refactoring without functionality changes
- `chore`: Maintenance tasks
- `style`: Code style changes (formatting, etc.)

**Commit Scope**: References the task number being completed (e.g., `task-2.1`, `task-3.4`)

**Example Commits**:
```
feat(task-2.1): implement parser for function declarations

Implemented parsing for C-style function declarations including:
- Return type before function name
- Typed parameters
- Static function support

Validates: Requirements 6.4, 6.5, 6.6, 6.7
```

```
test(task-2.2): add property tests for parser

Added property-based tests for function declaration parsing
using proptest library. Tests verify round-trip consistency.

Validates: Requirements 6.1, 6.3
```

**Commit Workflow**:
1. Complete a task or sub-task
2. Update tasks.md to mark task as complete
3. Stage all changes: `git add .`
4. Commit with conventional format: `git commit -m "type(scope): subject"`
5. Push to trigger CI: `git push`

**Breaking Changes**: When a commit introduces breaking changes, include `BREAKING CHANGE:` in the footer with a description.

### Pre-Commit Hooks

The project uses pre-commit framework (https://pre-commit.com) to automatically validate code before commits.

**Configuration** (.pre-commit-config.yaml):
```yaml
repos:
  - repo: local
    hooks:
      - id: crustyc-syntax
        name: Crusty Syntax Check
        entry: cargo run --bin crustyc -- --check
        language: system
        files: \.crst$
        pass_filenames: true
      
      - id: cargo-fmt
        name: Cargo Format Check
        entry: cargo fmt --
        language: system
        files: \.rs$
        pass_filenames: true
      
      - id: cargo-clippy
        name: Cargo Clippy
        entry: cargo clippy --
        language: system
        files: \.rs$
        pass_filenames: false
        args: ["-D", "warnings"]
```

**Installation**:
```bash
# Install pre-commit
pip install pre-commit

# Install hooks
pre-commit install
```

**Hook Execution**: Hooks run automatically before each commit. If any hook fails, the commit is prevented and error messages are displayed.

**Skipping Hooks**: In rare cases where hooks need to be skipped: `git commit --no-verify`

**External Project Usage**: External Crusty projects can reference the Crusty repository hooks:
```yaml
repos:
  - repo: https://github.com/major0/crusty.git
    rev: v1.0.0
    hooks:
      - id: crustyc-syntax
      - id: cargo-fmt
      - id: cargo-clippy
```

### License and Legal

**MIT License**: The project is licensed under the MIT License, providing permissive open-source licensing.

**LICENSE.txt** (root directory):
```
MIT License

Copyright (c) 2024 Crusty Programming Language

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

**Copyright Headers**: All source files include a copyright header:
```rust
// Copyright (c) 2024 Crusty Programming Language
// Licensed under the MIT License. See LICENSE.txt in the project root.
```

### EditorConfig

The project provides EditorConfig support for consistent formatting across editors.

**.editorconfig** (root directory):
```ini
root = true

[*]
charset = utf-8
end_of_line = lf
insert_final_newline = true
trim_trailing_whitespace = true

[*.crst]
indent_style = space
indent_size = 4

[*.rs]
indent_style = space
indent_size = 4

[*.toml]
indent_style = space
indent_size = 2

[*.md]
indent_style = space
indent_size = 2
trim_trailing_whitespace = false

[*.{yml,yaml}]
indent_style = space
indent_size = 2
```

**Editor Support**: Most modern editors and IDEs support EditorConfig automatically (VS Code, IntelliJ, Vim, Emacs, Sublime Text, etc.).

## Architecture

### Component Overview

```
┌─────────────────────────────────────────────────────────────┐
│                         crustyc CLI                          │
│  (Command-line interface, file I/O, option parsing)         │
└──────────────────┬──────────────────────────────────────────┘
                   │
                   ├──────────────────┐
                   │                  │
         ┌─────────▼────────┐  ┌─────▼──────────┐
         │  Crusty Parser   │  │  Rust Parser   │
         │  (Custom parser) │  │  (syn library) │
         └─────────┬────────┘  └─────┬──────────┘
                   │                  │
                   └────────┬─────────┘
                            │
                   ┌────────▼─────────┐
                   │   Unified AST    │
                   │  (Shared repr.)  │
                   └────────┬─────────┘
                            │
                   ┌────────▼──────────┐
                   │ Semantic Analyzer │
                   │ (Type checking,   │
                   │  scope resolution)│
                   └────────┬──────────┘
                            │
                   ┌────────▼──────────┐
                   │  Code Generator   │
                   │ (Crusty/Rust emit)│
                   └────────┬──────────┘
                            │
                   ┌────────▼──────────┐
                   │   Pretty Printer  │
                   │ (Format output)   │
                   └────────┬──────────┘
                            │
                   ┌────────▼──────────┐
                   │  rustc Invoker    │
                   │  (Optional)       │
                   └───────────────────┘
```

### Bidirectional Flow

**Crusty → Rust:**
```
Crusty Source → Crusty Parser → AST → Semantic Analysis → Code Generator → Rust Source → rustc → Binary
```

**Example Crusty Source:**
```crusty
// Crusty function syntax (C-style)
int add(int a, int b) {
    return a + b;
}

void print_message(char* msg) {
    __println__("{}", msg);
}

static int helper(int x) {
    return x * 2;
}
```

**Translates to Rust:**
```rust
pub fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

pub fn print_message(msg: &str) {
    println!("{}", msg);
}

fn helper(x: i32) -> i32 {
    return x * 2;
}
```

**Rust → Crusty:**
```
Rust Source → Rust Parser (syn) → AST → Semantic Analysis → Code Generator → Crusty Source
```

**Round-trip Validation:**
```
Crusty → AST → Crusty' (should be equivalent to original)
```

## Components and Interfaces

### 1. CLI Module

**Responsibility**: Parse command-line arguments, coordinate compilation pipeline, handle file I/O.

**Interface**:
```rust
pub struct CompilerOptions {
    pub input_file: PathBuf,
    pub output_file: Option<PathBuf>,
    pub emit: EmitMode,
    pub from_lang: SourceLanguage,
    pub verbose: bool,
    pub no_compile: bool,
}

pub enum EmitMode {
    Rust,      // Generate Rust source only
    Binary,    // Generate Rust and compile to binary
    Ast,       // Output AST in human-readable format
}

pub enum SourceLanguage {
    Crusty,
    Rust,
}

pub fn parse_args() -> Result<CompilerOptions, String>;
pub fn run_compiler(options: CompilerOptions) -> Result<(), CompilerError>;
```

### 2. Lexer Module

**Responsibility**: Tokenize source code into a stream of tokens.

**Interface**:
```rust
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub text: String,
}

pub struct Span {
    pub start: Position,
    pub end: Position,
}

pub struct Position {
    pub line: usize,
    pub column: usize,
}

pub enum TokenKind {
    // Keywords
    Let, Var, Const, Static, If, Else, While, For, Return,
    Break, Continue, Struct, Enum, Typedef, Namespace, Extern,
    
    // Types
    Int, I32, I64, U32, U64, Float, F32, F64, Bool, Char, Void,
    
    // Operators
    Plus, Minus, Star, Slash, Percent,
    Eq, Ne, Lt, Gt, Le, Ge,
    And, Or, Not,
    BitAnd, BitOr, BitXor, BitNot, Shl, Shr,
    Assign, PlusEq, MinusEq, StarEq, SlashEq,
    Inc, Dec,
    Dot, Arrow, DotDot, DotDotEq,
    
    // Delimiters
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Comma, Semicolon, Colon, DoubleColon, Question,
    
    // Literals
    IntLiteral, FloatLiteral, StringLiteral, CharLiteral, BoolLiteral,
    
    // Identifiers
    Ident,
    
    // Special
    Hash,  // # - for preprocessor directives (#use, #ifdef, etc.)
    Bang,  // ! - for error propagation operator (Rust's ?)
    At,    // @ - for type-scoped calls (@Type.method())
    Eof,
}

pub struct Lexer<'a> {
    source: &'a str,
    position: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self;
    pub fn next_token(&mut self) -> Result<Token, LexError>;
    pub fn peek_token(&mut self) -> Result<Token, LexError>;
}
```

### 3. Parser Module

**Responsibility**: Parse token stream into AST.

**Interface**:
```rust
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self;
    pub fn parse_file(&mut self) -> Result<File, ParseError>;
    
    // Internal parsing methods
    fn parse_item(&mut self) -> Result<Item, ParseError>;
    fn parse_function(&mut self) -> Result<Function, ParseError>;
    fn parse_struct(&mut self) -> Result<Struct, ParseError>;
    fn parse_enum(&mut self) -> Result<Enum, ParseError>;
    fn parse_statement(&mut self) -> Result<Statement, ParseError>;
    fn parse_expression(&mut self) -> Result<Expression, ParseError>;
    fn parse_type(&mut self) -> Result<Type, ParseError>;
}
```

### 4. Rust Parser Module

**Responsibility**: Parse Rust source code using the syn library.

**Interface**:
```rust
pub struct RustParser;

impl RustParser {
    pub fn parse_file(source: &str) -> Result<File, syn::Error>;
    
    // Convert syn AST to our unified AST
    fn convert_syn_file(syn_file: syn::File) -> File;
    fn convert_syn_item(syn_item: syn::Item) -> Item;
    fn convert_syn_expr(syn_expr: syn::Expr) -> Expression;
    fn convert_syn_type(syn_type: syn::Type) -> Type;
}
```

### 5. AST Module

**Responsibility**: Define the unified AST structure that represents both Crusty and Rust programs.

**Key Types**:
```rust
pub struct File {
    pub items: Vec<Item>,
    pub doc_comments: Vec<String>,
}

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

pub struct Function {
    pub visibility: Visibility,
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Block,
    pub doc_comments: Vec<String>,
}

pub struct Param {
    pub name: Ident,
    pub ty: Type,
}

pub struct Struct {
    pub visibility: Visibility,
    pub name: Ident,
    pub fields: Vec<Field>,
    pub methods: Vec<Function>,
    pub doc_comments: Vec<String>,
}

pub struct Field {
    pub visibility: Visibility,
    pub name: Ident,
    pub ty: Type,
    pub doc_comments: Vec<String>,
}

pub struct Enum {
    pub visibility: Visibility,
    pub name: Ident,
    pub variants: Vec<EnumVariant>,
    pub doc_comments: Vec<String>,
}

pub struct EnumVariant {
    pub name: Ident,
    pub value: Option<i64>,
}

pub struct MacroDefinition {
    pub name: Ident,
    pub params: Vec<Ident>,
    pub body: TokenStream,
}

pub enum Statement {
    Let { name: Ident, ty: Option<Type>, init: Option<Expression>, mutable: bool },
    Var { name: Ident, ty: Option<Type>, init: Option<Expression> },
    Const { name: Ident, ty: Type, value: Expression },
    Expr(Expression),
    Return(Option<Expression>),
    If { condition: Expression, then_block: Block, else_block: Option<Block> },
    While { condition: Expression, body: Block },
    For { init: Box<Statement>, condition: Expression, increment: Expression, body: Block },
    ForIn { var: Ident, iter: Expression, body: Block },
    Switch { expr: Expression, cases: Vec<SwitchCase>, default: Option<Block> },
    Break,
    Continue,
}

pub enum Expression {
    Literal(Literal),
    Ident(Ident),
    Binary { op: BinaryOp, left: Box<Expression>, right: Box<Expression> },
    Unary { op: UnaryOp, expr: Box<Expression> },
    Call { func: Box<Expression>, args: Vec<Expression> },
    FieldAccess { expr: Box<Expression>, field: Ident },
    Index { expr: Box<Expression>, index: Box<Expression> },
    Cast { expr: Box<Expression>, ty: Type },
    Sizeof { ty: Type },
    Ternary { condition: Box<Expression>, then_expr: Box<Expression>, else_expr: Box<Expression> },
    StructInit { ty: Type, fields: Vec<(Ident, Expression)> },
    ArrayLit { elements: Vec<Expression> },
    TupleLit { elements: Vec<Expression> },
    Range { start: Option<Box<Expression>>, end: Option<Box<Expression>>, inclusive: bool },
    MacroCall { name: Ident, args: TokenStream },
    RustBlock { tokens: TokenStream },
    ErrorProp { expr: Box<Expression> },
    MethodCall { receiver: Box<Expression>, method: Ident, args: Vec<Expression> },
    TypeScopedCall { ty: Type, method: Ident, args: Vec<Expression> },
    ExplicitGenericCall { ty: Type, generics: Vec<Type>, method: Ident, args: Vec<Expression> },
}

pub enum Type {
    Primitive(PrimitiveType),
    Ident(Ident),
    Pointer { ty: Box<Type>, mutable: bool },
    Reference { ty: Box<Type>, mutable: bool },
    Array { ty: Box<Type>, size: Option<usize> },
    Slice { ty: Box<Type> },
    Tuple { types: Vec<Type> },
    Generic { base: Box<Type>, args: Vec<Type> },
    Function { params: Vec<Type>, return_type: Box<Type> },
    Fallible { ty: Box<Type> },
    Auto,
}

pub enum Visibility {
    Public,
    Private,
}
```

### 6. Semantic Analyzer Module

**Responsibility**: Validate program semantics, perform type checking, resolve symbols.

**Type-Scoped Static Method Call Syntax**:

When calling static methods (associated functions) on types, Crusty requires the `@` prefix before the type name to distinguish type-scoped calls from instance method calls:

```crusty
// Static method calls (type-scoped) - ALWAYS require @ prefix
let v = @Vector.new();
let none = @Option.None;
let s = @String.from("hello");

// Instance method calls - no @ prefix
let len = v.len();
let item = v.get(0);
```

This syntax makes it immediately clear whether a call is:
- **Type-scoped** (`@Type.method()`): Calling a static method/associated function on the type itself
- **Instance-scoped** (`obj.method()`): Calling a method on an instance

**Dot Notation for Type-Scoped Calls**:

The `@` prefix is **required** for all type-scoped calls. After the `@` prefix and type name, use **dot notation (`.`)** to replace Rust's `::`:

```crusty
// Simple type-scoped calls - dot replaces ::
@Vector.new()                  → Vector::new()
@Option.None                   → Option::None
@String.from("hello")          → String::from("hello")

// Nested type paths - dot replaces ALL :: occurrences
@std.collections.HashMap.new() → std::collections::HashMap::new()
@std.io.Error.last_os_error()  → std::io::Error::last_os_error()
```

**Method Calls on Type-Scoped Values**:

When accessing a constant/associated item and then calling a method on the resulting value, use arrow notation for the method call:

```crusty
// Arrow for method calls on type-scoped values
@Foo.BAR->boo()                → Foo::BAR.boo()
// Where BAR is a constant value, and boo() is a method call on that value
```

The dot notation provides a consistent mapping where `.` in `@Type.path` always replaces `::` in Rust's `Type::path`.

**Macro Invocation Syntax**:

Crusty uses double-underscore naming for macros, WITHOUT the `!` suffix:

```crusty
// Macro invocations with double-underscore naming (no ! in Crusty)
__println__("Hello, world!");
__vec__[1, 2, 3];
__format__("Value: {}", x);
__assert__(x > 0);
__rust__{ /* raw Rust code */ };

// Translates to Rust (removing double-underscores, adding !)
println!("Hello, world!");
vec![1, 2, 3];
format!("Value: {}", x);
assert!(x > 0);
```

**Important**: The `!` suffix is Rust-specific syntax. Crusty macros do NOT use `!` - it is added during transpilation to Rust.

**Distinguishing Type-Scoped Calls from Macros**:

The parser distinguishes between type-scoped static method calls and macro invocations based on syntax:
- **Type-scoped call**: `@Type.method()` - uses `@` prefix with `.` separator
- **Macro invocation**: `__macro_name__(...)` - uses double-underscore prefix/suffix, NO `!`

Examples:
```crusty
@Vec.new()               // Type-scoped call → Vec::new()
__vec__[1, 2, 3]         // Macro invocation → vec![1, 2, 3]
@Option.None             // Type-scoped call → Option::None
__println__("hello")     // Macro invocation → println!("hello")
@String.from("hi")       // Type-scoped call → String::from("hi")
__format__("x={}", x)    // Macro invocation → format!("x={}", x)
@Foo.Bar.boo()           // Nested type path → Foo::Bar.boo()
@Foo.BAR->boo()          // Type-scoped value + method → Foo::BAR.boo()
```

The `@` prefix is exclusively for type-scoped calls, while double-underscores are exclusively for macros, eliminating any ambiguity.

**Nested Type Paths**:

For nested type paths (matching Rust's `Foo::Bar.boo()` pattern), use dot notation after the `@` prefix:
```crusty
@Foo.Bar.boo()                    // Nested type path → Foo::Bar.boo()
@std.collections.HashMap->new()   // Nested with arrow → std::collections::HashMap::new()
```

This allows natural mapping to Rust's nested type paths while maintaining the `@` prefix to distinguish type-scoped from instance-scoped calls.

**Defining Macros with #define**:

Crusty supports defining macros using the `#define` directive, which translates to Rust's `macro_rules!` system:

```crusty
// Simple macro definitions with double-underscore naming
#define __MAX__(a, b) ((a) > (b) ? (a) : (b))
#define __SQUARE__(x) ((x) * (x))
#define __DEBUG_PRINT__(msg) __println__("DEBUG: {}", msg)

// Translates to Rust macro_rules! (removing double-underscores)
macro_rules! max {
    ($a:expr, $b:expr) => {
        if $a > $b { $a } else { $b }
    };
}

macro_rules! square {
    ($x:expr) => {
        ($x) * ($x)
    };
}

macro_rules! debug_print {
    ($msg:expr) => {
        println!("DEBUG: {}", $msg)
    };
}
```

**#define Parsing Rules**:
- The parser recognizes `#define __MACRO_NAME__(params) body` syntax with double-underscore prefix and suffix
- Macro names MUST have double-underscores as prefix and suffix
- Macro parameters are parsed as identifiers
- The macro body is parsed as a token sequence
- When generating Rust, double-underscores are removed and name is converted to snake_case, and `!` is added
- Macro invocations within the body are also translated (double-underscores removed, `!` added)
- The parser does not perform full semantic analysis on macro bodies

**Reserved Pattern for Macros**:
- The double-underscore pattern (leading AND trailing) is **reserved exclusively for macros**
- Functions CANNOT use leading and trailing double-underscores
- The Semantic_Analyzer SHALL detect and reject function definitions with this pattern
- Error message: "Function names cannot use double-underscore pattern (reserved for macros)"

**Valid function names**:
```crusty
void helper() { }           // OK
void _private_helper() { }  // OK - single leading underscore
```

**Invalid function names**:
```crusty
void __helper__() { }       // ERROR - reserved for macros
int __compute__() { }       // ERROR - reserved for macros
```

**#define to macro_rules! Translation**:
- Macro name is preserved
- Parameters become pattern variables (`$param:expr`)
- The body is wrapped in appropriate Rust macro syntax
- Ternary operators are translated to if-else expressions
- Macro invocations (`macro!`) are passed through unchanged to Rust

**Label Syntax for Loops**:

Crusty uses dot-prefixed labels (`.label:`) for labeled loops, mimicking C/ASM identifier syntax:

```crusty
// Crusty label syntax - dot prefix for declaration, no dot in break/continue
.outer: loop {
    .inner: loop {
        if (condition) break outer;
        continue inner;
    }
}

// Translates to Rust
'outer: loop {
    'inner: loop {
        if condition { break 'outer; }
        continue 'inner;
    }
}
```

**Label Translation Rules**:
- `.label:` in Crusty → `'label:` in Rust (loop labels)
- `break label` in Crusty → `break 'label` in Rust
- `continue label` in Crusty → `continue 'label` in Rust
- Labels must be identifiers following the dot
- Labels are scoped to the enclosing function

**Important**: The dot (`.`) is a prefix for label declarations only, mimicking C/ASM identifier syntax. It is NOT part of the label name itself. When using `break` or `continue`, reference the label without the dot.

**Explicit Generic Type Parameters with Parentheses and Brackets**:

Crusty provides a C-like syntax for explicit generic type parameters using parentheses and brackets that alternate for nested generics. The `@` prefix is **required** for all type-scoped calls:

```crusty
// Type-scoped calls ALWAYS require @ prefix with -> notation
let opt = @Option->None;          // Type inferred from context
let v = @Vec->new();               // Type inferred from usage

// Explicit type parameters with required @ prefix
let opt = @Option(Result[String, std.io.Error])->None;
let v = @Vec(i32)->new();

// Deep nesting with alternating parentheses and brackets
let complex = @Option(Inner[Type(T), std.io.Error])->None;
// Translates to: Option::<Inner<Type<T>, std::io::Error>>::None

// @ is ALWAYS required for type-scoped calls
// Vec(i32).new()  is INVALID - missing @
// Vec.new()       is INVALID - missing @
// Option.None     is INVALID - missing @

// The parentheses and everything within can be dropped if types can be inferred:
// @Option(Result[String, std.io.Error]).None  can become  @Option.None
// @Vec(i32).new()  can become  @Vec.new()

// Rust equivalent translations:
// @Option(Result[String, std.io.Error]).None → Option::<Result<String, std::io::Error>>::None
// @Vec(i32).new() → Vec::<i32>::new()
// @Option(Inner[Type(T), std.io.Error]).None → Option::<Inner<Type<T>, std::io::Error>>::None
// @Option.None → Option::None (no turbofish, relies on inference)
// @Vec.new() → Vec::new() (no turbofish, relies on inference)
// @Type.new() → Type::new() (type-scoped call, no turbofish)
```

**Syntax Rules**:
- The `@` prefix is **required** for all type-scoped calls
- Use parentheses `()` for outermost generic parameters
- Use brackets `[]` for first level of nesting
- Continue alternating `()` and `[]` for deeper nesting (arbitrary depth supported)
- Omit generic parameters entirely (parentheses) when types can be fully inferred
- Type-scoped calls without `@` are syntax errors

**Translation to Rust**:
- `@Type(T)` → `Type::<T>`
- `@Type(Inner[T])` → `Type::<Inner<T>>`
- `@Type(A[B(C)])` → `Type::<A<B<C>>>`
- `@Type(A[B(C[D])])` → `Type::<A<B<C<D>>>>`
- `@Type` → `Type` (no turbofish, rely on inference)
- `@Type.method()` → `Type::method()` (type-scoped call, no turbofish)

**Implementation Blocks with typedef Syntax**:

Crusty uses C-style `typedef` syntax for defining structs and their implementations, providing a familiar syntax while mapping to Rust's `impl` blocks:

**Basic Struct Definition:**
```crusty
// Define a struct type
typedef struct {
    int x;
    int y;
} Point;

// Translates to Rust:
struct Point {
    x: i32,
    y: i32,
}
```

**Implementation Blocks (@Type):**
```crusty
// Add methods to an existing type
typedef struct {
    Point new(int x, int y) {
        return Point { x: x, y: y };
    }
    
    int distance_squared(&self) {
        return self.x * self.x + self.y * self.y;
    }
} @Point;

// Translates to Rust:
impl Point {
    pub fn new(x: i32, y: i32) -> Point {
        return Point { x: x, y: y };
    }
    
    pub fn distance_squared(&self) -> i32 {
        return self.x * self.x + self.y * self.y;
    }
}
```

**Default Trait Implementation (typedef default):**
```crusty
// Implement Default trait
typedef default {
    Point default() {
        return Point { x: 0, y: 0 };
    }
} @Point;

// Translates to Rust:
impl Default for Point {
    fn default() -> Self {
        return Point { x: 0, y: 0 };
    }
}
```

**Named Implementation Blocks (@Type->name):**
```crusty
// Named impl block for organization
typedef struct {
    void print(&self) {
        __println__("Point({}, {})", self.x, self.y);
    }
} @Point->display;

// Another named impl block
typedef struct {
    void debug(&self) {
        __println__("Point {{ x: {}, y: {} }}", self.x, self.y);
    }
} @Point->debug;

// Both translate to Rust (merged into single impl):
impl Point {
    pub fn print(&self) {
        println!("Point({}, {})", self.x, self.y);
    }
    
    pub fn debug(&self) {
        println!("Point {{ x: {}, y: {} }}", self.x, self.y);
    }
}
```

**Syntax Rules**:
- `typedef struct { ... } Type;` - Define a new struct type
- `typedef struct { methods } @Type;` - Add impl block for existing type
- `typedef default { fn default() { ... } } @Type;` - Implement Default trait
- `typedef struct { methods } @Type->name;` - Named impl block (for organization)
- The `@` prefix indicates the type already exists
- The `->name` suffix is optional and used for organizing multiple impl blocks
- All named impl blocks for the same type are merged in the generated Rust code

**Translation Rules**:
- `typedef struct @Type` → `impl Type`
- `typedef default @Type` → `impl Default for Type`
- `typedef struct @Type->name` → `impl Type` (name is for organization only)
- Multiple `@Type->name` blocks are merged into a single `impl Type` block

**Interface**:
```rust
pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
    type_env: TypeEnvironment,
    errors: Vec<SemanticError>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self;
    pub fn analyze(&mut self, file: &File) -> Result<(), Vec<SemanticError>>;
    
    // Internal analysis methods
    fn analyze_item(&mut self, item: &Item);
    fn analyze_function(&mut self, func: &Function);
    fn analyze_statement(&mut self, stmt: &Statement);
    fn analyze_expression(&mut self, expr: &Expression) -> Type;
    fn check_type_compatibility(&self, expected: &Type, actual: &Type) -> bool;
    fn resolve_symbol(&self, name: &Ident) -> Option<Symbol>;
}

pub struct SymbolTable {
    scopes: Vec<Scope>,
}

pub struct Scope {
    symbols: HashMap<String, Symbol>,
}

pub struct Symbol {
    pub name: String,
    pub ty: Type,
    pub kind: SymbolKind,
    pub mutable: bool,
}

pub enum SymbolKind {
    Variable,
    Function,
    Type,
    Const,
}

pub struct TypeEnvironment {
    types: HashMap<String, TypeInfo>,
}
```

### 7. Code Generator Module

**Responsibility**: Generate target language source code from AST.

**Interface**:
```rust
pub struct CodeGenerator {
    target: TargetLanguage,
    indent_level: usize,
}

pub enum TargetLanguage {
    Crusty,
    Rust,
}

impl CodeGenerator {
    pub fn new(target: TargetLanguage) -> Self;
    pub fn generate(&mut self, file: &File) -> String;
    
    // Internal generation methods
    fn generate_item(&mut self, item: &Item) -> String;
    fn generate_function(&mut self, func: &Function) -> String;
    fn generate_struct(&mut self, struct_def: &Struct) -> String;
    fn generate_statement(&mut self, stmt: &Statement) -> String;
    fn generate_expression(&mut self, expr: &Expression) -> String;
    fn generate_type(&mut self, ty: &Type) -> String;
}
```

### 8. Pretty Printer Module

**Responsibility**: Format generated code according to language conventions.

**Interface**:
```rust
pub struct PrettyPrinter {
    language: TargetLanguage,
}

impl PrettyPrinter {
    pub fn new(language: TargetLanguage) -> Self;
    pub fn format(&self, code: &str) -> String;
}
```

For Rust code, we'll use the `prettyplease` crate which formats syn AST into well-formatted Rust source code.

### 9. Documentation Generation Module (crustydoc)

**Responsibility**: Generate documentation by transpiling Crusty code and invoking rustdoc.

**Architecture**:

The crustydoc tool is a thin wrapper that:
1. Transpiles Crusty source files to Rust
2. Invokes rustdoc on the generated Rust code
3. Maps any errors back to Crusty source locations

This approach provides several benefits:
- **Zero maintenance**: rustdoc is maintained by the Rust team
- **Full feature parity**: All rustdoc features work automatically (search, cross-references, examples, etc.)
- **Consistent output**: Documentation looks identical to Rust documentation
- **Cargo integration**: Works seamlessly with Cargo's doc generation

**Interface**:
```rust
pub struct CrustyDoc {
    transpiler: Transpiler,
    rustdoc_path: PathBuf,
}

impl CrustyDoc {
    pub fn new() -> Self;
    
    /// Generate documentation for a single Crusty file
    pub fn document_file(&self, input: &Path, output_dir: &Path, options: &DocOptions) -> Result<(), DocError>;
    
    /// Generate documentation for a Crusty project (via Cargo)
    pub fn document_project(&self, manifest_path: &Path, options: &DocOptions) -> Result<(), DocError>;
    
    /// Map rustdoc error locations back to Crusty source
    fn map_error_location(&self, rust_error: &RustdocError) -> CrustyError;
}

pub struct DocOptions {
    pub open_browser: bool,
    pub document_private: bool,
    pub deny_missing_docs: bool,
    pub extra_args: Vec<String>,
}

pub enum DocError {
    Transpilation(CompilerError),
    Rustdoc(RustdocError),
    Io(std::io::Error),
}
```

**CLI Interface**:
```bash
# Generate documentation for a single file
crustydoc src/lib.crst --output target/doc

# Generate documentation for entire project (via Cargo)
crustydoc --manifest-path Cargo.toml

# Open documentation in browser after generation
crustydoc src/lib.crst --open

# Treat missing documentation as errors
crustydoc src/lib.crst -D missing-docs

# Document private items
crustydoc src/lib.crst --document-private-items

# Pass additional rustdoc options
crustydoc src/lib.crst -- --html-in-header header.html
```

**Cargo Integration**:

For Crusty projects using Cargo, documentation generation works automatically:

```toml
# Cargo.toml
[package]
name = "my-crusty-lib"
version = "0.1.0"

[build-dependencies]
crustyc = "0.1"
```

```rust
// build.rs
fn main() {
    // Transpile all .crst files to OUT_DIR
    crustyc::transpile_all("src", env::var("OUT_DIR").unwrap());
}
```

Then run:
```bash
cargo doc --open
```

Cargo will:
1. Run build.rs to transpile .crst files
2. Run rustdoc on the generated Rust code
3. Open the documentation in a browser

**Error Mapping**:

When rustdoc reports errors, crustydoc maps them back to Crusty source:

```
error: missing documentation for a function
  --> src/utils.crst:42:1
   |
42 | int calculate(int x, int y) {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: requested on the command line with `-D missing-docs`
```

### 10. Code Formatter Module (crustyfmt)

**Responsibility**: Format Crusty source code according to consistent style conventions.

**Interface**:
```rust
pub struct CrustyFormatter {
    config: FormatterConfig,
}

pub struct FormatterConfig {
    pub indent_size: usize,
    pub max_line_width: usize,
    pub space_before_brace: bool,
    pub space_after_comma: bool,
}

impl CrustyFormatter {
    pub fn new(config: FormatterConfig) -> Self;
    pub fn format_file(&self, path: &Path) -> Result<String, FormatterError>;
    pub fn format_source(&self, source: &str) -> Result<String, FormatterError>;
    pub fn check_file(&self, path: &Path) -> Result<bool, FormatterError>;
    pub fn format_files(&self, paths: Vec<PathBuf>) -> Result<(), FormatterError>;
}

pub enum FormatterError {
    Parse(ParseError),
    Io(std::io::Error),
    InvalidConfig(String),
}
```

**Formatting Rules**:
- **Indentation**: 4 spaces per level (no tabs)
- **Line Width**: Maximum 100 characters (configurable)
- **Braces**: Opening brace on same line as declaration
- **Spacing**: Space after commas, around binary operators
- **Blank Lines**: One blank line between top-level items
- **Comments**: Preserve position and content of all comments
- **Alignment**: Align struct field types and values in initializers

**CLI Interface**:
```bash
# Format a single file (in-place)
crustyfmt src/main.crst

# Check formatting without modifying
crustyfmt --check src/main.crst

# Format all .crst files in directory
crustyfmt src/

# Format from stdin to stdout (for editor integration)
crustyfmt --stdin < input.crst > output.crst

# Use custom config file
crustyfmt --config crustyfmt.toml src/
```

**Integration Points**:
- **Pre-commit hooks**: Automatically format staged .crst files
- **CI/CD**: Verify formatting in pull requests
- **Editor plugins**: Format on save via stdin/stdout mode
- **Build scripts**: Format generated code

### 11. Error Handling

**Error Types**:
```rust
pub enum CompilerError {
    Lex(LexError),
    Parse(ParseError),
    Semantic(Vec<SemanticError>),
    CodeGen(CodeGenError),
    Io(std::io::Error),
    RustcInvocation(String),
}

pub struct LexError {
    pub span: Span,
    pub message: String,
}

pub struct ParseError {
    pub span: Span,
    pub message: String,
    pub expected: Vec<TokenKind>,
    pub found: TokenKind,
}

pub struct SemanticError {
    pub span: Span,
    pub kind: SemanticErrorKind,
    pub message: String,
}

pub enum SemanticErrorKind {
    UndefinedVariable,
    TypeMismatch,
    DuplicateDefinition,
    InvalidOperation,
    UnsupportedFeature,
}
```

## Data Models

### Build System Integration

The transpiler supports integration with Rust's build system through build.rs scripts and multi-file project handling.

**build.rs Integration Architecture:**

The transpiler provides a `--out-dir` option that allows build.rs scripts to specify where generated Rust files should be placed. This enables seamless integration with Cargo's build process:

```rust
// Example build.rs script
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    
    // Discover all .crst files in src/
    let crst_files = discover_crst_files(&manifest_dir);
    
    // Transpile each file to OUT_DIR
    for crst_file in crst_files {
        let status = Command::new("crustyc")
            .arg(&crst_file)
            .arg("--out-dir")
            .arg(&out_dir)
            .arg("--no-compile")
            .status()
            .expect("Failed to run crustyc");
        
        if !status.success() {
            panic!("Failed to transpile {}", crst_file.display());
        }
        
        // Tell Cargo to rerun if this file changes
        println!("cargo:rerun-if-changed={}", crst_file.display());
    }
}
```

**Multi-File Project Structure:**

For projects with multiple Crusty source files, the transpiler:
1. Preserves directory structure in the output directory
2. Resolves `#use` directives to locate local modules
3. Builds a module dependency graph
4. Transpiles files in dependency order

**Module Resolution:**

When encountering a `#use` directive for a local module:
```crusty
#use crate.utils.helpers;
```

The transpiler:
1. Resolves the module path to a file: `src/utils/helpers.crst`
2. Parses the referenced file if not already parsed
3. Resolves symbols across module boundaries
4. Generates appropriate Rust `use` statements

**Batch Transpilation:**

The transpiler supports batch mode for transpiling multiple files:
```bash
crustyc --out-dir target/generated src/**/*.crst
```

This discovers all `.crst` files in the source directory and transpiles them to the output directory, preserving the directory structure.

### Example Directory Architecture

The project includes an `example/` directory demonstrating Crusty usage and serving as integration tests.

**Directory Structure:**
```
example/
├── Cargo.toml          # Example project manifest
├── build.rs            # Build script that invokes crustyc
├── README.md           # Build and run instructions
└── src/
    ├── main.crst       # Hello world example
    ├── functions.crst  # Function examples
    ├── structs.crst    # Struct and method examples
    ├── methods.crst    # Struct method examples
    ├── generics.crst   # Generic type parameter examples
    ├── attributes.crst # Attribute examples
    ├── macros.crst     # Macro usage examples
    ├── ranges.crst     # Range syntax examples
    └── slices.crst     # Slice examples
```

**Example Cargo.toml:**
```toml
[package]
name = "crusty-example"
version = "0.1.0"
edition = "2021"

[build-dependencies]
# Reference crustyc from parent directory
crustyc = { path = ".." }
```

**Example build.rs:**
```rust
// Discovers all .crst files and transpiles them
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let src_dir = Path::new("src");
    
    // Discover all .crst files
    for entry in fs::read_dir(src_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("crst") {
            // Transpile to OUT_DIR
            let status = Command::new("crustyc")
                .arg(&path)
                .arg("--out-dir")
                .arg(&out_dir)
                .arg("--no-compile")
                .status()
                .expect("Failed to run crustyc");
            
            if !status.success() {
                panic!("Failed to transpile {:?}", path);
            }
            
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}
```

**CI/CD Integration:**

The example directory is built and tested in the CI/CD pipeline:
```yaml
- name: Build examples
  run: |
    cd example
    cargo build --verbose
    cargo run
```

This ensures that:
1. The example project builds successfully
2. All Crusty syntax features work correctly
3. Generated Rust code compiles
4. The example binary runs without errors

### Rust Ecosystem Integration

Crusty programs can seamlessly integrate with the Rust ecosystem, using external crates and publishing their own crates.

**Using External Crates:**

Crusty code can import and use types from external Rust crates:

```crusty
// Import external crate types
#use serde.Serialize;
#use serde.Deserialize;
#use tokio.runtime.Runtime;

// Use external types in Crusty code
#[derive(Serialize, Deserialize)]
struct User {
    name: char*,
    age: i32,
}

void process_user(User* user) {
    // Use external crate functions
    let json = @serde_json->to_string(user)!;
    __println__("{}", json);
}
```

**Type Compatibility:**

The transpiler ensures type compatibility between Crusty and external Rust types:
- Crusty structs can implement external traits
- External types can be used in Crusty function signatures
- Generic types from external crates work correctly

**Publishing Crusty Crates:**

Crusty libraries can be published as Rust crates:

1. **Library Structure:**
```
my-crusty-lib/
├── Cargo.toml
├── build.rs
└── src/
    ├── lib.crst
    └── utils.crst
```

2. **Cargo.toml:**
```toml
[package]
name = "my-crusty-lib"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["rlib"]

[build-dependencies]
crustyc = "0.1"
```

3. **build.rs:**
```rust
// Transpile all .crst files to OUT_DIR
// (same pattern as example directory)
```

4. **Publishing:**
```bash
cargo build --release
cargo publish
```

**Consuming Crusty Libraries from Rust:**

Rust projects can depend on Crusty libraries:

```toml
[dependencies]
my-crusty-lib = "0.1"
```

```rust
// Rust code using Crusty library
use my_crusty_lib::User;

fn main() {
    let user = User::new("Alice", 30);
    user.process();
}
```

**API Compatibility:**

The transpiler ensures that Crusty libraries expose Rust-compatible APIs:
- Public functions become `pub fn`
- Public structs become `pub struct`
- Type signatures are Rust-compatible
- Documentation comments are preserved

**Performance Parity:**

Crusty code compiles to the same Rust code that a human would write, ensuring:
- No runtime overhead
- Same optimization opportunities
- Identical performance characteristics
- Zero-cost abstractions

### Symbol Table

The symbol table tracks all declared symbols (variables, functions, types) and their scopes. It uses a stack of scopes to handle nested blocks.

```rust
pub struct SymbolTable {
    scopes: Vec<Scope>,
}

impl SymbolTable {
    pub fn enter_scope(&mut self);
    pub fn exit_scope(&mut self);
    pub fn insert(&mut self, name: String, symbol: Symbol) -> Result<(), String>;
    pub fn lookup(&self, name: &str) -> Option<&Symbol>;
    pub fn lookup_in_current_scope(&self, name: &str) -> Option<&Symbol>;
}
```

### Type Environment

The type environment tracks type definitions and provides type checking utilities.

```rust
pub struct TypeEnvironment {
    types: HashMap<String, TypeInfo>,
}

pub struct TypeInfo {
    pub name: String,
    pub kind: TypeKind,
}

pub enum TypeKind {
    Primitive,
    Struct { fields: Vec<(String, Type)> },
    Enum { variants: Vec<String> },
    Alias { target: Type },
}

impl TypeEnvironment {
    pub fn register_type(&mut self, name: String, info: TypeInfo);
    pub fn get_type(&self, name: &str) -> Option<&TypeInfo>;
    pub fn is_compatible(&self, t1: &Type, t2: &Type) -> bool;
}
```



## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*


### Core Parsing Properties

Property 1: Valid Crusty programs parse successfully
*For any* syntactically valid Crusty source file, the Parser should successfully parse it into a complete AST without errors.
**Validates: Requirements 6.1**

Property 2: Invalid syntax produces error reports with location
*For any* Crusty source file with syntax errors, the Parser should report each error with its line number, column number, and a descriptive error message.
**Validates: Requirements 6.2, 10.1**

Property 3: Multiple errors are all reported
*For any* Crusty source file containing multiple syntax or semantic errors, the compiler should report all errors found, not just the first one.
**Validates: Requirements 10.4**

### Code Generation Properties

Property 4: Generated Rust code is syntactically valid
*For any* valid AST, the Code_Generator should produce Rust source code that can be successfully parsed by the Rust parser (syn or rustc) without syntax errors.
**Validates: Requirements 8.1**

Property 5: Generated Rust code follows formatting conventions
*For any* generated Rust source code, running rustfmt on it should produce no changes, indicating it already follows Rust style conventions.
**Validates: Requirements 8.16**

Property 6: Transparent syntax preservation
*For any* AST node containing Rust-compatible syntax (tuples, array literals, macro invocations, attributes, ranges), the generated Rust code should preserve that syntax exactly as it appears in the AST.
**Validates: Requirements 19.7, 20.4, 23.6, 25.8, 26.8**

### Translation Properties

Property 7: Variable declarations translate correctly
*For any* variable declaration in the AST (const, var, let, static), the generated Rust code should use the corresponding Rust keyword (const, let mut, let, static) with correct semantics.
**Validates: Requirements 35.7, 35.8, 35.9**

Property 8: Reference syntax translates correctly
*For any* reference expression in the AST (& for immutable, &var or &mut for mutable), the generated Rust code should use the corresponding Rust reference syntax (& or &mut).
**Validates: Requirements 36.10, 36.11**

Property 9: Type casts translate to 'as' operator
*For any* C-style cast expression ((type)expr) in the AST, the generated Rust code should use the 'as' operator (expr as type).
**Validates: Requirements 27.5**

Property 10: Sizeof translates to std::mem functions
*For any* sizeof expression in the AST, the generated Rust code should use std::mem::size_of<T>() for types or std::mem::size_of_val(&expr) for expressions.
**Validates: Requirements 28.6**

Property 11: Increment/decrement operators translate with correct semantics
*For any* prefix increment (++i) in the AST, the generated Rust code should evaluate to the incremented value; for any postfix increment (i++), the generated Rust code should evaluate to the original value before incrementing.
**Validates: Requirements 29.10, 29.11**

Property 12: Typedef translates to type alias
*For any* typedef declaration in the AST, the generated Rust code should create a corresponding type alias using the 'type' keyword.
**Validates: Requirements 31.9**

Property 13: C-style enums translate to Rust enums with discriminants
*For any* C-style enum declaration in the AST, the generated Rust code should create a Rust enum with explicit integer discriminants matching the C-style values.
**Validates: Requirements 32.8**

Property 14: NULL translates to Option types
*For any* NULL literal in the AST, the generated Rust code should use @Option->None (translating to Option::None); for any nullable pointer type, the generated Rust code should use Option<&T> or Option<&mut T>.
**Validates: Requirements 34.4, 34.5**

Property 15: Struct initializers translate to Rust struct literals
*For any* struct initialization with designated initializers (.field = value) in the AST, the generated Rust code should use Rust struct literal syntax (StructName { field: value }).
**Validates: Requirements 39.6**

Property 16: Struct methods translate to impl blocks
*For any* struct with methods in the AST, the generated Rust code should create a corresponding impl block containing all methods. Static method calls using @Type->method() syntax should translate to Rust Type::method() syntax.
**Validates: Requirements 21.9**

Property 17: VTable structs translate to traits
*For any* struct following the vtable pattern (function pointers with self parameter) in the AST, the generated Rust code should create a corresponding trait definition.
**Validates: Requirements 22.6**

Property 18: For loops translate appropriately
*For any* C-style for loop in the AST matching the pattern for(i=start; i<end; i++), the generated Rust code should use range syntax (for i in start..end); for multi-variable for loops, the generated Rust code should use a scoped while loop.
**Validates: Requirements 38.4, 38.5, 38.7**

Property 19: Switch statements translate to match expressions
*For any* switch statement in the AST, the generated Rust code should create a corresponding match expression with all cases and default branch.
**Validates: Requirements 45.7**

Property 20: Error handling syntax translates correctly
*For any* fallible return type (Type!) in the AST, the generated Rust code should use Result<Type, E>; for any error(value) expression, the generated Rust code should use Err(value); for any error propagation operator (!), the generated Rust code should use the ? operator.
**Validates: Requirements 46.8, 46.9, 46.10**

Property 21: Module directives translate correctly
*For any* #use directive in the AST, the generated Rust code should create a corresponding use statement; for any namespace declaration, the generated Rust code should create a corresponding mod block.
**Validates: Requirements 47.3, 48.5**

Property 22: #define macros translate to macro_rules!
*For any* #define macro definition in the AST, the generated Rust code should create a corresponding macro_rules! definition with parameters translated to pattern variables and the body wrapped in appropriate Rust macro syntax.
**Validates: Requirements 24.7, 24.8, 24.9**

Property 23: Label syntax translates correctly
*For any* labeled loop (.label: loop), break statement (break label), or continue statement (continue label) in the AST, the generated Rust code should use Rust's label syntax ('label:, break 'label, continue 'label). Note: The dot is a prefix for label declarations only, not part of the label name.
**Validates: Requirements 6.13, 6.14, 6.15**

Property 24: Explicit generic parameters translate correctly
*For any* explicit generic type parameter specification using parentheses/brackets syntax in the AST, the generated Rust code should use Rust's turbofish syntax with angle brackets. Nested generics should correctly alternate between parentheses and brackets in Crusty and translate to nested angle brackets in Rust.
**Validates: Requirements 38.18, 38.19, 38.20, 38.21**

### Bidirectional Transpilation Properties

Property 25: Rust to Crusty translation preserves structure
*For any* Rust source file, parsing it and generating Crusty code should produce valid Crusty syntax that preserves the program structure (functions become functions, match becomes switch, etc.).
**Validates: Requirements 47.5, 47.8**

Property 26: Round-trip transpilation preserves semantics (CRITICAL)
*For any* valid Crusty source file, transpiling to Rust and then back to Crusty should produce a program that is semantically equivalent to the original (same AST structure after normalization).
**Validates: Requirements 54.20**

### Parsing Round-Trip Property

Property 27: Pretty-print then parse is identity (CRITICAL)
*For any* valid AST, pretty-printing it to Crusty source code and then parsing that source code should produce an AST that is equivalent to the original (modulo formatting differences).
**Validates: Requirements 16.1, 16.2**

### Type System Properties

Property 28: Type checking matches Rust semantics
*For any* type operation in a valid program (assignment, function call, operator application), the Semantic_Analyzer should accept it if and only if Rust's type system would accept it.
**Validates: Requirements 18.9**

Property 29: Valid file paths are read successfully
*For any* valid file path provided to crustyc, the file contents should be successfully read into memory.
**Validates: Requirements 11.1**

Property 30: Example directory builds successfully
*For any* valid example project in the example/ directory, running `cargo build` should succeed without errors, and running `cargo run` should execute the example binary successfully.
**Validates: Requirements 6.1-6.34**

Property 31: Rust ecosystem integration works correctly
*For any* Crusty project using external Rust crates, the transpiled code should compile and link correctly, with full type compatibility and API access to external types and functions.
**Validates: Requirements 40.1-40.15**

Property 32: Function names with double-underscore pattern are rejected
*For any* function definition with both leading AND trailing double-underscores (e.g., `void __helper__()`), the Semantic_Analyzer should report an error indicating that this pattern is reserved for macros.
**Validates: Requirements 25.10, 25.11**

Property 33: crustyfmt preserves semantic meaning
*For any* valid Crusty source file, formatting it with crustyfmt and then parsing both the original and formatted versions should produce semantically equivalent ASTs (modulo whitespace and formatting differences).
**Validates: Requirements 56.10**

Property 34: crustyfmt is idempotent
*For any* valid Crusty source file, formatting it with crustyfmt multiple times should produce identical output after the first formatting pass.
**Validates: Requirements 56.1-56.20**

## Error Handling

### Error Reporting Strategy

All errors should be reported with:
- **Location**: Line and column number in source file
- **Context**: Snippet of source code showing the error
- **Description**: Clear explanation of what went wrong
- **Suggestion**: When possible, suggest how to fix the error

### Error Categories

1. **Lexical Errors**: Invalid characters, unterminated strings, malformed numbers
2. **Syntax Errors**: Unexpected tokens, missing delimiters, invalid grammar
3. **Semantic Errors**: Type mismatches, undefined variables, invalid operations
4. **Unsupported Feature Errors**: C features that cannot be translated to Rust
5. **File I/O Errors**: Missing files, permission errors, write failures
6. **Rustc Errors**: Compilation failures in generated Rust code

### Unsupported C Features

The compiler should detect and reject these C features with clear error messages:

- **C unions**: Not supported (Rust doesn't have unions in the same way)
- **goto statements**: Not supported (use Rust's labeled break/continue instead)
- **#include directives**: Not supported (use #use instead)
- **Arbitrary pointer arithmetic**: Only safe pointer operations allowed
- **Implicit type conversions**: Explicit casts required

**Supported with Translation**:
- **#define macros**: Supported for macro definitions, translate to Rust macro_rules!

**Supported Natively (Pass-through to Rust)**:
- **Labeled break/continue**: Crusty supports labeled break and continue with `.label:` syntax (translates to Rust's `'label:`)
- **Labeled loops**: Crusty supports labeled loops with `.label: loop { ... }` syntax (translates to Rust's `'label: loop { ... }`)

For each unsupported feature, the error message should:
1. Identify the feature
2. Explain why it's not supported
3. Suggest an alternative approach

### Error Recovery

The parser should implement error recovery to continue parsing after errors, allowing multiple errors to be reported in a single compilation run. This is achieved by:

1. **Synchronization points**: Recover at statement boundaries (semicolons, braces)
2. **Panic mode**: Skip tokens until a synchronization point is found
3. **Error productions**: Add grammar rules for common mistakes

## Testing Strategy

### Dual Testing Approach

The test suite will use both unit tests and property-based tests:

**Unit Tests**:
- Specific examples of valid and invalid programs
- Edge cases (empty files, single statements, deeply nested structures)
- Error conditions (missing files, invalid syntax, type errors)
- Integration points (CLI argument parsing, file I/O, rustc invocation)

**Property-Based Tests**:
- Universal properties that hold for all inputs
- Comprehensive input coverage through randomization
- Each property test runs minimum 100 iterations
- Tests are tagged with references to design properties

### Property-Based Testing Configuration

We'll use the `proptest` crate for Rust property-based testing. Each property test will:

1. Generate random valid AST nodes or source code
2. Apply the operation being tested (parse, generate, analyze)
3. Assert the property holds
4. Run for at least 100 iterations

Example test structure:
```rust
#[test]
fn test_property_25_pretty_print_parse_roundtrip() {
    // Feature: crusty-compiler-phase1, Property 25: Pretty-print then parse is identity
    proptest!(|(ast: ValidAst)| {
        let crusty_code = pretty_print(&ast);
        let parsed_ast = parse(&crusty_code)?;
        assert_ast_equivalent(&ast, &parsed_ast);
    });
}
```

### Test Categories

1. **Parser Tests**
   - Valid syntax parsing (Property 1)
   - Error reporting (Properties 2, 3)
   - All language features (functions, structs, enums, statements, expressions)

2. **Code Generation Tests**
   - Rust code validity (Property 4)
   - Formatting (Property 5)
   - All translation properties (Properties 6-23)

3. **Semantic Analysis Tests**
   - Type checking (Property 27)
   - Symbol resolution
   - Scope handling

4. **Round-Trip Tests** (CRITICAL)
   - Crusty → AST → Crusty (Property 26)
   - Crusty → Rust → Crusty (Property 25)
   - Rust → AST → Rust

5. **Integration Tests**
   - CLI argument parsing
   - File I/O (Property 28)
   - rustc invocation
   - Multi-file projects
   - build.rs integration

6. **Error Handling Tests**
   - All error categories
   - Error message quality
   - Multiple error reporting

### Test Data Generation

For property-based tests, we'll implement generators for:

- **Valid AST nodes**: Functions, structs, enums, statements, expressions
- **Valid Crusty source code**: Using grammar-based generation
- **Invalid syntax**: Mutations of valid programs
- **Type-correct expressions**: Respecting type constraints
- **Type-incorrect expressions**: For negative testing

### Coverage Goals

- **Line coverage**: Minimum 90% for Parser, Semantic_Analyzer, Code_Generator
- **Branch coverage**: Minimum 85% for all modules
- **Property coverage**: All 28 correctness properties must have corresponding tests

### Continuous Testing

- Run unit tests on every commit
- Run property tests (100 iterations) on every commit
- Run extended property tests (1000 iterations) nightly
- Run integration tests with real Rust compilation on every PR

## Implementation Notes

### Technology Stack

- **Language**: Rust (for implementing the transpiler itself)
- **Parsing**: Custom recursive descent parser for Crusty, `syn` crate for Rust
- **Pretty Printing**: `prettyplease` crate for Rust code formatting
- **Testing**: `proptest` crate for property-based testing
- **CLI**: `clap` crate for command-line argument parsing
- **Error Reporting**: `codespan-reporting` crate for beautiful error messages

### Development Phases

**Phase 1a: Core Infrastructure**
- Lexer and token types
- Basic parser for minimal Crusty subset
- AST data structures
- Simple code generator (Crusty → Rust)

**Phase 1b: Complete Parser**
- All Crusty syntax features
- Error recovery
- Documentation comment extraction

**Phase 1c: Semantic Analysis**
- Symbol table
- Type checking
- Scope resolution

**Phase 1d: Bidirectional Transpilation**
- Rust parser integration (syn)
- Rust → Crusty code generation
- Round-trip validation

**Phase 1e: Advanced Features**
- Multi-file projects
- build.rs integration
- crustydoc tool

**Phase 1f: Polish**
- Error message improvements
- Performance optimization
- Documentation

### Key Design Decisions

1. **Unified AST**: Using a single AST representation for both Crusty and Rust simplifies bidirectional transpilation and ensures consistency.

2. **No Crusty Standard Library**: Crusty programs use Rust's std library directly, avoiding the need to maintain a separate standard library and ensuring full compatibility.

3. **Escape Hatch (rust! macro)**: Providing a way to embed raw Rust code allows users to access Rust features not yet supported by Crusty syntax.

4. **Conservative Feature Set**: Only supporting C features that map cleanly to Rust ensures safety and maintainability.

5. **Property-Based Testing**: Using property-based tests for round-trip validation catches edge cases that unit tests might miss.

6. **Explicit Mutability**: Using `var` for mutable variables makes mutability explicit and maps directly to Rust's `let mut`.

7. **Lifetime Inference**: Inferring lifetimes from reference patterns in function signatures reduces annotation burden while maintaining safety.

### Performance Considerations

- **Incremental Parsing**: For large files, consider parsing incrementally
- **Parallel Compilation**: For multi-file projects, parse files in parallel
- **AST Caching**: Cache parsed ASTs to avoid re-parsing unchanged files
- **Lazy Code Generation**: Only generate code for modules that have changed

### Future Extensions (Post-Phase 1)

- **Macro System**: Define Crusty-specific macros that expand to Rust macros
- **Generic Functions**: Support generic function definitions in Crusty syntax
- **Trait Definitions**: More ergonomic trait syntax beyond vtable structs
- **Pattern Matching**: Direct pattern matching syntax (not just switch)
- **Async/Await**: Support for async functions and await expressions
- **Procedural Macros**: Crusty syntax for defining procedural macros
- **IDE Integration**: Language server protocol (LSP) support
- **Debugger Integration**: Map Crusty source locations to Rust for debugging
