use ast::{
    Block, Decl, Expr, ExprAccess, ExprAssign, ExprCall, ExprIf, FnDecl, IntegerSuffix, Literal,
    Local, Operator, PathExpr, Segment, Stmt, Unit,
};
use errors::Error;
use typed_ast::TypedUnit;
use types::{DefID, ScopeID, TypeID, defs::TypeDef};

use crate::{
    context::Context,
    symbols::{DefKind, Definition},
};

pub struct BlockCtx {
    #[allow(unused)]
    allow_break: bool,

    #[allow(unused)]
    allow_continue: bool,

    #[allow(unused)]
    expected_return: TypeID,
}

pub struct Checker<'a> {
    ctx: &'a mut Context,
}

pub enum Resolved {
    Value(TypeID),
    Type(TypeID, DefID),
    EnumValue(TypeID),
    Module(ScopeID),
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
                    if let Some(f) = self.resolve_fn(scope, decl)? {
                        out.decls.push(f);
                    }
                }
                _ => {}
            }
        }

        Ok(out)
    }

    pub fn resolve_fn(
        &mut self,
        scope: ScopeID,
        decl: FnDecl,
    ) -> Result<Option<typed_ast::FnDecl>, Error> {
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

        let scopeid = self.ctx.table.push(scope)?;

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
                    .define(scopeid, Definition::var(param_name, typeid.clone()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        if decl.has_attr("extern") {
            Ok(None)
        } else {
            let expected_return = fntype.return_type;
            let no_emit = decl.has_attr("no_emit");
            let body = self.resolve_block(
                scopeid,
                decl.block
                    .ok_or("expected a function body".to_string())?
                    .statements,
                &BlockCtx {
                    allow_break: false,
                    allow_continue: false,
                    expected_return: expected_return,
                },
            )?;

            let fndecl = typed_ast::FnDecl {
                no_emit,
                signature: typed_ast::FnSignature {
                    name: decl.signature.name,
                    params: params_defid,
                },
                body: Some(body),
                scopeid,
                typeid,
                defid,
            };

            Ok(Some(fndecl))
        }
    }

    pub fn resolve_block(
        &mut self,
        scope: ScopeID,
        stmts: Vec<Stmt>,
        block_ctx: &BlockCtx,
    ) -> Result<typed_ast::Block, Error> {
        let statements = stmts
            .into_iter()
            .map(|stmt| self.resolve_stmt(scope, stmt, block_ctx))
            .collect::<Result<Vec<_>, Error>>()?;

        let Some(last) = statements.last() else {
            return Ok(typed_ast::Block {
                statements: Vec::new(),
                typeid: self.ctx.primitives.nothing,
            });
        };

        let typeid = match last {
            typed_ast::Stmt::Expr(expr) => expr.type_id(),
            _ => self.ctx.primitives.nothing,
        };

        if block_ctx.expected_return != self.ctx.primitives.nothing
            && !self.ctx.is_coercible(typeid, block_ctx.expected_return)
        {
            Err(format!(
                "mismatched return types expected {}, found {}",
                self.ctx.type_name(block_ctx.expected_return),
                self.ctx.type_name(typeid)
            )
            .into())
        } else {
            Ok(typed_ast::Block {
                statements,
                typeid: block_ctx.expected_return,
            })
        }
    }

    pub fn resolve_stmt(
        &mut self,
        scope: ScopeID,
        stmt: Stmt,
        block_ctx: &BlockCtx,
    ) -> Result<typed_ast::Stmt, Error> {
        match stmt {
            Stmt::Break => {
                if !block_ctx.allow_break {
                    return Err("Not allowed Break here".to_string().into());
                }

                Ok(typed_ast::Stmt::Break(None))
            }
            Stmt::Continue => {
                if !block_ctx.allow_continue {
                    return Err("Not allowed Continue here".to_string().into());
                }

                unimplemented!("");
            }
            Stmt::Return(expr) => self.resolve_return(scope, expr, block_ctx),
            Stmt::Defer(expr) => Ok(typed_ast::Stmt::Defer(
                self.resolve_expr(scope, expr, block_ctx)?,
            )),
            Stmt::Expr(expr) => Ok(typed_ast::Stmt::Expr(
                self.resolve_expr(scope, expr, block_ctx)?,
            )),
            Stmt::Local(local) => Ok(typed_ast::Stmt::Local(
                self.resolve_local(scope, local, block_ctx)?,
            )),
        }
    }

    pub fn resolve_return(
        &mut self,
        scope: ScopeID,
        expr: Option<Expr>,
        block_ctx: &BlockCtx,
    ) -> Result<typed_ast::Stmt, Error> {
        match (block_ctx.expected_return, expr) {
            (expected, None) => {
                if expected == self.ctx.primitives.nothing {
                    Ok(typed_ast::Stmt::Return(None))
                } else {
                    Err(format!(
                        "expected return value of type {}",
                        self.ctx.type_name(expected)
                    )
                    .into())
                }
            }
            (expected, Some(expr)) => {
                let return_expr = self.resolve_expr(scope, expr, block_ctx)?;

                if self.ctx.is_coercible(return_expr.type_id(), expected) {
                    Ok(typed_ast::Stmt::Return(Some(return_expr)))
                } else {
                    Err(format!(
                        "expected return type {}, found {}",
                        self.ctx.type_name(expected),
                        self.ctx.type_name(return_expr.type_id())
                    )
                    .into())
                }
            }
        }
    }

    pub fn resolve_expr(
        &mut self,
        scope: ScopeID,
        expr: Expr,
        block_ctx: &BlockCtx,
    ) -> Result<typed_ast::Expr, Error> {
        match expr {
            Expr::Value(value) => {
                let typeid = match &value {
                    Literal::Integer { value: _, suffix } => match suffix {
                        None => self.ctx.primitives.i32_,
                        Some(IntegerSuffix::U8) => self.ctx.primitives.u8_,
                        Some(IntegerSuffix::U16) => self.ctx.primitives.u16_,
                        Some(IntegerSuffix::U32) => self.ctx.primitives.u32_,
                        Some(IntegerSuffix::U64) => self.ctx.primitives.u64_,
                        Some(IntegerSuffix::I8) => self.ctx.primitives.i8_,
                        Some(IntegerSuffix::I16) => self.ctx.primitives.i16_,
                        Some(IntegerSuffix::I32) => self.ctx.primitives.i32_,
                        Some(IntegerSuffix::I64) => self.ctx.primitives.i64_,
                    },
                    Literal::Float(_literal) => self.ctx.primitives.f32_,
                    Literal::Bool(_value) => self.ctx.primitives.bool_,
                    Literal::String(lit) => self.ctx.interner.intern(TypeDef::Array {
                        element: self.ctx.primitives.u8_,
                        size: lit.len(),
                    }),
                    // ExprValue::String(_lit) => self.ctx.interner.intern(TypeDef::Pointer {
                    //     pointee: self.ctx.primitives.u8_,
                    //     mutability: false,
                    // }),
                };
                Ok(typed_ast::Expr::Value(value, typeid))
            }
            Expr::BinaryOp { left, op, right } => {
                self.resolve_binaryop(scope, *left, op, *right, block_ctx)
            }
            Expr::If(expr) => self.resolve_if(scope, expr, block_ctx),
            Expr::Assign(expr) => self.resolve_assign(scope, expr, block_ctx),
            Expr::Path(path) => {
                let (segments, resolved) = self.resolve_pathexpr(scope, path)?;
                Ok(typed_ast::Expr::Path(typed_ast::PathExpr {
                    segments,
                    typeid: match resolved {
                        Resolved::Value(typeid) => typeid,
                        Resolved::EnumValue(typeid) => typeid,
                        _ => return Err(format!("Expected value expression").into()),
                    },
                }))
            }
            Expr::UnaryOp { op, expr } => self.resolve_unaryop(scope, op, *expr, block_ctx),
            Expr::Call(expr) => self.resolve_callexpr_or_intrinsic(scope, expr, block_ctx),
            Expr::Access(expr) => self.resolve_access(scope, expr, block_ctx),
            Expr::Loop(block) => self.resolve_loop(scope, block, block_ctx),
            _ => {
                unimplemented!("Expression resolution not implemented for {:?}", expr)
            } // Expr::Init(expr) => self.resolve_init(scope, expr, block_ctx),
              // Expr::Loop(block) => self.resolve_loop(scope, block, block_ctx),
        }
    }

    pub fn resolve_loop(
        &mut self,
        scope: ScopeID,
        block: Block,
        block_ctx: &BlockCtx,
    ) -> Result<typed_ast::Expr, Error> {
        let block = self.resolve_block(
            scope,
            block.statements,
            &BlockCtx {
                allow_break: true,
                allow_continue: true,
                expected_return: block_ctx.expected_return,
            },
        )?;
        Ok(typed_ast::Expr::Loop(block))
    }

    pub fn resolve_callexpr_or_intrinsic(
        &mut self,
        scope: ScopeID,
        expr: ExprCall,
        block_ctx: &BlockCtx,
    ) -> Result<typed_ast::Expr, Error> {
        if self.is_intrinsic(&expr.callee) {
            self.resolve_intrinsic_call(scope, expr, block_ctx)
        } else {
            self.resolve_callexpr(scope, expr, block_ctx)
        }
    }

    pub fn resolve_callexpr(
        &mut self,
        scope: ScopeID,
        expr: ExprCall,
        block_ctx: &BlockCtx,
    ) -> Result<typed_ast::Expr, Error> {
        let fnexpr = self.resolve_expr(scope, *expr.callee, block_ctx)?;

        let (params, return_typeid) = {
            let TypeDef::FnPointer(fn_type) = self
                .ctx
                .interner
                .get(fnexpr.type_id())
                .ok_or_else(|| format!("type {} does not exists", fnexpr.type_id().0))?
            else {
                return Err(format!(
                    "type {} is not callable",
                    self.ctx.type_name(fnexpr.type_id())
                )
                .into());
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

        let args = expr
            .args
            .into_iter()
            .map(|arg| self.resolve_expr(scope, arg, block_ctx))
            .collect::<Result<Vec<_>, Error>>()?;

        for (param, arg) in params.iter().zip(&args) {
            if !self.ctx.is_coercible(arg.type_id(), *param) {
                return Err(format!(
                    "argument type error: expected {}, found {}",
                    self.ctx.type_name(*param),
                    self.ctx.type_name(arg.type_id())
                )
                .into());
            }
        }

        Ok(typed_ast::Expr::Call(typed_ast::ExprCall {
            typeid: return_typeid,
            args,
            callee: fnexpr.into(),
        }))
    }

    fn is_intrinsic(&mut self, callee: &Expr) -> bool {
        match callee {
            Expr::Path(expr) => {
                if expr.segments.len() == 1 {
                    expr.segments[0].name.str() == "cast"
                } else {
                    false
                }
            }
            _ => return false,
        }
    }

    fn resolve_intrinsic_call(
        &mut self,
        scope: ScopeID,
        expr: ExprCall,
        block_ctx: &BlockCtx,
    ) -> Result<typed_ast::Expr, Error> {
        let ExprCall { callee, args } = expr;
        let Expr::Path(callee) = *callee else {
            return Err("intrinsic call must be a path".to_string().into());
        };

        let PathExpr { mut segments } = callee;

        if segments.len() != 1 {
            return Err("intrinsic call must be a single segment".to_string().into());
        }
        let segment = segments.remove(0);
        let name = segment.name.str();

        if name != "cast" {
            return Err(format!("Unknown intrinsic '{}'", name).into());
        }

        if args.len() != 1 {
            return Err("cast expects exactly 1 argument".to_string().into());
        }
        let value = self.resolve_expr(scope, args[0].clone(), block_ctx)?;

        let Some(generics) = segment.generics else {
            return Err("cast expects exactly 1 geneirc argument".to_string().into());
        };

        if generics.len() != 1 {
            return Err("cast expects exactly 1 geneirc argument".to_string().into());
        }
        let generic = self.ctx.resolve_id(scope, &generics[0])?;

        let from = value.type_id();

        Ok(typed_ast::Expr::Reinterpret {
            inner: Box::new(value),
            from,
            to: generic,
        })
    }

    pub fn resolve_binaryop(
        &mut self,
        scope: ScopeID,
        left: Expr,
        op: Operator,
        right: Expr,
        block_ctx: &BlockCtx,
    ) -> Result<typed_ast::Expr, Error> {
        let l_expr = self.resolve_expr(scope, left, block_ctx)?;
        let r_expr = self.resolve_expr(scope, right, block_ctx)?;

        let l = l_expr.type_id();
        let r = r_expr.type_id();

        let typeid: Result<TypeID, Error> = match op {
            Operator::Add | Operator::Sub | Operator::Mul | Operator::Div | Operator::Mod => {
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
        };

        Ok(typed_ast::Expr::BinaryOp {
            left: Box::new(l_expr),
            op: op,
            right: Box::new(r_expr),
            typeid: typeid?,
        })
    }

    pub fn resolve_if(
        &mut self,
        scope: ScopeID,
        expr: ExprIf,
        block_ctx: &BlockCtx,
    ) -> Result<typed_ast::Expr, Error> {
        let condition = self.resolve_expr(scope, *expr.condition, block_ctx)?;

        if !self.ctx.primitives.is_bool(condition.type_id()) {
            return Err(format!(
                "expected bool in if condition, found {}",
                self.ctx.type_name(condition.type_id())
            )
            .into());
        }

        let then_return = {
            let scope = self.ctx.table.push(scope)?;
            self.resolve_block(scope, expr.then_branch.statements, block_ctx)?
        };

        let else_return = if let Some(else_branch) = expr.else_branch {
            let scope = self.ctx.table.push(scope)?;
            Some(self.resolve_block(scope, else_branch.statements, block_ctx)?)
        } else {
            None
        };

        Ok(typed_ast::Expr::If(typed_ast::ExprIf {
            condition: Box::new(condition),
            then_branch: then_return,
            else_branch: else_return,
        }))
    }

    pub fn resolve_local(
        &mut self,
        scope: ScopeID,
        local: Local,
        block_ctx: &BlockCtx,
    ) -> Result<typed_ast::Local, Error> {
        let typeid = self.ctx.resolve_id(
            scope,
            local
                .var_type
                .as_ref()
                .unwrap_or_else(|| todo!("Implemente Infered types")),
        )?;

        let expr = local
            .initializer
            .map(|expr| {
                let expr = self.resolve_expr(scope, *expr, block_ctx)?;
                let exprid = expr.type_id();

                if !self.ctx.is_coercible(exprid, typeid) {
                    return Err(Error::from(format!(
                        "assignment error: expected {}, found {}",
                        self.ctx.type_name(typeid),
                        self.ctx.type_name(exprid)
                    )));
                }

                Ok(expr)
            })
            .transpose()?;

        let _ = self
            .ctx
            .table
            .define(scope, Definition::var(local.name.str(), typeid))?;

        Ok(typed_ast::Local {
            name: local.name.string(),
            typeid,
            initializer: expr,
        })
    }

    pub fn resolve_assign(
        &mut self,
        scope: ScopeID,
        expr: ExprAssign,
        block_ctx: &BlockCtx,
    ) -> Result<typed_ast::Expr, Error> {
        if !self.is_lvalue(&*expr.left) {
            return Err("invalid left-hand side of assignment".to_string().into());
        }

        let left = self.resolve_expr(scope, *expr.left, block_ctx)?;
        let right = self.resolve_expr(scope, *expr.right, block_ctx)?;

        if !self.ctx.is_coercible(right.type_id(), left.type_id()) {
            return Err(format!(
                "expected {}, found {}",
                self.ctx.type_name(left.type_id()),
                self.ctx.type_name(right.type_id())
            )
            .into());
        }

        Ok(typed_ast::Expr::Assign(typed_ast::ExprAssign {
            left: Box::new(left),
            kind: expr.kind,
            right: Box::new(right),
        }))
    }

    pub fn is_lvalue(&mut self, expr: &Expr) -> bool {
        match expr {
            Expr::UnaryOp {
                op: Operator::Deref,
                ..
            }
            | Expr::Path(_)
            | Expr::Access(_) => true,
            _ => false,
        }
    }

    pub fn resolve_pathexpr(
        &mut self,
        scope: ScopeID,
        path: PathExpr,
    ) -> Result<(Vec<Segment>, Resolved), Error> {
        // TODO: Revisar esta implementacion para resolver PathExpressions y devovler un TypedUnit
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

        Ok((path.segments, current))
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

    fn def_to_resolved(&self, def: &Definition, defid: DefID) -> Resolved {
        match &def.kind {
            DefKind::Variable { typeid } => Resolved::Value(*typeid),
            DefKind::Function(sig) => Resolved::Value(sig.typeid),
            DefKind::Struct(s) => Resolved::Type(s.typeid, defid),
            DefKind::Enum(e) => Resolved::Type(e.typeid, defid),
            DefKind::Trait(t) => Resolved::Type(t.typeid, defid),
            DefKind::TypeAlias(a) => Resolved::Type(a.typeid, defid),
            DefKind::Module { scope } => Resolved::Module(*scope),
        }
    }

    pub fn resolve_unaryop(
        &mut self,
        scope: ScopeID,
        op: Operator,
        expr: Expr,
        block_ctx: &BlockCtx,
    ) -> Result<typed_ast::Expr, Error> {
        let expr = self.resolve_expr(scope, expr, block_ctx)?;
        let typeid = expr.type_id();

        match op {
            // Operator::Neg => {
            //     if self.ctx.primitives.is_negatable(typeid) {
            //         Ok(typeid)
            //     } else {
            //         Err(format!("cannot negate {}", self.ctx.type_name(typeid)).into())
            //     }
            // }
            // Operator::Not => {
            //     if self.ctx.primitives.is_bool(typeid) {
            //         Ok(typeid)
            //     } else {
            //         Err(format!("expected bool, found {}", self.ctx.type_name(typeid)).into())
            //     }
            // }
            Operator::Deref => match self
                .ctx
                .interner
                .get(typeid)
                .ok_or_else(|| format!("type {} does not exists", typeid.0))?
            {
                TypeDef::Pointer { pointee } => Ok(typed_ast::Expr::UnaryOp {
                    op,
                    expr: Box::new(expr),
                    typeid: *pointee,
                }),
                _ => Err(format!("cannot dereference {}", self.ctx.type_name(typeid)).into()),
            },
            Operator::Ref => match self
                .ctx
                .interner
                .get(typeid)
                .ok_or_else(|| format!("type {} does not exists", typeid.0))?
            {
                TypeDef::FnPointer(_) => Ok(typed_ast::Expr::UnaryOp {
                    op,
                    expr: Box::new(expr),
                    typeid: typeid,
                }),
                _ => {
                    let typeid = self
                        .ctx
                        .interner
                        .intern(TypeDef::Pointer { pointee: typeid });

                    Ok(typed_ast::Expr::UnaryOp {
                        op,
                        expr: Box::new(expr),
                        typeid: typeid,
                    })
                }
            },
            _ => unreachable!("not a unary operator"),
        }
    }

    pub fn resolve_access(
        &mut self,
        scope: ScopeID,
        expr: ExprAccess,
        block_ctx: &BlockCtx,
    ) -> Result<typed_ast::Expr, Error> {
        let inner = self.resolve_expr(scope, *expr.inner, block_ctx)?;
        let base = Resolved::Value(inner.type_id());

        match self.resolve_next(base, expr.segment.name.str())? {
            Resolved::Value(typeid) => Ok(typed_ast::Expr::Access(typed_ast::ExprAccess {
                inner: Box::new(inner),
                segment: expr.segment,
                typeid: typeid,
            })),
            _ => unreachable!("a type could not be reachable from a instance"),
        }
    }

    /*
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


    */
}
