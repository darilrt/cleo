use ast::{ExprValue, Operator};
use errors::Error;
use resolver::context;
use typed_ast::TypedUnit;
use types::TypeID;

use crate::ir::{BlockIR, ExprIR, FnIR, StmtIR, UnitIR};

pub struct UnitLowerer<'a> {
    ir: UnitIR,
    ctx: &'a context::Context,
}

impl<'a> UnitLowerer<'a> {
    pub fn new(ctx: &'a context::Context) -> Self {
        Self {
            ir: UnitIR::new(),
            ctx,
        }
    }

    pub fn lower_unit(mut self, unit: TypedUnit) -> Result<UnitIR, Error> {
        for decl in unit.decls {
            let fnir = FrameLowerer::lower_fn(decl, 0, self.ctx)?;
            self.ir.add_fn(fnir);
        }

        Ok(self.ir)
    }
}

pub struct FrameLowerer<'a> {
    stmts: Vec<StmtIR>,
    temps: u32,
    ctx: &'a context::Context,
}

impl<'a> FrameLowerer<'a> {
    fn new(temps: u32, ctx: &'a context::Context) -> Self {
        Self {
            stmts: Vec::new(),
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

    pub fn lower_fn(
        fndecl: typed_ast::FnDecl,
        temps: u32,
        ctx: &context::Context,
    ) -> Result<FnIR, Error> {
        let (_, block) = FrameLowerer::lower_block(fndecl.block, None, temps, ctx)?;

        Ok(FnIR {
            name: fndecl.signature.name.string(),
            defid: fndecl.defid,
            scope: fndecl.scopeid,
            typeid: fndecl.typeid,
            block,
        })
    }

    pub fn lower_block(
        block: typed_ast::Block,
        ret_var: Option<String>,
        temps: u32,
        ctx: &context::Context,
    ) -> Result<(u32, BlockIR), Error> {
        let mut lowerer = FrameLowerer::new(temps, ctx);
        let mut stmts = block.statements;

        let Some(last) = stmts.pop() else {
            return Ok((lowerer.temps, BlockIR { stmts: Vec::new() }));
        };

        for stmt in stmts {
            let stmt = lowerer.lower_stmt(stmt)?;
            lowerer.stmts.push(stmt);
        }

        match ret_var {
            Some(var) => match last {
                typed_ast::Stmt::Expr(expr) => {
                    let expr_ir = lowerer.lower_expr(expr)?;
                    lowerer.stmts.push(StmtIR::Assign(var, expr_ir));
                }
                _ => unimplemented!("stmt"),
            },
            None => {
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
            _ => unimplemented!("stmt"),
        }
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
                ExprValue::Bool(b) => b.to_string(),
                ExprValue::Integer(i) => i.to_string(),
                ExprValue::Float(f) => f.to_string(),
                ExprValue::String(s) => s.clone(),
            })),
            typed_ast::Expr::BinaryOp {
                left,
                op,
                right,
                typeid,
            } => self.lower_binary_op(*left, op, *right, typeid),
            typed_ast::Expr::If(expr) => self.lower_if(expr),
            typed_ast::Expr::Assign(expr) => self.lower_assign(expr),
            typed_ast::Expr::Path(expr) => Ok(ExprIR::Atom(
                expr.segments
                    .iter()
                    .map(|s| s.name.str())
                    .collect::<Vec<_>>()
                    .join("."),
            )),
            typed_ast::Expr::Call(expr) => self.lower_callexpr(expr),
            _ => unimplemented!("expr"),
        }
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
            _ => unimplemented!(""),
        };

        Ok(ExprIR::BinaryOp {
            left: Box::new(left_expr),
            op: op_str.to_string(),
            right: Box::new(right_expr),
        })
    }

    fn lower_if(&mut self, expr: typed_ast::ExprIf) -> Result<ExprIR, Error> {
        let condition_expr = self.lower_expr(*expr.condition)?;
        let tmp: String = self.make_temp(expr.then_branch.typeid);

        let (temps, if_block) =
            FrameLowerer::lower_block(expr.then_branch, Some(tmp.clone()), self.temps, self.ctx)?;

        let else_block = expr
            .else_branch
            .and_then(|branch| {
                FrameLowerer::lower_block(branch, Some(tmp.clone()), temps, self.ctx).ok()
            })
            .map(|(temps, block)| {
                self.temps = temps;
                block
            });

        self.temps = temps;

        self.stmts
            .push(StmtIR::If(condition_expr, if_block, else_block));

        Ok(ExprIR::Atom(tmp))
    }
}

pub fn lower(unit: TypedUnit, ctx: &context::Context) -> Result<UnitIR, Error> {
    let lowerer = UnitLowerer::new(ctx);

    lowerer.lower_unit(unit)
}
