use std::io;

use errors::Error;
use resolver::context::Context;
use types::ScopeID;

use crate::{
    codegen::emit_type,
    ir::{BlockIR, ExprIR, StmtIR},
};

#[allow(unused)]
pub struct BlockEmitter<'a, 's> {
    ctx: &'a Context,
    ir: &'s Vec<StmtIR>,
    indent: u32,
}

impl<'a, 's> BlockEmitter<'a, 's> {
    pub fn new(ctx: &'a Context, ir: &'s Vec<StmtIR>, indent: u32) -> Self {
        Self { ctx, ir, indent }
    }

    pub fn emit(&mut self, buffer: &mut impl io::Write, _scope: ScopeID) -> Result<(), Error> {
        for stmt in self.ir.iter() {
            emit_identation(buffer, self.indent)?;

            match stmt {
                StmtIR::Expr(expr) => {
                    self.emit_expr(buffer, expr)?;
                    writeln!(buffer, ";")
                }
                StmtIR::Local(name, typeid, expr) => {
                    emit_type(self.ctx, buffer, *typeid, name)?;
                    if let Some(expr) = expr {
                        write!(buffer, " = ")?;
                        self.emit_expr(buffer, expr)?;
                    };
                    writeln!(buffer, ";")
                }
                StmtIR::Assign(label, expr) => {
                    write!(buffer, "{} = ", label)?;
                    self.emit_expr(buffer, expr)?;
                    writeln!(buffer, ";")
                }
                StmtIR::If(expr, if_block, else_block) => {
                    self.emit_if(buffer, expr, if_block, else_block.as_ref())?;
                    writeln!(buffer, "")
                }
            }?;
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
            ExprIR::Call { callee, args } => {
                self.emit_expr(buffer, callee)?;
                write!(buffer, "(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(buffer, ", ")?;
                    }
                    self.emit_expr(buffer, arg)?;
                }
                write!(buffer, ")")?;
            }
            ExprIR::Assign { left, kind, right } => {
                self.emit_expr(buffer, left)?;
                write!(
                    buffer,
                    " {} ",
                    match kind {
                        ast::AssignKind::AddEqual => "+=",
                        ast::AssignKind::SubEqual => "-=",
                        ast::AssignKind::MulEqual => "*=",
                        ast::AssignKind::DivEqual => "/=",
                        ast::AssignKind::Equal => "=",
                    }
                )?;
                self.emit_expr(buffer, right)?;
            }
        }

        Ok(())
    }

    fn emit_if(
        &self,
        buffer: &mut impl io::Write,
        expr: &ExprIR,
        if_block: &BlockIR,
        else_block: Option<&BlockIR>,
    ) -> Result<(), Error> {
        write!(buffer, "if (")?;
        self.emit_expr(buffer, expr)?;
        writeln!(buffer, ") {{")?;
        BlockEmitter::new(self.ctx, &if_block.stmts, self.indent + 1).emit(buffer, ScopeID(0))?;
        emit_identation(buffer, self.indent)?;
        writeln!(buffer, "}}")?;

        if let Some(block) = else_block {
            emit_identation(buffer, self.indent)?;
            writeln!(buffer, "else {{")?;
            BlockEmitter::new(self.ctx, &block.stmts, self.indent + 1).emit(buffer, ScopeID(0))?;
            emit_identation(buffer, self.indent)?;
            write!(buffer, "}}")?;
        };

        Ok(())
    }
}

fn emit_identation(buffer: &mut impl io::Write, identation: u32) -> Result<(), Error> {
    for _ in 0..identation {
        write!(buffer, "  ")?;
    }

    Ok(())
}
