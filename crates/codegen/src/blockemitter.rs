use std::io;

use errors::Error;
use resolver::context::Context;
use types::{ScopeID, defs::TypeDef};

use crate::{
    codegen::emit_type,
    ir::{BlockIR, ExprIR, Intrinsic, StmtIR},
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
            match stmt {
                StmtIR::Expr(expr) => match expr {
                    ExprIR::Empty => Ok(()),
                    expr => {
                        emit_identation(buffer, self.indent)?;
                        self.emit_expr(buffer, expr)?;
                        writeln!(buffer, ";")
                    }
                },
                StmtIR::Local(name, typeid, expr) => {
                    emit_identation(buffer, self.indent)?;
                    emit_type(self.ctx, buffer, *typeid, name)?;
                    if let Some(expr) = expr {
                        write!(buffer, " = ")?;
                        self.emit_expr(buffer, expr)?;
                    };
                    writeln!(buffer, ";")
                }
                StmtIR::Assign(label, expr) => {
                    emit_identation(buffer, self.indent)?;
                    write!(buffer, "{} = ", label)?;
                    self.emit_expr(buffer, expr)?;
                    writeln!(buffer, ";")
                }
                StmtIR::If(expr, if_block, else_block) => {
                    self.emit_if(buffer, expr, if_block, else_block.as_ref())?;
                    writeln!(buffer, "")
                }
                StmtIR::Return(expr) => {
                    emit_identation(buffer, self.indent)?;
                    write!(buffer, "return")?;

                    if let Some(expr) = expr {
                        write!(buffer, " ")?;
                        self.emit_expr(buffer, expr)?;
                    }

                    writeln!(buffer, ";")?;
                    Ok(())
                }
                StmtIR::For(block) => {
                    self.emit_for(buffer, block)?;
                    writeln!(buffer, "")
                }
                StmtIR::Break => {
                    emit_identation(buffer, self.indent)?;
                    writeln!(buffer, "break;")
                }
            }?;
        }

        Ok(())
    }

    fn emit_expr(&self, buffer: &mut impl io::Write, expr: &ExprIR) -> Result<(), Error> {
        match expr {
            ExprIR::Empty => {}
            ExprIR::Intrinsic(intrinsic) => {
                self.emit_intrinsic(buffer, intrinsic)?;
            }
            ExprIR::Access { inner, segment } => {
                self.emit_expr(buffer, inner)?;
                write!(buffer, ".{}", segment)?;
            }
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
            ExprIR::UnaryOp { op, expr } => {
                write!(buffer, "{}", op)?;
                self.emit_expr(buffer, expr)?;
            }
        }

        Ok(())
    }

    fn emit_intrinsic(
        &self,
        buffer: &mut impl io::Write,
        intrinsic: &Intrinsic,
    ) -> Result<(), Error> {
        match intrinsic {
            Intrinsic::Reinterpret {
                inner: expr,
                from,
                to,
            } => {
                let from_type = self.ctx.interner.get(*from).unwrap();
                let to_type = self.ctx.interner.get(*to).unwrap();

                let is_from_ptr = matches!(from_type, TypeDef::Pointer { .. });
                let is_to_ptr = matches!(to_type, TypeDef::Pointer { .. });

                let is_from_int = matches!(from_type, TypeDef::Int(_, _));
                let is_to_int = matches!(to_type, TypeDef::Int(_, _));

                if (is_from_ptr && is_to_int)
                    || (is_from_int && is_to_ptr)
                    || (is_from_ptr && is_to_ptr)
                {
                    // Conversión directa escalar/puntero: (ToType)(expr)
                    // Cubre: T* -> Int, Int -> T*, T* -> U*
                    write!(buffer, "(")?;
                    emit_type(self.ctx, buffer, *to, "")?;
                    write!(buffer, ")(")?;
                    self.emit_expr(buffer, expr)?;
                    write!(buffer, ")")?;
                } else if is_from_ptr {
                    // Caso: T* -> Valor U por desreferencia: *(U*)(ptr)
                    write!(buffer, "*(")?;
                    emit_type(self.ctx, buffer, *to, "")?;
                    write!(buffer, "*)(")?;
                    self.emit_expr(buffer, expr)?;
                    write!(buffer, ")")?;
                } else {
                    // Caso por valor general: T -> U mediante Type Punning: *(U*)&(expr)
                    write!(buffer, "*(")?;
                    emit_type(self.ctx, buffer, *to, "")?;
                    write!(buffer, "*)&(")?;
                    self.emit_expr(buffer, expr)?;
                    write!(buffer, ")")?;
                }
            }
        }
        Ok(())
    }

    fn emit_for(&self, buffer: &mut impl io::Write, block: &BlockIR) -> Result<(), Error> {
        emit_identation(buffer, self.indent)?;
        writeln!(buffer, "for (;;) {{")?;
        BlockEmitter::new(self.ctx, &block.stmts, self.indent + 1).emit(buffer, ScopeID(0))?;
        emit_identation(buffer, self.indent)?;
        write!(buffer, "}}")?;
        Ok(())
    }

    fn emit_if(
        &self,
        buffer: &mut impl io::Write,
        expr: &ExprIR,
        if_block: &BlockIR,
        else_block: Option<&BlockIR>,
    ) -> Result<(), Error> {
        emit_identation(buffer, self.indent)?;
        write!(buffer, "if (")?;
        self.emit_expr(buffer, expr)?;
        writeln!(buffer, ") {{")?;
        BlockEmitter::new(self.ctx, &if_block.stmts, self.indent + 1).emit(buffer, ScopeID(0))?;
        emit_identation(buffer, self.indent)?;
        write!(buffer, "}}")?;

        if let Some(block) = else_block {
            writeln!(buffer, "")?;
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
