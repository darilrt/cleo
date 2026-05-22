use crate::symboltable::SymbolTable;

mod collector;
mod symboltable;

fn analyze(source: &str) -> Result<SymbolTable, String> {
    let unit = parser::parse(source).map_err(|e| format!("Parse error: {:?}", e))?;
    let table = SymbolTable::new();
    Ok(table)
}
