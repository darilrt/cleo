use std::collections::HashMap;

pub struct SymbolTable {
    scopes: Vec<Scope>,
}

struct Scope(HashMap<String, Symbol>);

struct Symbol {
    name: String,
    symbol_type: SymbolKind,
}

enum SymbolKind {
    Variable,
    Function,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable { scopes: Vec::new() }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope(HashMap::new()));
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }
}

impl Symbol {
    pub fn function(name: String) -> Self {
        Symbol {
            name,
            symbol_type: SymbolKind::Function,
        }
    }
}

#[allow(unused)]
mod test {
    use super::*;

    #[test]
    fn test_symbol_table() {
        let mut sym_table = SymbolTable::new();
        sym_table.push_scope();
        sym_table.pop_scope();
    }
}
