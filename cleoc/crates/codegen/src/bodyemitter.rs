use std::io;

use errors::Error;
use resolver::context::Context;
use typed_ast::Block;
use types::ScopeID;

use crate::{
    codegen::emit_type,
    ir::{ExprIR, FnIR, StmtIR},
};

#[allow(unused)]
pub struct BodyEmitter<'a, 's> {
    ctx: &'a Context,
    ir: &'s FnIR,
    indent: u32,
}

impl<'a, 's> BodyEmitter<'a, 's> {
    pub fn new(ctx: &'a Context, ir: &'s FnIR) -> Self {
        Self { ctx, ir, indent: 1 }
    }

    pub fn emit(&mut self, buffer: &mut impl io::Write, _scope: ScopeID) -> Result<(), Error> {
        for stmt in self.ir.body.iter() {
            emit_identation(buffer, self.indent)?;

            match stmt {
                StmtIR::Expr(expr) => self.emit_expr(buffer, expr),
                StmtIR::Local(label, typeid) => {
                    emit_type(self.ctx, buffer, *typeid, label)?;
                    Ok(())
                }
                StmtIR::Assign(label, expr) => {
                    write!(buffer, "{} = ", label)?;
                    self.emit_expr(buffer, expr)
                }
                StmtIR::If(expr, if_block, else_block) => {
                    self.emit_if(buffer, expr, if_block, else_block.as_ref())
                }
            }?;

            writeln!(buffer, ";")?;
        }

        Ok(())
    }

    fn emit_expr(&self, buffer: &mut impl io::Write, expr: &ExprIR) -> Result<(), Error> {
        match expr {
            ExprIR::Atom(value) => write!(buffer, "{}", value)?,
            ExprIR::BinaryOp { left, op, right } => {
                write!(buffer, "(")?;
                self.emit_expr(buffer, left)?;
                write!(buffer, " {} ", op)?;
                self.emit_expr(buffer, right)?;
                write!(buffer, ")")?;
            }
            ExprIR::Lit(expr) => {
                self.emit_expr(buffer, expr)?;
            }
        }

        Ok(())
    }

    fn emit_if(
        &self,
        buffer: &mut impl io::Write,
        expr: &ExprIR,
        _if_block: &Block,
        _else_block: Option<&Block>,
    ) -> Result<(), Error> {
        write!(buffer, "if (")?;
        self.emit_expr(buffer, expr)?;
        writeln!(buffer, ") {{ }}")?;

        Ok(())
    }
}

fn emit_identation(buffer: &mut impl io::Write, identation: u32) -> Result<(), Error> {
    for _ in 0..identation {
        write!(buffer, "  ")?;
    }

    Ok(())
}
