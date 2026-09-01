use ast::Ident;
use types::{DefID, ScopeID, TypeID};

#[derive(Debug, Clone, PartialEq)]
pub struct TypedUnit {
    pub decls: Vec<FnDecl>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Ptr(Box<Type>),
    ConstPtr(Box<Type>),
    Path(PathExpr),
    Array(usize, Box<Type>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumValue {
    pub name: Ident,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: Ident,
    pub field_type: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitMethod {
    pub signature: FnSignature,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PathExpr {
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    pub name: Ident,
    pub generics: Option<Vec<Type>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(Expr),
    Defer(Expr),
    Return(Option<Expr>),
    Break,
    Continue,
    Local(Local),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Local {
    pub binding: Binding,
    pub name: Ident,
    pub var_type: Option<Type>,
    pub initializer: Option<Box<Expr>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Binding {
    Var,
    Const,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Value(ExprValue),
    BinaryOp {
        left: Box<Expr>,
        op: Operator,
        right: Box<Expr>,
    },
    UnaryOp {
        op: Operator,
        expr: Box<Expr>,
    },
    Call(ExprCall),
    Access(ExprAccess),
    Path(PathExpr),
    Init(ExprInit),
    Assign(ExprAssign),
    If(ExprIf),
    Loop(Block),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprAssign {
    pub left: Box<Expr>,
    pub kind: AssignKind,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignKind {
    Equal,    // =
    AddEqual, // +=
    SubEqual, // -=
    MulEqual, // *=
    DivEqual, // /=
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprInit {
    pub path: PathExpr,
    pub fields: Vec<ExprInitField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprInitField {
    pub name: Ident,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprIf {
    pub condition: Box<Expr>,
    pub then_branch: Block,
    pub else_branch: Option<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprValue {
    Integer(String),
    Float(String),
    Bool(bool),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprAccess {
    pub expr: Box<Expr>,
    pub segment: Segment,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprCall {
    pub expr: Box<Expr>,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /

    Deref, // *
    Ref,   // &
    Neg,   // -
    Not,   // !

    Equal,        // ==
    NotEqual,     // !=
    Less,         // <
    Greater,      // >
    LessEqual,    // <=
    GreaterEqual, // >=

    And, // &&
    Or,  // ||
}

impl Operator {
    pub fn name(&self) -> &str {
        match self {
            Operator::Add => "add",
            Operator::Div => "divide",
            Operator::Sub => "subtract",
            Operator::Mul => "divde",
            Operator::And
            | Operator::Greater
            | Operator::Equal
            | Operator::Less
            | Operator::GreaterEqual
            | Operator::LessEqual
            | Operator::NotEqual
            | Operator::Or => "compare",
            Operator::Deref => "dereference",
            Operator::Neg => "negate",
            Operator::Not => "invert",
            Operator::Ref => "get reference",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnDecl {
    pub signature: FnSignature,
    pub block: Block,
    pub scopeid: ScopeID,
    pub typeid: TypeID,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnSignature {
    pub name: Ident,
    pub params: Vec<DefID>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericParams(pub Vec<GenericParam>);

#[derive(Debug, Clone, PartialEq)]
pub struct GenericParam {
    pub name: Ident,
    pub bound: Option<Type>,
}
