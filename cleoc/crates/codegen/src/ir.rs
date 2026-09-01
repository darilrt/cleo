pub struct UnitIR {
    types: Vec<GenType>,
    fns: Vec<FnIR>,
}

impl UnitIR {
    pub fn new() -> Self {
        Self {
            types: Vec::new(),
            fns: Vec::new(),
        }
    }
}

pub struct FnIR {}

pub struct GenType {}
