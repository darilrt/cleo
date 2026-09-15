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
            typed_ast::Expr::Value(value, _typeid) => Ok(ExprIR::Value(match value {
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
            } => self.lower_binary_op(left, op, right, *typeid),
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
        let left_str = match left {
            typed_ast::Expr::Value(value, _typeid) => match value {
                ExprValue::Bool(b) => b.to_string(),
                ExprValue::Integer(i) => i.to_string(),
                ExprValue::Float(f) => f.to_string(),
                ExprValue::String(s) => s.clone(),
            },
            _ => {
                let temp_name = self.make_temp(typeid);
                let expr_ir = self.lower_expr(left)?;
                self.body.push(StmtIR::Assign(temp_name.clone(), expr_ir));
                temp_name
            }
        };

        let right_str = match right {
            typed_ast::Expr::Value(value, _typeid) => match value {
                ExprValue::Bool(b) => b.to_string(),
                ExprValue::Integer(i) => i.to_string(),
                ExprValue::Float(f) => f.to_string(),
                ExprValue::String(s) => s.clone(),
            },
            _ => {
                let temp_name = self.make_temp(typeid);
                let expr_ir = self.lower_expr(right)?;
                self.body.push(StmtIR::Assign(temp_name.clone(), expr_ir));
                temp_name
            }
        };

        let op_str = match op {
            Operator::Add => "+",
            Operator::Sub => "-",
            Operator::Mul => "*",
            Operator::Div => "/",
            _ => unimplemented!(""),
        };

        Ok(ExprIR::Value(format!(
            "{} {} {}",
            left_str, op_str, right_str
        )))
    }
}

pub fn lower(scope: ScopeID, unit: &TypedUnit) -> Result<UnitIR, Error> {
    let lowerer = UnitLowerer::new(scope, unit);

    lowerer.lower_unit()
}
