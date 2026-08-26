use std::{collections::HashMap, fmt::Debug};

use errors::Error;
use types::{DefID, ScopeID, TypeID};

use crate::defkinds::{EnumDef, FnSig, StructDef, TraitDef, TypeAliasDef};

#[derive(Debug)]
pub struct Scope {
    parent: Option<ScopeID>,
    symbols: HashMap<String, DefID>,
}

#[derive(Debug)]
pub struct SymbolTable {
    scopes: Vec<Scope>,
    defs: Vec<Definition>,
}

#[derive(Debug)]
pub struct Definition {
    pub name: String,
    pub kind: DefKind,
}

impl Definition {
    pub fn var(name: &str, ty: TypeID) -> Self {
        Self {
            name: name.to_string(),
            kind: DefKind::Variable { typeid: ty },
        }
    }

    pub fn typeid(&self) -> Option<TypeID> {
        match &self.kind {
            DefKind::Enum(def) => Some(def.typeid),
            DefKind::Struct(def) => Some(def.typeid),
            DefKind::TypeAlias(def) => Some(def.typeid),
            DefKind::Trait(def) => Some(def.typeid),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum DefKind {
    Variable { typeid: TypeID },
    TypeAlias(TypeAliasDef),
    Struct(StructDef),
    Trait(TraitDef),
    Enum(EnumDef),
    Function(FnSig),
    Module { scope: ScopeID },
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            defs: Vec::new(),
            scopes: {
                let mut vec = Vec::new();
                vec.push(Scope::new(None));
                vec
            },
        }
    }

    pub fn get_scope(&self, scope: ScopeID) -> Option<&Scope> {
        self.scopes.get(scope.0)
    }

    pub fn root(&self) -> ScopeID {
        ScopeID(0)
    }

    pub fn push(&mut self, parent: ScopeID) -> Result<ScopeID, Error> {
        let Some(_) = self.scopes.get(parent.0) else {
            return Err(format!("invalid parent scope: {:?}", parent.0));
        };

        let id = ScopeID(self.scopes.len());
        self.scopes.push(Scope::new(Some(parent)));

        Ok(id)
    }

    pub fn define(&mut self, scope: ScopeID, def: Definition) -> Result<DefID, String> {
        let Some(scope) = self.scopes.get_mut(scope.0) else {
            return Err(format!("invalid scope: {:?}", scope.0));
        };

        if scope.symbols.contains_key(&def.name) {
            return Err(format!("symbol already defined: {}", def.name));
        }

        let def_id = DefID(self.defs.len());
        scope.symbols.insert(def.name.clone(), def_id);
        self.defs.push(def);

        Ok(def_id)
    }

    pub fn get_def(&self, defid: DefID) -> Option<&Definition> {
        self.defs.get(defid.0)
    }

    pub fn get_def_mut(&mut self, defid: DefID) -> Option<&mut Definition> {
        self.defs.get_mut(defid.0)
    }

    pub fn lookup(&self, scope: ScopeID, name: &str) -> Option<DefID> {
        let Some(scope) = self.scopes.get(scope.0) else {
            return None;
        };

        if let Some(&defid) = scope.symbols.get(name) {
            return Some(defid);
        }

        if let Some(parent) = scope.parent {
            return self.lookup(parent, name);
        }

        None
    }

    pub fn lookup_local(&self, scope: ScopeID, name: &str) -> Option<DefID> {
        let Some(scope) = self.scopes.get(scope.0) else {
            return None;
        };

        scope.symbols.get(name).copied()
    }
}

impl Scope {
    pub fn new(parent: Option<ScopeID>) -> Self {
        Self {
            parent: parent,
            symbols: HashMap::new(),
        }
    }

    pub fn parent(&self) -> Option<ScopeID> {
        self.parent
    }

    pub fn symbols(&self) -> &HashMap<String, DefID> {
        &self.symbols
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn define_and_lookup() {
        let mut table = super::SymbolTable::new();
        let root = table.root();

        let defid = table
            .define(
                root,
                super::Definition {
                    name: "x".to_string(),
                    kind: super::DefKind::Variable {
                        typeid: super::TypeID(0),
                    },
                },
            )
            .unwrap();

        let lookup_id = table.lookup(root, "x").unwrap();
        assert_eq!(defid, lookup_id);
    }

    #[test]
    fn nested_scopes() {
        let mut table = super::SymbolTable::new();
        let root = table.root();

        let defid = table
            .define(
                root,
                super::Definition {
                    name: "x".to_string(),
                    kind: super::DefKind::Variable {
                        typeid: super::TypeID(0),
                    },
                },
            )
            .unwrap();

        let child_scope = table.push(root).unwrap();
        let lookup_id = table.lookup(child_scope, "x").unwrap();
        assert_eq!(defid, lookup_id);
    }

    #[test]
    fn shadowing() {
        let mut table = super::SymbolTable::new();
        let root = table.root();

        let def_id_root = table
            .define(
                root,
                super::Definition {
                    name: "x".to_string(),
                    kind: super::DefKind::Variable {
                        typeid: super::TypeID(0),
                    },
                },
            )
            .unwrap();

        let child_scope = table.push(root).unwrap();
        let defid_child = table
            .define(
                child_scope,
                super::Definition {
                    name: "x".to_string(),
                    kind: super::DefKind::Variable {
                        typeid: super::TypeID(1),
                    },
                },
            )
            .unwrap();

        let lookup_id_child = table.lookup(child_scope, "x").unwrap();
        assert_eq!(defid_child, lookup_id_child);

        let lookup_id_root = table.lookup(root, "x").unwrap();
        assert_eq!(def_id_root, lookup_id_root);
    }
}
