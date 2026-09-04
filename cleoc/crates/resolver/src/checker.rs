use ast::{
    Block, Decl, Expr, ExprAccess, ExprAssign, ExprCall, ExprIf, ExprInit, ExprValue, FnDecl,
    Operator, PathExpr, Stmt, Unit,
};
use errors::Error;
use typed_ast::TypedUnit;
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

    pub fn check(&mut self, scope: ScopeID, unit: Unit) -> Result<TypedUnit, Error> {
        let mut out = TypedUnit { decls: Vec::new() };

        for decl in unit.decls {
            match decl {
                Decl::Fn(decl) => {
                    out.decls.push(self.resolve_fn(scope, decl)?);
                }
                _ => {}
            }
        }

        Ok(out)
    }

    pub fn resolve_fn(&mut self, scope: ScopeID, decl: FnDecl) -> Result<typed_ast::FnDecl, Error> {
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
            return Err(format!("Definition for '{}' is not a function", fn_name).into());
        };

        let typeid = sig.typeid;

        let TypeDef::FnPointer(fntype) = self
            .ctx
            .interner
            .get(sig.typeid)
            .ok_or_else(|| format!("Function type for '{}' not found in interner", fn_name))?
        else {
            return Err(format!("Type for '{}' is not a function pointer", fn_name).into());
        };

        let body_scope = self.ctx.table.push(scope)?;

        if fntype.params.len() != decl.signature.params.len() {
            return Err(format!(
                "Function '{}' expects {} parameters, but {} were provided",
                fn_name,
                fntype.params.len(),
                decl.signature.params.len()
            )
            .into());
        }

        let params_defid = fntype
            .params
            .iter()
            .zip(&decl.signature.params)
            .map(|(typeid, param)| {
                let param_name = param.name.str();

                self.ctx
                    .table
                    .define(body_scope, Definition::var(param_name, typeid.clone()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        self.resolve_block(
            body_scope,
            &decl.block.statements,
            &BlockCtx {
                allow_break: false,
                allow_continue: false,
                expected_return: Some(fntype.return_type),
            },
        )?;

        Ok(typed_ast::FnDecl {
            signature: typed_ast::FnSignature {
                name: decl.signature.name,
                params: params_defid,
            },
            block: typed_ast::Block {
                statements: Vec::new(),
            },
            scopeid: body_scope,
            typeid,
        })
    }

    pub fn resolve_block(
        &mut self,
        scope: ScopeID,
        stmts: &[Stmt],
        block_ctx: &BlockCtx,
    ) -> Result<(), Error> {
        for stmt in stmts {
            self.resolve_stmt(scope, stmt, block_ctx)?;
        }
        Ok(())
    }

    pub fn resolve_block_value(
        &mut self,
        scope: ScopeID,
        stmts: &[Stmt],
        block_ctx: &BlockCtx,
    ) -> Result<TypeID, Error> {
        let Some((last, stmts)) = stmts.split_last() else {
            return Ok(self.ctx.primitives.void);
        };

        for stmt in stmts {
            self.resolve_stmt(scope, stmt, block_ctx)?;
        }

        match last {
            Stmt::Expr(expr) => self.resolve_expr(scope, expr, block_ctx),
            _ => {
                self.resolve_stmt(scope, last, block_ctx)?;
                Ok(self.ctx.primitives.void)
            }
        }
    }

    pub fn resolve_stmt(
        &mut self,
        scope: ScopeID,
        stmt: &Stmt,
        block_ctx: &BlockCtx,
    ) -> Result<(), Error> {
        match stmt {
            Stmt::Break => {
                if !block_ctx.allow_break {
                    return Err("Not allowed Break here".to_string().into());
                }
            }
            Stmt::Continue => {
                if !block_ctx.allow_continue {
                    return Err("Not allowed Continue here".to_string().into());
                }
            }
            Stmt::Return(value) => match (block_ctx.expected_return, value) {
                (Some(expected), None) => {
                    return Err(format!(
                        "expected return value of type {}",
                        self.ctx.type_name(expected)
                    )
                    .into());
                }
                (None, Some(_)) => {
                    return Err("unexpected return value in void function"
                        .to_string()
                        .into());
                }
                (Some(expected), Some(expr)) => {
                    let ret_type = self.resolve_expr(scope, expr, block_ctx)?;
                    if !self.ctx.is_coercible(ret_type, expected) {
                        return Err(format!(
                            "expected return type {}, found {}",
                            self.ctx.type_name(expected),
                            self.ctx.type_name(ret_type)
                        )
                        .into());
                    }
                }
                (None, None) => {}
            },
            Stmt::Local(local) => {
                let var_type = self.ctx.resolve_id(
                    scope,
                    local
                        .var_type
                        .as_ref()
                        .unwrap_or_else(|| todo!("Implemente Infered types")),
                )?;

                if let Some(expr) = local.initializer.as_ref() {
                    let expr_type = self.resolve_expr(scope, expr.as_ref(), block_ctx)?;

                    if !self.ctx.is_coercible(expr_type, var_type) {
                        return Err(format!(
                            "expected {}, found {}",
                            self.ctx.type_name(var_type),
                            self.ctx.type_name(expr_type)
                        )
                        .into());
                    }
                }

                self.ctx
                    .table
                    .define(scope, Definition::var(local.name.str(), var_type))?;
            }
            Stmt::Expr(expr) => {
                self.resolve_expr(scope, expr, block_ctx)?;
            }
            Stmt::Defer(expr) => {
                self.resolve_expr(scope, expr, block_ctx)?;
            }
        }
        Ok(())
    }

    pub fn resolve_expr(
        &mut self,
        scope: ScopeID,
        expr: &Expr,
        block_ctx: &BlockCtx,
    ) -> Result<TypeID, Error> {
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
            Expr::BinaryOp { left, op, right } => {
                self.resolve_binaryop(scope, left, op, right, block_ctx)
            }
            Expr::UnaryOp { op, expr } => self.resolve_unaryop(scope, op, expr, block_ctx),
            Expr::Path(path) => match self.resolve_pathexpr(scope, path)? {
                Resolved::Value(typeid) => Ok(typeid),
                Resolved::EnumValue(typeid) => Ok(typeid),
                _ => Err(format!("Expected value, found {:?} in path expression", path).into()),
            },
            Expr::Call(expr) => self.resolve_callexpr(scope, expr, block_ctx),
            Expr::Assign(expr) => self.resolve_assign(scope, expr, block_ctx),
            Expr::Access(expr) => self.resolve_access(scope, expr, block_ctx),
            Expr::If(expr) => self.resolve_if(scope, expr, block_ctx),
            Expr::Init(expr) => self.resolve_init(scope, expr, block_ctx),
            Expr::Loop(block) => self.resolve_loop(scope, block, block_ctx),
        }
    }

    pub fn resolve_loop(
        &mut self,
        _scope: ScopeID,
        _expr: &Block,
        _block_ctx: &BlockCtx,
    ) -> Result<TypeID, Error> {
        todo!("Not implemented loop until codegen")
    }

    pub fn resolve_init(
        &mut self,
        scope: ScopeID,
        expr: &ExprInit,
        block_ctx: &BlockCtx,
    ) -> Result<TypeID, Error> {
        let typeid = match self.resolve_pathexpr(scope, &expr.path)? {
            Resolved::Type(typeid, _defid) => typeid,
            _ => return Err(format!("not a struct").into()),
        };

        let type_def = self
            .ctx
            .interner
            .get(typeid)
            .ok_or_else(|| format!("type {} does not exists", typeid.0))?;

        let struct_def = match type_def {
            TypeDef::UserDef(defid) => {
                let def = self
                    .ctx
                    .table
                    .get_def(*defid)
                    .ok_or_else(|| format!("definition {} does not exists", defid.0))?;

                match &def.kind {
                    DefKind::Struct(def) => Ok(def.fields.clone()),
                    _ => Err(format!("not a strcut")),
                }
            }
            _ => Err(format!("not a strcut")),
        }?;

        if expr.fields.len() < struct_def.len() {
            return Err(format!("missing struct fields").into());
        }

        expr.fields
            .iter()
            .try_for_each::<_, Result<(), Error>>(|field| {
                let field_type = struct_def.iter().find_map(|(name, typeid)| {
                    if name == &field.name.string() {
                        Some(typeid)
                    } else {
                        None
                    }
                });

                let Some(typeid) = field_type else {
                    return Err(format!(
                        "{} does not have field '{}'",
                        self.ctx.type_name(typeid),
                        field.name.str()
                    )
                    .into());
                };

                let exprid = self.resolve_expr(scope, &field.value, block_ctx)?;

                if &exprid != typeid {
                    return Err(format!(
                        "expected {}, found {}",
                        self.ctx.type_name(*typeid),
                        self.ctx.type_name(exprid)
                    )
                    .into());
                }

                Ok(())
            })?;

        Ok(typeid)
    }

    pub fn resolve_if(
        &mut self,
        scope: ScopeID,
        expr: &ExprIf,
        block_ctx: &BlockCtx,
    ) -> Result<TypeID, Error> {
        let condition = self.resolve_expr(scope, &expr.condition, block_ctx)?;

        if !self.ctx.primitives.is_bool(condition) {
            return Err(format!(
                "expected bool in if condition, found {}",
                self.ctx.type_name(condition)
            )
            .into());
        }

        let then_return = {
            let scope = self.ctx.table.push(scope)?;
            self.resolve_block_value(scope, &expr.then_branch.statements, block_ctx)?
        };

        let else_return = if let Some(else_branch) = &expr.else_branch {
            let scope = self.ctx.table.push(scope)?;
            self.resolve_block_value(scope, &else_branch.statements, block_ctx)?
        } else {
            self.ctx.primitives.void
        };

        if then_return == else_return {
            Ok(then_return)
        } else {
            Err(format!(
                "if branches have incompatible types: {} and {}",
                self.ctx.type_name(then_return),
                self.ctx.type_name(else_return)
            )
            .into())
        }
    }

    pub fn resolve_access(
        &mut self,
        scope: ScopeID,
        expr: &ExprAccess,
        block_ctx: &BlockCtx,
    ) -> Result<TypeID, Error> {
        let base = Resolved::Value(self.resolve_expr(scope, &expr.expr, block_ctx)?);

        match self.resolve_next(base, expr.segment.name.str())? {
            Resolved::Value(typeid) => Ok(typeid),
            _ => unreachable!("a type could not be reachable from a instance"),
        }
    }

    pub fn resolve_assign(
        &mut self,
        scope: ScopeID,
        expr: &ExprAssign,
        block_ctx: &BlockCtx,
    ) -> Result<TypeID, Error> {
        if !self.is_lvalue(&expr.left) {
            return Err("invalid left-hand side of assignment".to_string().into());
        }

        let left = self.resolve_expr(scope, &expr.left, block_ctx)?;
        let right = self.resolve_expr(scope, &expr.right, block_ctx)?;

        if !self.ctx.is_coercible(right, left) {
            return Err(format!(
                "expected {}, found {}",
                self.ctx.type_name(left),
                self.ctx.type_name(right)
            )
            .into());
        }

        Ok(self.ctx.primitives.void)
    }

    pub fn is_lvalue(&mut self, expr: &Expr) -> bool {
        match expr {
            Expr::Path(_) => true,
            Expr::UnaryOp {
                op: Operator::Deref,
                ..
            } => true,
            Expr::Access(_) => true,
            _ => false,
        }
    }

    pub fn resolve_callexpr(
        &mut self,
        scope: ScopeID,
        expr: &ExprCall,
        block_ctx: &BlockCtx,
    ) -> Result<TypeID, Error> {
        let (params, return_type) = {
            let fn_typeid = self.resolve_expr(scope, &expr.expr, block_ctx)?;
            let TypeDef::FnPointer(fn_type) = self
                .ctx
                .interner
                .get(fn_typeid)
                .ok_or_else(|| format!("type {} does not exists", fn_typeid.0))?
            else {
                return Err(
                    format!("type {} is not callable", self.ctx.type_name(fn_typeid)).into(),
                );
            };

            if fn_type.params.len() != expr.args.len() {
                return Err(format!(
                    "expected {} arguments, found {}",
                    fn_type.params.len(),
                    expr.args.len()
                )
                .into());
            }

            (fn_type.params.clone(), fn_type.return_type)
        };

        for (param, arg) in params.iter().zip(&expr.args) {
            let arg = self.resolve_expr(scope, arg, block_ctx)?;

            if !self.ctx.is_coercible(arg, *param) {
                return Err(format!(
                    "expected {}, found {}",
                    self.ctx.type_name(*param),
                    self.ctx.type_name(arg)
                )
                .into());
            }
        }

        Ok(return_type)
    }

    pub fn resolve_unaryop(
        &mut self,
        scope: ScopeID,
        op: &Operator,
        expr: &Expr,
        block_ctx: &BlockCtx,
    ) -> Result<TypeID, Error> {
        let typeid = self.resolve_expr(scope, expr, block_ctx)?;

        match op {
            Operator::Neg => {
                if self.ctx.primitives.is_negatable(typeid) {
                    Ok(typeid)
                } else {
                    Err(format!("cannot negate {}", self.ctx.type_name(typeid)).into())
                }
            }
            Operator::Not => {
                if self.ctx.primitives.is_bool(typeid) {
                    Ok(typeid)
                } else {
                    Err(format!("expected bool, found {}", self.ctx.type_name(typeid)).into())
                }
            }
            Operator::Deref => match self
                .ctx
                .interner
                .get(typeid)
                .ok_or_else(|| format!("type {} does not exists", typeid.0))?
            {
                TypeDef::Pointer {
                    pointee,
                    mutability: _,
                } => Ok(*pointee),
                _ => Err(format!("cannot dereference {}", self.ctx.type_name(typeid)).into()),
            },
            Operator::Ref => match self
                .ctx
                .interner
                .get(typeid)
                .ok_or_else(|| format!("type {} does not exists", typeid.0))?
            {
                TypeDef::Array { element, size: _ } => {
                    Ok(self.ctx.interner.intern(TypeDef::Pointer {
                        pointee: *element,
                        mutability: true,
                    }))
                }
                TypeDef::FnPointer(..) => Ok(typeid),
                _ => Ok(self.ctx.interner.intern(TypeDef::Pointer {
                    pointee: typeid,
                    mutability: true,
                })),
            },
            _ => unreachable!("not a unary operator"),
        }
    }

    pub fn resolve_binaryop(
        &mut self,
        scope: ScopeID,
        left: &Expr,
        op: &Operator,
        right: &Expr,
        block_ctx: &BlockCtx,
    ) -> Result<TypeID, Error> {
        let l = self.resolve_expr(scope, left, block_ctx)?;
        let r = self.resolve_expr(scope, right, block_ctx)?;

        match op {
            Operator::Add | Operator::Sub | Operator::Mul | Operator::Div => {
                if l == r && self.ctx.primitives.is_numeric(l) {
                    Ok(l)
                } else {
                    Err(format!(
                        "cannot {} {} and {}",
                        op.name(),
                        self.ctx.type_name(l),
                        self.ctx.type_name(r)
                    )
                    .into())
                }
            }
            Operator::Equal | Operator::NotEqual => {
                if l == r && self.ctx.is_comparable(l) {
                    Ok(self.ctx.primitives.bool_)
                } else {
                    Err(format!(
                        "cannot compare {} and {}",
                        self.ctx.type_name(l),
                        self.ctx.type_name(r)
                    )
                    .into())
                }
            }
            Operator::Less | Operator::Greater | Operator::LessEqual | Operator::GreaterEqual => {
                if l == r && self.ctx.primitives.is_numeric(l) {
                    Ok(self.ctx.primitives.bool_)
                } else {
                    Err(format!(
                        "cannot compare {} and {}",
                        self.ctx.type_name(l),
                        self.ctx.type_name(r)
                    )
                    .into())
                }
            }
            Operator::And | Operator::Or => {
                if l == self.ctx.primitives.bool_ && r == self.ctx.primitives.bool_ {
                    Ok(self.ctx.primitives.bool_)
                } else {
                    Err(format!(
                        "expected bool, found {} and {}",
                        self.ctx.type_name(l),
                        self.ctx.type_name(r)
                    )
                    .into())
                }
            }
            Operator::Deref | Operator::Neg | Operator::Ref | Operator::Not => {
                unreachable!("Not parseable binary operation")
            }
        }
    }

    pub fn resolve_pathexpr(&mut self, scope: ScopeID, path: &PathExpr) -> Result<Resolved, Error> {
        let mut it = path.segments.iter();
        let first = it
            .next()
            .ok_or_else(|| "expected at least 1 segment in PathExpr".to_string())?;

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

    pub fn resolve_next(&mut self, current: Resolved, name: &str) -> Result<Resolved, Error> {
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
                let field_ty = self.resolve_field_or_method(ty, name)?;
                Ok(Resolved::Value(field_ty))
            }
            Resolved::Type(typeid, def_id) => {
                let def = self.ctx.table.get_def(def_id).unwrap();
                match &def.kind {
                    DefKind::Enum(enum_def) => {
                        if enum_def.values.contains(&name.to_string()) {
                            Ok(Resolved::EnumValue(typeid))
                        } else {
                            Err(format!("enum '{}' has no value '{}'", def.name, name).into())
                        }
                    }
                    DefKind::Struct(_) => Err(format!(
                        "cannot access '{}' on type '{}', use an instance",
                        name, def.name
                    )
                    .into()),
                    DefKind::Trait(_) => {
                        Err(format!("cannot access '{}' on trait '{}'", name, def.name).into())
                    }
                    _ => Err(format!("cannot access '{}' on '{}'", name, def.name).into()),
                }
            }
            Resolved::EnumValue(_typeid) => {
                Err(format!("cannot access '{}' on enum variant, use an instance", name).into())
            }
        }
    }

    /// This method asume the typeid a type of an instanced value, not a type
    /// So it returns a value type of ar instance methods
    pub fn resolve_field_or_method(&mut self, typeid: TypeID, name: &str) -> Result<TypeID, Error> {
        let mut typeid = typeid;
        loop {
            match self.ctx.interner.get(typeid) {
                Some(TypeDef::Pointer { pointee, .. }) => typeid = *pointee,
                _ => break,
            }
        }

        if let Some(method) = self.ctx.methods.lookup(name, typeid) {
            let def = self
                .ctx
                .table
                .get_def(method)
                .ok_or_else(|| "internal: method DefID not found".to_string())?;

            match &def.kind {
                DefKind::Function(fnsig) => Ok(fnsig.typeid),
                _ => unreachable!("MethodTable contains non-function DefID"),
            }
        } else if let Some(instance) = self.ctx.interner.get(typeid) {
            match instance {
                TypeDef::UserDef(defid) => {
                    let Some(def) = self.ctx.table.get_def(*defid) else {
                        unreachable!("defid not exists");
                    };

                    match &def.kind {
                        DefKind::Struct(def) => def.get_field(name).ok_or_else(|| {
                            {
                                format!(
                                    "no field \"{}\" on type {}",
                                    name,
                                    self.ctx.type_name(typeid)
                                )
                            }
                            .into()
                        }),
                        _ => Err(format!(
                            "no field \"{}\" on type {}",
                            name,
                            self.ctx.type_name(typeid)
                        )
                        .into()),
                    }
                }
                _ => Err(format!(
                    "no field \"{}\" on type {}",
                    name,
                    self.ctx.type_name(typeid)
                )
                .into()),
            }
        } else {
            unreachable!("TypeID {} not found in interner", typeid.0)
        }
    }
}
