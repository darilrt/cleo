pub mod bodyemitter;
pub mod codegen;
pub mod ir;

mod lower;

pub use lower::lower;

use std::io;

use errors::Error;
use resolver::context;
use typed_ast::TypedUnit;
use types::ScopeID;

pub fn generate(
    ctx: &context::Context,
    buffer: &mut impl io::Write,
    scope: ScopeID,
    unit: &TypedUnit,
) -> Result<(), Error> {
    let ir = lower(scope, unit)?;

    let codegen = codegen::Codegen::new(ctx);

    codegen.emit_header(buffer, scope)?;

    for fnir in ir.fns() {
        codegen.emit_fn_def(buffer, scope, fnir)?;
    }

    Ok(())
}
