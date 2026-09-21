use std::collections::HashMap;

use types::{DefID, TypeID};

// TODO: To improve performance the names can be ids instead of strings (NameID)
pub struct MethodTable {
    statics: HashMap<(TypeID, String), DefID>,
    instance: HashMap<(TypeID, String), DefID>,
}

impl MethodTable {
    pub fn new() -> Self {
        Self {
            statics: HashMap::new(),
            instance: HashMap::new(),
        }
    }

    pub fn insert_static(&mut self, name: &str, typeid: TypeID, defid: DefID) {
        self.statics.insert((typeid, name.to_string()), defid);
    }

    pub fn insert(&mut self, name: &str, typeid: TypeID, defid: DefID) {
        self.instance.insert((typeid, name.to_string()), defid);
    }

    pub fn lookup_static(&mut self, name: &str, typeid: TypeID) -> Option<DefID> {
        self.statics.get(&(typeid, name.to_string())).cloned()
    }

    pub fn lookup(&mut self, name: &str, typeid: TypeID) -> Option<DefID> {
        self.instance.get(&(typeid, name.to_string())).cloned()
    }
}
