use std::io;

use errors::Error;
use resolver::{
    context::{self, Context},
    defkinds::FnSig,
    symbols::DefKind,
};
use types::{ScopeID, TypeID, defs::TypeDef};

use crate::{blockemitter::BlockEmitter, ir::FnIR};

pub struct Codegen<'a> {
    ctx: &'a context::Context,
}

impl<'a> Codegen<'a> {
    pub fn new(ctx: &'a context::Context) -> Self {
        Self { ctx }
    }

    pub fn emit_header(&self, buffer: &mut impl io::Write, scopeid: ScopeID) -> Result<(), Error> {
        use std::io::Write;

        let scope = self
            .ctx
            .table
            .get_scope(scopeid)
            .ok_or_else(|| format!("Scope {} des not exists", scopeid.0))?;

        let mut definitions: Vec<u8> = Vec::new();

        let mut any_def = false;
        for (name, defid) in scope.symbols().iter() {
            let def = self
                .ctx
                .table
                .get_def(*defid)
                .ok_or_else(|| format!("Def {} does not exists", defid.0))?;

            match &def.kind {
                DefKind::Struct(def) => {
                    any_def = true;
                    writeln!(buffer, "typedef struct {0} {0};", name)?;

                    writeln!(definitions, "struct {0} {{", name)?;
                    for (name, typeid) in &def.fields {
                        write!(definitions, "  ")?;
                        emit_type(self.ctx, &mut definitions, *typeid, name)?;
                        writeln!(definitions, "")?;
                    }
                    writeln!(definitions, "}};\n")?;
                }
                _ => {}
            }
        }

        if any_def {
            writeln!(buffer, "")?;
            buffer.write_all(&definitions)?;
        }

        self.emit_fn_decl(buffer, scopeid)?;
        writeln!(buffer, "")?;

        Ok(())
    }

    pub fn emit_fn_decl(&self, buffer: &mut impl io::Write, scope: ScopeID) -> Result<(), Error> {
        let scope = self
            .ctx
            .table
            .get_scope(scope)
            .ok_or_else(|| format!("Scope {} des not exists", scope.0))?;

        for (_name, defid) in scope.symbols().iter() {
            let def = self
                .ctx
                .table
                .get_def(*defid)
                .ok_or_else(|| format!("Def {} does not exists", defid.0))?;

            match &def.kind {
                DefKind::Function(fnsig) => {
                    self.emit_fn_sig(buffer, fnsig)?;
                    writeln!(buffer, ";")?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn emit_fn_sig(&self, buffer: &mut impl io::Write, fnsig: &FnSig) -> Result<(), Error> {
        emit_type(self.ctx, buffer, fnsig.return_type, &fnsig.name)?;
        write!(buffer, "(")?;

        let fntype = match self
            .ctx
            .interner
            .get(fnsig.typeid)
            .ok_or_else(|| format!("Type {} does not exists", fnsig.typeid.0))?
        {
            TypeDef::FnPointer(fntype) => fntype,
            _ => unreachable!("Expected a function pointer"),
        };

        for (name, typeid) in fnsig.params.iter().zip(&fntype.params) {
            emit_type(self.ctx, buffer, *typeid, name)?;
        }

        write!(buffer, ")")?;

        Ok(())
    }

    pub fn emit_fn_def(
        &self,
        buffer: &mut impl io::Write,
        scope: ScopeID,
        fnir: &FnIR,
    ) -> Result<(), Error> {
        let fn_sig = self
            .ctx
            .table
            .get_def(fnir.defid)
            .ok_or_else(|| format!("Def {} does not exists", fnir.defid.0))?
            .fn_sig()
            .ok_or_else(|| format!("Def {} is not a function", fnir.defid.0))?;

        self.emit_fn_sig(buffer, &fn_sig)?;

        writeln!(buffer, " {{")?;
        BlockEmitter::new(self.ctx, &fnir.block.stmts, 1).emit(buffer, scope)?;
        writeln!(buffer, "}}\n")?;

        Ok(())
    }
}

pub fn emit_type<'a>(
    ctx: &Context,
    buffer: &mut impl io::Write,
    typeid: TypeID,
    ident: &str,
) -> Result<(), Error> {
    let type_def = ctx
        .interner
        .get(typeid)
        .ok_or_else(|| format!("Type {} does not exists", typeid.0))?;

    match type_def {
        TypeDef::Void => write!(buffer, "void {}", ident),
        TypeDef::Bool => write!(buffer, "bool {}", ident),
        TypeDef::Int(size, sign) => write!(buffer, "{}{} {}", sign.to_prefix(), size, ident),
        TypeDef::Float(size) => write!(buffer, "f{} {}", size, ident),
        TypeDef::Pointer {
            pointee,
            mutability,
        } => {
            emit_type(ctx, buffer, *pointee, "")?;
            if !*mutability {
                write!(buffer, "const ")?;
            }
            write!(buffer, "* {}", ident)
        }
        TypeDef::Array { element, size } => {
            emit_type(ctx, buffer, *element, ident)?;
            write!(buffer, "[{}]", size)
        }
        TypeDef::FnPointer(_fntype) => {
            todo!("Function Pointer not implemented yet")
        }
        TypeDef::UserDef(defid) => {
            let def = ctx
                .table
                .get_def(*defid)
                .ok_or_else(|| format!("Def {} does not exists", defid.0))?;

            write!(buffer, "{} {}", def.name, ident)
        }
    }?;

    Ok(())
}
