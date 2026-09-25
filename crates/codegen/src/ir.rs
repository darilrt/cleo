use ast::AssignKind;
use types::{DefID, ScopeID, TypeID};

#[derive(Debug)]
pub struct UnitIR {
    fns: Vec<FnIR>,
}

impl UnitIR {
    pub fn new() -> Self {
        Self { fns: Vec::new() }
    }

    pub fn add_fn(&mut self, fnir: FnIR) {
        self.fns.push(fnir);
    }

    pub fn fns(&self) -> &[FnIR] {
        &self.fns
    }
}

#[derive(Debug)]
pub struct FnIR {
    pub name: String,
    pub defid: DefID,
    pub scope: ScopeID,
    pub typeid: TypeID,
    pub block: BlockIR,
}

#[derive(Debug)]
pub enum StmtIR {
    Expr(ExprIR),
    Local(String, TypeID, Option<ExprIR>),
    Assign(String, ExprIR),
    If(ExprIR, BlockIR, Option<BlockIR>),
}

#[derive(Debug)]
pub enum ExprIR {
    BinaryOp {
        left: Box<ExprIR>,
        op: String,
        right: Box<ExprIR>,
    },
    Assign {
        left: Box<ExprIR>,
        kind: AssignKind,
        right: Box<ExprIR>,
    },
    Call {
        callee: Box<ExprIR>,
        args: Vec<ExprIR>,
    },
    Atom(String),
}

#[derive(Debug)]
pub struct BlockIR {
    pub stmts: Vec<StmtIR>,
}
