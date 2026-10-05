use errors::Error;
use types::ScopeID;

pub mod checker;
pub mod context;
pub mod defkinds;
pub mod gather;
pub mod imports;
pub mod methos;
pub mod resolver;
pub mod symbols;
pub mod unit;

pub fn resolve(ctx: &mut context::Context, scope: ScopeID, unit: &ast::Root) -> Result<(), Error> {
    resolver::Resolver::new(ctx).resolve(scope, unit)
}

pub fn check(
    ctx: &mut context::Context,
    scope: ScopeID,
    unit: ast::Root,
) -> Result<typed_ast::TypedUnit, Error> {
    checker::Checker::new(ctx).check(scope, unit)
}
