use types::ScopeID;

pub mod checker;
pub mod context;
pub mod defkinds;
pub mod gather;
pub mod methos;
pub mod resolver;
pub mod symbols;

pub fn resolve(
    ctx: &mut context::Context,
    scope: ScopeID,
    unit: &parser::ast::Unit,
) -> Result<(), String> {
    resolver::Resolver::new(ctx).resolve(scope, unit)
}

pub fn check(
    ctx: &mut context::Context,
    scope: ScopeID,
    unit: &parser::ast::Unit,
) -> Result<(), String> {
    checker::Checker::new(ctx).check(scope, unit)
}
