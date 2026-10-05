use std::collections::HashMap;

use types::ScopeID;

pub struct UnitRegistry {
    units: HashMap<String, ScopeID>,
}

impl UnitRegistry {
    pub fn new() -> Self {
        Self {
            units: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: String, scope: ScopeID) {
        self.units.insert(name, scope);
    }

    pub fn get(&self, name: &str) -> Option<ScopeID> {
        self.units.get(name).copied()
    }
}
