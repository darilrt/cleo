use ast::{AssignKind, Ident, Operator, Segment};
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
    pub typeid: TypeID,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Stmt>,
    pub typeid: TypeID,
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
    pub name: String,
    pub typeid: TypeID,
    pub initializer: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Binding {
    Var,
    Const,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Value(ast::ExprValue, TypeID),
    BinaryOp {
        left: Box<Expr>,
        op: Operator,
        right: Box<Expr>,
        typeid: TypeID,
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

impl Expr {
    pub fn type_id(&self) -> TypeID {
        match self {
            Expr::Value(_, typeid) => *typeid,
            Expr::BinaryOp { left, .. } => left.type_id(),
            Expr::UnaryOp { expr, .. } => expr.type_id(),
            Expr::Call(call) => call.typeid,
            Expr::Access(access) => access.expr.type_id(),
            Expr::If(if_expr) => if_expr.then_branch.typeid,
            Expr::Path(path) => path.typeid,
            // Expr::Init(init) => init.path.segments.last().unwrap().name.type_id,
            // Expr::Assign(assign) => assign.right.type_id(),
            // Expr::Loop(block) => block.typeid,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprAssign {
    pub left: Box<Expr>,
    pub kind: AssignKind,
    pub right: Box<Expr>,
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
pub struct ExprAccess {
    pub expr: Box<Expr>,
    pub segment: Segment,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprCall {
    pub typeid: TypeID,
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnDecl {
    pub signature: FnSignature,
    pub block: Block,
    pub scopeid: ScopeID,
    pub typeid: TypeID,
    pub defid: DefID,
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
