use errors::Error;
use parser::ast::{Decl, Expr, ExprValue, FnDecl, Operator, PathExpr, Stmt, Unit};
use types::{DefID, ScopeID, TypeID, defs::TypeDef};

use crate::{
    context::Context,
    resolver::Resolved,
    symbols::{DefKind, Definition},
};

pub struct BlockCtx {
    #[allow(unused)]
    allow_break: bool,

    #[allow(unused)]
    allow_continue: bool,

    #[allow(unused)]
    expected_return: Option<TypeID>,
}

pub struct Checker<'a> {
    ctx: &'a mut Context,
}

impl<'a> Checker<'a> {
    pub fn new(ctx: &'a mut Context) -> Self {
        Self { ctx }
    }

    pub fn check(&mut self, scope: ScopeID, unit: &Unit) -> Result<(), Error> {
        for decl in &unit.decls {
            match decl {
                Decl::Fn(decl) => {
                    self.resolve_fn(scope, decl)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn resolve_fn(&mut self, scope: ScopeID, decl: &FnDecl) -> Result<(), Error> {
        let fn_name = decl.signature.name.str();

        let defid = self
            .ctx
            .table
            .lookup_local(scope, &fn_name)
            .ok_or_else(|| format!("Function '{}' not found in symbol table", fn_name))?;

        let DefKind::Function(sig) = &self
            .ctx
            .table
            .get_def(defid)
            .ok_or_else(|| {
                format!(
                    "Definition for function '{}' not found in symbol table",
                    fn_name
                )
            })?
            .kind
        else {
            return Err(format!("Definition for '{}' is not a function", fn_name));
        };

        let TypeDef::FnPointer(fntype) = self
            .ctx
            .interner
            .get(sig.typeid)
            .ok_or_else(|| format!("Function type for '{}' not found in interner", fn_name))?
        else {
            return Err(format!("Type for '{}' is not a function pointer", fn_name));
        };

        let body_scope = self.ctx.table.push(scope)?;

        if fntype.params.len() != decl.signature.params.len() {
            return Err(format!(
                "Function '{}' expects {} parameters, but {} were provided",
                fn_name,
                fntype.params.len(),
                decl.signature.params.len()
            ));
        }

        fntype
            .params
            .iter()
            .zip(&decl.signature.params)
            .try_for_each(|(typeid, param)| {
                let param_name = param.name.str();

                self.ctx
                    .table
                    .define(body_scope, Definition::var(param_name, typeid.clone()))?;

                Ok::<(), Error>(())
            })?;

        self.resolve_block(
            body_scope,
            &decl.block.statements,
            &BlockCtx {
                allow_break: false,
                allow_continue: false,
                expected_return: Some(fntype.return_type),
            },
        )?;

        Ok(())
    }

    pub fn resolve_block(
        &mut self,
        scope: ScopeID,
        stmts: &[Stmt],
        ctx: &BlockCtx,
    ) -> Result<(), Error> {
        for stmt in stmts {
            self.resolve_stmt(scope, stmt, ctx)?;
        }
        Ok(())
    }

    pub fn resolve_stmt(
        &mut self,
        scope: ScopeID,
        stmt: &Stmt,
        ctx: &BlockCtx,
    ) -> Result<(), Error> {
        match stmt {
            Stmt::Break => {
                if !ctx.allow_break {
                    return Err("Not allowed Break here".to_string());
                }
            }
            Stmt::Continue => {
                if !ctx.allow_continue {
                    return Err("Not allowed Continue here".to_string());
                }
            }
            Stmt::Local(local) => {
                let var_type = self.ctx.type_to_id(
                    scope,
                    local
                        .var_type
                        .as_ref()
                        .unwrap_or_else(|| todo!("Implemente Infered types")),
                )?;

                self.ctx
                    .table
                    .define(scope, Definition::var(local.name.str(), var_type))?;

                if let Some(expr) = local.initializer.as_ref() {
                    let expr_type = self.resolve_expr(scope, expr.as_ref())?;

                    if var_type != expr_type {
                        return Err(format!(
                            "expected {}, found {}",
                            self.ctx.type_name(var_type),
                            self.ctx.type_name(expr_type)
                        ));
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn resolve_expr(&mut self, scope: ScopeID, expr: &Expr) -> Result<TypeID, Error> {
        match expr {
            Expr::Value(value) => match value {
                ExprValue::Integer(_lit) => Ok(self.ctx.primitives.i32_),
                ExprValue::Float(_lit) => Ok(self.ctx.primitives.f32_),
                ExprValue::Bool(_value) => Ok(self.ctx.primitives.bool_),
                ExprValue::String(lit) => Ok(self.ctx.interner.intern(TypeDef::Array {
                    element: self.ctx.primitives.u8_,
                    size: lit.len(),
                })),
            },
            Expr::BinaryOp { left, op, right } => self.resolve_binaryop(scope, left, op, right),
            Expr::Path(path) => match self.resolve_pathexpr(scope, path)? {
                Resolved::Value(typeid) => Ok(typeid),
                Resolved::EnumVariant(typeid) => Ok(typeid),
                _ => Err(format!(
                    "Expected value, found {:?} in path expression",
                    path
                )),
            },
            _ => Ok(TypeID(0)),
        }
    }

    pub fn resolve_binaryop(
        &mut self,
        scope: ScopeID,
        left: &Expr,
        op: &Operator,
        right: &Expr,
    ) -> Result<TypeID, Error> {
        let l = self.resolve_expr(scope, left)?;
        let r = self.resolve_expr(scope, right)?;

        if self.ctx.is_numeric(&l) && self.ctx.is_numeric(&r) && l == r {
            Ok(l)
        } else {
            Err(format!(
                "cannot add {} and {}",
                self.ctx.type_name(l),
                self.ctx.type_name(r)
            ))
        }
    }

    pub fn resolve_pathexpr(&mut self, scope: ScopeID, path: &PathExpr) -> Result<Resolved, Error> {
        let mut it = path.segments.iter();
        let first = it
            .next()
            .ok_or_else(|| "expected at least 1 segment in PathExpr")?;

        let defid = self
            .ctx
            .table
            .lookup(scope, first.name.str())
            .ok_or_else(|| format!("'{}' not defined", first.name.str()))?;
        let def = self
            .ctx
            .table
            .get_def(defid)
            .ok_or_else(|| format!("unknown definition '{}'", first.name.str()))?;

        let mut current = self.def_to_resolved(def, defid);

        for seg in it {
            current = self.resolve_next(current, seg.name.str())?;
        }

        Ok(current)
    }

    fn def_to_resolved(&self, def: &Definition, def_id: DefID) -> Resolved {
        match &def.kind {
            DefKind::Variable { typeid } => Resolved::Value(*typeid),
            DefKind::Function(sig) => Resolved::Value(sig.typeid),
            DefKind::Struct(s) => Resolved::Type(s.typeid, def_id),
            DefKind::Enum(e) => Resolved::Type(e.typeid, def_id),
            DefKind::Trait(t) => Resolved::Type(t.typeid, def_id),
            DefKind::TypeAlias(a) => Resolved::Type(a.typeid, def_id),
            DefKind::Module { scope } => Resolved::Module(*scope),
        }
    }

    pub fn resolve_next(&self, current: Resolved, name: &str) -> Result<Resolved, Error> {
        match current {
            Resolved::Module(scope) => {
                let def_id = self.ctx.table.lookup(scope, name).ok_or_else(|| {
                    format!(
                        "Cannot resolve member '{}' in module at scope {:?}",
                        name, scope
                    )
                })?;
                let def = self.ctx.table.get_def(def_id).ok_or_else(|| {
                    format!(
                        "Cannot resolve member '{}' in module at scope {:?}",
                        name, scope
                    )
                })?;
                Ok(self.def_to_resolved(def, def_id))
            }

            Resolved::Value(ty) => {
                let field_ty = self.resolve_field(ty, name)?;
                Ok(Resolved::Value(field_ty))
            }

            Resolved::Type(_, def_id) => {
                let def = self.ctx.table.get_def(def_id).unwrap();
                match &def.kind {
                    DefKind::Enum(_) => {
                        todo!("resolve enum variant access");
                    }
                    DefKind::Struct(_) => Err(format!(
                        "cannot access '{}' on type '{}', use an instance",
                        name, def.name
                    )),
                    DefKind::Trait(_) => {
                        Err(format!("cannot access '{}' on trait '{}'", name, def.name))
                    }

                    _ => Err(format!("cannot access '{}' on '{}'", name, def.name)),
                }
            }

            Resolved::EnumVariant(_typeid) => Err(format!(
                "cannot access '{}' on enum variant, use an instance",
                name
            )),
        }
    }

    pub fn resolve_field(&self, _typeid: TypeID, _name: &str) -> Result<TypeID, Error> {
        Ok(TypeID(0)) // TODO: Implement field resolution based on typeid and name
    }
}
