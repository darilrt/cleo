pub mod defs;

use std::collections::HashMap;

use crate::defs::TypeDef;

#[derive(Clone, Copy, PartialEq, Hash, Eq, Debug)]
pub struct DefID(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ScopeID(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub struct TypeID(pub usize);

pub struct TypeInterner {
    types: Vec<TypeDef>,
    dedup: HashMap<TypeDef, TypeID>,
}

impl TypeInterner {
    pub fn new() -> Self {
        Self {
            types: Vec::new(),
            dedup: HashMap::new(),
        }
    }

    pub fn types(&self) -> &Vec<TypeDef> {
        &self.types
    }

    pub fn intern(&mut self, type_def: TypeDef) -> TypeID {
        if let Some(&id) = self.dedup.get(&type_def) {
            return id;
        }

        let typeid = TypeID(self.types.len());
        self.types.push(type_def.clone());
        self.dedup.insert(type_def, typeid);

        typeid
    }

    pub fn get(&self, typeid: TypeID) -> Option<&TypeDef> {
        self.types.get(typeid.0)
    }
}

#[cfg(test)]
mod test {
    use std::assert_matches;

    use crate::defs::Signedness;

    use super::*;

    #[test]
    fn test() {
        let mut interner = TypeInterner::new();

        let i8_ty = interner.intern(TypeDef::Int(8, defs::Signedness::Signed));
        let def = interner.get(i8_ty).unwrap();

        assert_matches!(def, TypeDef::Int(8, Signedness::Signed));
    }

    #[test]
    fn dedup_works() {
        let mut interner = TypeInterner::new();

        let a = interner.intern(TypeDef::Int(8, defs::Signedness::Signed));
        let b = interner.intern(TypeDef::Int(8, defs::Signedness::Signed));

        assert_eq!(a, b); // mismo TypeId
    }
}
