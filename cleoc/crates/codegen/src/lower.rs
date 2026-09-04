use errors::Error;
use typed_ast::TypedUnit;
use types::ScopeID;

use crate::ir::UnitIR;

pub fn lower(scope: ScopeID, unit: &TypedUnit) -> Result<UnitIR, Error> {
    for _decl in &unit.decls {}

    Ok(UnitIR::new(scope))
}
