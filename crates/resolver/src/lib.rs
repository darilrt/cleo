use errors::Error;
use typed_ast::Expr;
use types::{ScopeID, TypeID};

use crate::context::Primitives;

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

pub fn type_id(expr: &Expr, primitives: &Primitives) -> TypeID {
    match expr {
        Expr::Value(_, typeid) => *typeid,
        Expr::BinaryOp { typeid, .. } => *typeid,
        Expr::UnaryOp {
            op: _,
            expr: _,
            typeid,
        } => *typeid,
        Expr::Call(call) => call.typeid,
        Expr::Access(access) => access.typeid,
        Expr::If(if_expr) => if_expr.then_branch.typeid,
        Expr::Path(path) => path.typeid,
        Expr::Reinterpret {
            inner: _,
            from: _,
            to,
        } => *to,
        Expr::Assign(assign) => type_id(&assign.right, primitives),
        Expr::Loop(block) => block.typeid,
        Expr::ProcPath(proc) => proc.typeid,
        Expr::SizeOf { .. } => primitives.usize_t,
        _ => unimplemented!("{:?}", expr),
        // Expr::Init(init) => init.path.segments.last().unwrap().name.type_id,
    }
}
