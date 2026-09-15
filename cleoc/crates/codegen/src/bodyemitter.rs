use std::io;

use errors::Error;
use resolver::context::Context;
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
            }?;

            writeln!(buffer, ";")?;
        }

        Ok(())
    }

    fn emit_expr(&self, buffer: &mut impl io::Write, expr: &ExprIR) -> Result<(), Error> {
        match expr {
            ExprIR::Value(value) => write!(buffer, "{}", value)?,
            // _ => {}
        }

        Ok(())
    }
}

fn emit_identation(buffer: &mut impl io::Write, identation: u32) -> Result<(), Error> {
    for _ in 0..identation {
        write!(buffer, "  ")?;
    }

    Ok(())
}
