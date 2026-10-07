use types::TypeID;

#[derive(Debug)]
pub struct StructDef {
    pub fields: Vec<(String, TypeID)>,
    pub resolved: bool,
    pub typeid: TypeID,
}

impl StructDef {
    pub fn get_field(&self, name: &str) -> Option<TypeID> {
        self.fields
            .iter()
            .find(|(field, _)| field == name)
            .map(|v| v.1)
    }
}

#[derive(Debug)]
pub struct TraitDef {
    pub resolved: bool,
    pub typeid: TypeID,
}

#[derive(Debug)]
pub struct EnumDef {
    pub resolved: bool,
    pub values: Vec<String>,
    pub typeid: TypeID,
}

#[derive(Debug)]
pub struct TypeAliasDef {
    pub typeid: TypeID,
    pub resolved: bool,
}

#[derive(Debug)]
pub struct ProcSig {
    pub name: String,
    pub params: Vec<String>,
    pub typeid: TypeID,
    pub return_type: TypeID,
    pub resolved: bool,
    pub is_extern: bool,
    pub no_emit: bool,
}

impl ProcSig {
    pub fn unresolved(name: &str) -> Self {
        Self {
            name: name.to_string(),
            params: Vec::new(),
            return_type: TypeID(0),
            resolved: false,
            typeid: TypeID(0),
            is_extern: false,
            no_emit: false,
        }
    }
}
