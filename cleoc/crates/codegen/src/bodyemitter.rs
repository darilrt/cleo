use std::io;

use errors::Error;
use resolver::context::Context;
use types::ScopeID;

use crate::ir::{ExprIR, FnIR, StmtIR};

#[allow(unused)]
pub struct BodyEmitter<'a, 's, 'b> {
    ctx: &'a Context,
    ir: &'s FnIR,
    temps: u32,
    indent: u32,
    frames: Vec<Frame<'b>>,
}

#[allow(unused)]
struct Frame<'a> {
    defers: Vec<&'a StmtIR>,
}

impl<'a, 's, 'b> BodyEmitter<'a, 's, 'b> {
    pub fn new(ctx: &'a Context, ir: &'s FnIR) -> Self {
        Self {
            ctx,
            ir,
            temps: 0,
            indent: 1,
            frames: Vec::new(),
        }
    }

    pub fn emit(&mut self, buffer: &mut impl io::Write, _scope: ScopeID) -> Result<(), Error> {
        for stmt in self.ir.body.iter() {
            emit_identation(buffer, self.indent)?;

            match stmt {
                StmtIR::Expr(expr) => self.emit_expr(buffer, expr),
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
