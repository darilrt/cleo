use errors::Error;
use types::ScopeID;

pub mod checker;
pub mod context;
pub mod defkinds;
pub mod gather;
pub mod methos;
pub mod resolver;
pub mod symbols;

pub fn resolve(ctx: &mut context::Context, scope: ScopeID, unit: &ast::Unit) -> Result<(), Error> {
    resolver::Resolver::new(ctx).resolve(scope, unit)
}

pub fn check(
    ctx: &mut context::Context,
    scope: ScopeID,
    unit: ast::Unit,
) -> Result<typed_ast::TypedUnit, Error> {
    checker::Checker::new(ctx).check(scope, unit)
}
