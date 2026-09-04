use types::ScopeID;

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
}

#[derive(Debug)]
pub struct GenType {}
