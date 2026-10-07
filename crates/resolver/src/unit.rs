use std::collections::HashMap;

use types::ScopeID;

pub struct UnitRegistry {
    units: HashMap<String, ScopeID>,
    paths: HashMap<ScopeID, String>,
}

impl UnitRegistry {
    pub fn new() -> Self {
        Self {
            units: HashMap::new(),
            paths: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: String, scope: ScopeID) {
        self.units.insert(name.clone(), scope);
        self.paths.insert(scope, name);
    }

    pub fn get(&self, name: &str) -> Option<ScopeID> {
        self.units.get(name).copied()
    }

    pub fn get_path(&self, scope: ScopeID) -> Option<&String> {
        self.paths.get(&scope)
    }
}
