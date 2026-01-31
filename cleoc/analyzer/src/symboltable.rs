use std::collections::HashMap;

pub struct SymbolTable {
    scopes: Vec<Scope>,
}

struct Scope {
    symbols: 
    types: HashMap<String, Symbol>,
}

struct Symbol {
    name: String,
    symbol_type: SymbolType,
}

enum SymbolType {
    Variable,
    Function,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            scopes: vec![Scope {
                symbols: HashMap::new(),
            }],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope {
            symbols: HashMap::new(),
        });
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
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
