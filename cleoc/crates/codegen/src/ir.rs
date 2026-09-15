use types::{DefID, ScopeID, TypeID};

#[derive(Debug)]
pub struct UnitIR {
    scope: ScopeID,
    types: Vec<GenType>,
    fns: Vec<FnIR>,
}

impl UnitIR {
    pub fn new(scope: ScopeID) -> Self {
        Self {
            scope,
            types: Vec::new(),
            fns: Vec::new(),
        }
    }

    pub fn add_fn(&mut self, fnir: FnIR) {
        self.fns.push(fnir);
    }

    pub fn fns(&self) -> &[FnIR] {
        &self.fns
    }

    pub fn types(&self) -> &[GenType] {
        &self.types
    }
}

#[derive(Debug)]
pub struct FnIR {
    pub name: String,
    pub defid: DefID,
    pub scope: ScopeID,
    pub typeid: TypeID,
    pub body: Vec<StmtIR>,
}

#[derive(Debug)]
pub enum StmtIR {
    Expr(ExprIR),
    Local(String, TypeID),
    Assign(String, ExprIR),
}

#[derive(Debug)]
pub enum ExprIR {
    Value(String),
}

#[derive(Debug)]
pub struct GenType {}
