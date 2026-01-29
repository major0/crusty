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
    pub attributes: Vec<Attribute>,
}

/// Struct definition
#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub visibility: Visibility,
    pub name: Ident,
    pub fields: Vec<Field>,
    pub methods: Vec<Function>,
    pub doc_comments: Vec<String>,
    pub attributes: Vec<Attribute>,
}

/// Enum definition
#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub visibility: Visibility,
    pub name: Ident,
    pub variants: Vec<EnumVariant>,
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
    pub body: Vec<crate::lexer::Token>,
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
#[allow(dead_code)]
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
    Null,
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
            visibility: Visibility::Public,
            name: Ident::new("main"),
            params: vec![],
            return_type: None,
            body: Block::empty(),
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
            body: Block::empty(),
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
            visibility: Visibility::Public,
            name: Ident::new("test"),
            params: vec![],
            return_type: None,
            body: Block::empty(),
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
