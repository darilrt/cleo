use errors::Error;
use typed_ast::TypedUnit;

use crate::ir::UnitIR;

pub fn lower(unit: &TypedUnit) -> Result<UnitIR, Error> {
    for _decl in &unit.decls {}

    Ok(UnitIR::new())
}
