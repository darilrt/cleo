use ast::{Literal, Operator};
use errors::Error;
use resolver::{context, symbols::DefKind};
use typed_ast::TypedUnit;
use types::TypeID;

use crate::ir::{BlockIR, ExprIR, FnIR, Intrinsic, RootIR, StmtIR};

pub struct UnitLowerer<'a> {
    ir: RootIR,
    ctx: &'a context::Context,
}

impl<'a> UnitLowerer<'a> {
    pub fn new(ctx: &'a context::Context) -> Self {
        Self {
            ir: RootIR::new(),
            ctx,
        }
    }

    pub fn lower_unit(mut self, unit: TypedUnit) -> Result<RootIR, Error> {
        for decl in unit.decls {
            if decl.no_emit {
                continue;
            }

            let unit_path =
                self.ctx.units.get_path(unit.scope).ok_or_else(|| {
                    format!("Could not find unit path for scope {}", unit.scope.0)
                })?;
            let fnir = FrameLowerer::lower_fn(decl, 0, unit_path.clone(), self.ctx)?;
            self.ir.add_fn(fnir);
        }

        Ok(self.ir)
    }
}

pub struct FrameLowerer<'a> {
    unit_path: String,
    stmts: Vec<StmtIR>,
    temps: u32,
    ctx: &'a context::Context,
}

impl<'a> FrameLowerer<'a> {
    fn new(ctx: &'a context::Context, unit_path: String, temps: u32) -> Self {
        Self {
            stmts: Vec::new(),
            unit_path,
            temps,
            ctx,
        }
    }

    pub fn make_temp(&mut self, typeid: TypeID) -> String {
        let temp_name = format!("tmp{}", self.temps);
        self.temps += 1;

        self.stmts
            .push(StmtIR::Local(temp_name.clone(), typeid, None));

        temp_name
    }

    pub fn make_temp_if_not_nothing(&mut self, typeid: TypeID) -> Option<String> {
        if typeid != self.ctx.primitives.nothing {
            Some(self.make_temp(typeid))
        } else {
            None
        }
    }

    pub fn lower_fn(
        fndecl: typed_ast::FnDecl,
        temps: u32,
        unit_path: String,
        ctx: &context::Context,
    ) -> Result<FnIR, Error> {
        if fndecl.body.is_none() {
            return Err(format!("expected a function body").into());
        }

        let body: typed_ast::Block = fndecl.body.unwrap();
        let is_nothing = body.typeid == ctx.primitives.nothing;

        let (_, block) = FrameLowerer::lower_block(
            body,
            if is_nothing {
                ReturnMethod::None
            } else {
                ReturnMethod::Return
            },
            temps,
            unit_path,
            ctx,
        )?;

        Ok(FnIR {
            name: fndecl.signature.name.string(),
            defid: fndecl.defid,
            scope: fndecl.scopeid,
            typeid: fndecl.typeid,
            block,
        })
    }

    fn lower_block(
        block: typed_ast::Block,
        return_method: ReturnMethod,
        temps: u32,
        unit_path: String,
        ctx: &context::Context,
    ) -> Result<(u32, BlockIR), Error> {
        let mut lowerer = FrameLowerer::new(ctx, unit_path, temps);
        let mut stmts = block.statements;

        let Some(last) = stmts.pop() else {
            return Ok((lowerer.temps, BlockIR { stmts: Vec::new() }));
        };

        for stmt in stmts {
            let stmt = lowerer.lower_stmt(stmt)?;
            lowerer.stmts.push(stmt);
        }

        match return_method {
            ReturnMethod::Var(var) => match last {
                typed_ast::Stmt::Expr(expr) => {
                    let expr_ir = lowerer.lower_expr(expr)?;
                    lowerer.stmts.push(StmtIR::Assign(var, expr_ir));
                }
                typed_ast::Stmt::Break(expr) => {
                    if let Some(expr) = expr {
                        let expr_ir = lowerer.lower_expr(expr)?;
                        lowerer.stmts.push(StmtIR::Assign(var, expr_ir));
                    }
                    lowerer.stmts.push(StmtIR::Break);
                }
                _ => unimplemented!("stmt"),
            },
            ReturnMethod::Return => match last {
                typed_ast::Stmt::Expr(expr) => {
                    let expr = lowerer.lower_expr(expr)?;
                    lowerer.stmts.push(StmtIR::Return(Some(expr)));
                }
                _ => unimplemented!(),
            },
            ReturnMethod::None => {
                let stmt = lowerer.lower_stmt(last)?;
                lowerer.stmts.push(stmt);
            }
        }

        Ok((
            lowerer.temps,
            BlockIR {
                stmts: lowerer.stmts,
            },
        ))
    }

    fn lower_stmt(&mut self, stmt: typed_ast::Stmt) -> Result<StmtIR, Error> {
        match stmt {
            typed_ast::Stmt::Expr(expr) => Ok(StmtIR::Expr(self.lower_expr(expr)?)),
            typed_ast::Stmt::Local(local) => Ok(self.lower_local(local)?),
            typed_ast::Stmt::Return(expr) => Ok(self.lower_return(expr)?),
            typed_ast::Stmt::Break(expr) => Ok(self.lower_break(expr)?),
            _ => unimplemented!("{:?}", stmt),
        }
    }

    fn lower_break(&mut self, expr: Option<typed_ast::Expr>) -> Result<StmtIR, Error> {
        if let Some(_expr) = expr {
            unimplemented!()
        }
        Ok(StmtIR::Break)
    }

    fn lower_return(&mut self, expr: Option<typed_ast::Expr>) -> Result<StmtIR, Error> {
        let expr = if let Some(expr) = expr {
            Some(self.lower_expr(expr)?)
        } else {
            None
        };

        Ok(StmtIR::Return(expr))
    }

    fn lower_local(&mut self, local: typed_ast::Local) -> Result<StmtIR, Error> {
        let initializer = if let Some(expr) = local.initializer {
            Some(self.lower_expr(expr)?)
        } else {
            None
        };

        Ok(StmtIR::Local(local.name, local.typeid, initializer))
    }

    fn lower_expr(&mut self, expr: typed_ast::Expr) -> Result<ExprIR, Error> {
        match expr {
            typed_ast::Expr::Value(value, _typeid) => Ok(ExprIR::Atom(match value {
                Literal::Bool(b) => b.to_string(),
                Literal::Integer {
                    value: raw,
                    suffix: _,
                } => raw.to_string(),
                Literal::Float(f) => f.to_string(),
                Literal::String(s) => s.clone(),
            })),
            typed_ast::Expr::BinaryOp {
                left,
                op,
                right,
                typeid,
            } => self.lower_binary_op(*left, op, *right, typeid),
            typed_ast::Expr::If(expr) => self.lower_if(expr),
            typed_ast::Expr::Assign(expr) => self.lower_assign(expr),
            typed_ast::Expr::Path(expr) => self.lower_path(expr),
            typed_ast::Expr::Call(expr) => self.lower_callexpr(expr),
            typed_ast::Expr::UnaryOp {
                op,
                expr,
                typeid: _,
            } => self.lower_unaryop(op, *expr),
            typed_ast::Expr::Access(expr) => self.lower_access(expr),
            typed_ast::Expr::Reinterpret { inner, from, to } => self.lower_cast(*inner, from, to),
            typed_ast::Expr::Loop(block) => self.lower_loop(block),
            typed_ast::Expr::ProcPath(proc) => self.lower_procpath(proc),
            _ => unimplemented!("expr {:?}", expr),
        }
    }

    fn lower_procpath(&mut self, proc: typed_ast::PorcPath) -> Result<ExprIR, Error> {
        let def = self.ctx.table.get_def(proc.defid).ok_or_else(|| {
            format!(
                "Could not find definition for proc path with defid {:?}",
                proc.defid
            )
        })?;

        let proc = match &def.kind {
            DefKind::Proc(proc) => proc,
            _ => {
                return Err(format!(
                    "Expected proc definition for proc path with defid {:?}, found {:?}",
                    proc.defid, def.kind
                )
                .into());
            }
        };

        Ok(ExprIR::Atom(def.get_mangled_name(&self.ctx)))
    }

    fn lower_path(&mut self, expr: typed_ast::PathExpr) -> Result<ExprIR, Error> {
        Ok(ExprIR::Atom(
            expr.segments
                .iter()
                .map(|s| s.name.str())
                .collect::<Vec<_>>()
                .join("."),
        ))
    }

    fn lower_loop(&mut self, block: typed_ast::Block) -> Result<ExprIR, Error> {
        let tmp = self.make_temp_if_not_nothing(block.typeid);

        let return_method = tmp
            .clone()
            .map_or(ReturnMethod::None, |var| ReturnMethod::Var(var));

        let (temps, block) = FrameLowerer::lower_block(
            block,
            return_method,
            self.temps,
            self.unit_path.clone(),
            self.ctx,
        )?;

        self.temps = temps;
        self.stmts.push(StmtIR::For(block));

        if let Some(tmp) = tmp {
            Ok(ExprIR::Atom(tmp))
        } else {
            Ok(ExprIR::Empty)
        }
    }

    fn lower_cast(
        &mut self,
        inner: typed_ast::Expr,
        from: TypeID,
        to: TypeID,
    ) -> Result<ExprIR, Error> {
        let inner = self.lower_expr(inner)?;

        Ok(ExprIR::Intrinsic(Intrinsic::Reinterpret {
            inner: Box::new(inner),
            from,
            to,
        }))
    }

    fn lower_access(&mut self, expr: typed_ast::ExprAccess) -> Result<ExprIR, Error> {
        let inner = self.lower_expr(*expr.inner)?;

        Ok(ExprIR::Access {
            inner: Box::new(inner),
            segment: expr.segment.name.string(),
        })
    }

    fn lower_unaryop(&mut self, op: Operator, expr: typed_ast::Expr) -> Result<ExprIR, Error> {
        Ok(ExprIR::UnaryOp {
            op: match op {
                Operator::Ref => "&".to_string(),
                Operator::Deref => "*".to_string(),
                _ => Err(format!("Invalid unary operator '{:?}'", op))?,
            },
            expr: Box::new(self.lower_expr(expr)?),
        })
    }

    fn lower_assign(&mut self, expr: typed_ast::ExprAssign) -> Result<ExprIR, Error> {
        let left = self.lower_expr(*expr.left)?;
        let right = self.lower_expr(*expr.right)?;

        Ok(ExprIR::Assign {
            left: Box::new(left),
            kind: expr.kind,
            right: Box::new(right),
        })
    }

    fn lower_callexpr(&mut self, expr: typed_ast::ExprCall) -> Result<ExprIR, Error> {
        let callee = self.lower_expr(*expr.callee)?;
        let args = expr
            .args
            .into_iter()
            .map(|arg| self.lower_expr(arg))
            .collect::<Result<Vec<_>, Error>>()?;

        Ok(ExprIR::Call {
            callee: Box::new(callee),
            args,
        })
    }

    fn lower_binary_op(
        &mut self,
        left: typed_ast::Expr,
        op: Operator,
        right: typed_ast::Expr,
        _typeid: TypeID,
    ) -> Result<ExprIR, Error> {
        let left_expr = self.lower_expr(left)?;
        let right_expr = self.lower_expr(right)?;

        let op_str = match op {
            Operator::Add => "+",
            Operator::Sub => "-",
            Operator::Mul => "*",
            Operator::Div => "/",
            Operator::Mod => "%",
            Operator::Greater => ">",
            Operator::GreaterEqual => ">=",
            Operator::Less => "<",
            Operator::LessEqual => "<=",
            _ => panic!("unimplemented operator {:?}", op),
        };

        Ok(ExprIR::BinaryOp {
            left: Box::new(left_expr),
            op: op_str.to_string(),
            right: Box::new(right_expr),
        })
    }

    fn lower_if(&mut self, expr: typed_ast::ExprIf) -> Result<ExprIR, Error> {
        let condition_expr = self.lower_expr(*expr.condition)?;
        let tmp = self.make_temp_if_not_nothing(expr.then_branch.typeid);
        let return_method = tmp
            .clone()
            .map_or(ReturnMethod::None, |var| ReturnMethod::Var(var));

        let (temps, if_block) = FrameLowerer::lower_block(
            expr.then_branch,
            return_method.clone(),
            self.temps,
            self.unit_path.clone(),
            self.ctx,
        )?;

        let else_block = expr
            .else_branch
            .and_then(|branch| {
                FrameLowerer::lower_block(
                    branch,
                    return_method,
                    temps,
                    self.unit_path.clone(),
                    self.ctx,
                )
                .ok()
            })
            .map(|(temps, block)| {
                self.temps = temps;
                block
            });

        self.temps = temps;

        self.stmts
            .push(StmtIR::If(condition_expr, if_block, else_block));

        Ok(if let Some(tmp) = tmp {
            ExprIR::Atom(tmp)
        } else {
            ExprIR::Empty
        })
    }
}

pub fn lower(unit: TypedUnit, ctx: &context::Context) -> Result<RootIR, Error> {
    let lowerer = UnitLowerer::new(ctx);

    lowerer.lower_unit(unit)
}

#[derive(Clone)]
enum ReturnMethod {
    None,
    Return,
    Var(String),
}
