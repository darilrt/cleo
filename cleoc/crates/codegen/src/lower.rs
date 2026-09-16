use ast::{Expr, ExprValue, Operator};
use errors::Error;
use typed_ast::TypedUnit;
use types::{ScopeID, TypeID};

use crate::ir::{ExprIR, FnIR, StmtIR, UnitIR};

pub struct UnitLowerer<'a> {
    scope: ScopeID,
    unit: &'a TypedUnit,
    ir: UnitIR,
}

impl<'a> UnitLowerer<'a> {
    pub fn new(scope: ScopeID, unit: &'a TypedUnit) -> Self {
        Self {
            scope,
            unit,
            ir: UnitIR::new(scope),
        }
    }

    pub fn lower_unit(mut self) -> Result<UnitIR, Error> {
        for decl in &self.unit.decls {
            let fnir = FnLowerer::new(self.scope, decl).lower_fn()?;
            self.ir.add_fn(fnir);
        }

        Ok(self.ir)
    }
}

pub struct FnLowerer<'a> {
    scope: ScopeID,
    decl: &'a typed_ast::FnDecl,
    body: Vec<StmtIR>,
    temps: u32,
}

impl<'a> FnLowerer<'a> {
    pub fn new(scope: ScopeID, decl: &'a typed_ast::FnDecl) -> Self {
        Self {
            scope,
            decl,
            body: Vec::new(),
            temps: 0,
        }
    }

    pub fn make_temp(&mut self, typeid: TypeID) -> String {
        let temp_name = format!("tmp{}", self.temps);
        self.temps += 1;

        self.body.push(StmtIR::Local(temp_name.clone(), typeid));

        temp_name
    }

    pub fn lower_fn(mut self) -> Result<FnIR, Error> {
        for stmt in &self.decl.block.statements {
            let stmt = self.lower_stmt(stmt)?;
            self.body.push(stmt);
        }

        Ok(FnIR {
            name: self.decl.signature.name.string(),
            defid: self.decl.defid,
            scope: self.decl.scopeid,
            typeid: self.decl.typeid,
            body: self.body,
        })
    }

    fn lower_stmt(&mut self, stmt: &typed_ast::Stmt) -> Result<StmtIR, Error> {
        match stmt {
            typed_ast::Stmt::Expr(expr) => Ok(StmtIR::Expr(self.lower_expr(expr)?)),
            _ => unimplemented!("stmt"),
        }
    }

    fn lower_expr(&mut self, expr: &typed_ast::Expr) -> Result<ExprIR, Error> {
        match expr {
            typed_ast::Expr::Value(value, _typeid) => {
                Ok(ExprIR::Lit(Box::new(ExprIR::Atom(match value {
                    ExprValue::Bool(b) => b.to_string(),
                    ExprValue::Integer(i) => i.to_string(),
                    ExprValue::Float(f) => f.to_string(),
                    ExprValue::String(s) => s.clone(),
                }))))
            }
            typed_ast::Expr::BinaryOp {
                left,
                op,
                right,
                typeid,
            } => self.lower_binary_op(left, op, right, *typeid),
            typed_ast::Expr::If(expr) => self.lower_if(expr),
            _ => unimplemented!("expr"),
        }
    }

    fn lower_binary_op(
        &mut self,
        left: &typed_ast::Expr,
        op: &Operator,
        right: &typed_ast::Expr,
        typeid: TypeID,
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

        match (left_expr, right_expr) {
            (ExprIR::Lit(lhs), ExprIR::Lit(rhs)) => Ok(ExprIR::Lit(Box::new(ExprIR::BinaryOp {
                left: lhs,
                op: op_str.to_string(),
                right: rhs,
            }))),
            (left_expr, right_expr) => {
                let lhs_tmp = self.make_temp(typeid);
                self.body.push(StmtIR::Assign(lhs_tmp.clone(), left_expr));

                let rhs_tmp = self.make_temp(typeid);
                self.body.push(StmtIR::Assign(rhs_tmp.clone(), right_expr));

                Ok(ExprIR::BinaryOp {
                    left: Box::new(ExprIR::Atom(lhs_tmp)),
                    op: op_str.to_string(),
                    right: Box::new(ExprIR::Atom(rhs_tmp)),
                })
            }
        }
    }

    fn lower_if(&mut self, expr: &typed_ast::ExprIf) -> Result<ExprIR, Error> {
        let condition_expr = self.lower_expr(&expr.condition)?;

        Ok(ExprIR::)
    }
}

pub fn lower(scope: ScopeID, unit: &TypedUnit) -> Result<UnitIR, Error> {
    let lowerer = UnitLowerer::new(scope, unit);

    lowerer.lower_unit()
}
