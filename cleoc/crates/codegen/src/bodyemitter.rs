use std::io;

use ast::{Expr, ExprValue, Stmt};
use errors::Error;
use resolver::context::Context;

#[allow(unused)]
pub struct BodyEmitter<'a, 's, 'b> {
    ctx: &'a Context,
    stmts: &'s [Stmt],
    temps: u32,
    indent: u32,
    frames: Vec<Frame<'b>>,
}

#[allow(unused)]
struct Frame<'a> {
    defers: Vec<&'a Stmt>,
}

impl<'a, 's, 'b> BodyEmitter<'a, 's, 'b> {
    pub fn new(ctx: &'a Context, stmts: &'s [Stmt]) -> Self {
        Self {
            ctx,
            stmts,
            temps: 0,
            indent: 1,
            frames: Vec::new(),
        }
    }

    pub fn emit(&mut self, buffer: &mut impl io::Write) -> Result<(), Error> {
        for stmt in self.stmts {
            emit_identation(buffer, self.indent)?;
            match stmt {
                Stmt::Expr(expr) => self.emit_expr(buffer, expr),
                _ => unimplemented!(""),
            }?;

            writeln!(buffer, ";")?;
        }

        Ok(())
    }

    fn emit_expr(&self, buffer: &mut impl io::Write, expr: &Expr) -> Result<(), Error> {
        match expr {
            Expr::Value(value) => write!(
                buffer,
                "{}",
                match value {
                    ExprValue::Integer(v) => v,
                    ExprValue::Bool(v) =>
                        if *v {
                            "true"
                        } else {
                            "false"
                        },
                    ExprValue::Float(v) => v,
                    ExprValue::String(v) => v,
                },
            )?,
            // Expr::If(ifexpr) => {}
            _ => {}
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
