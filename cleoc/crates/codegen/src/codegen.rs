use std::io;

use ast::FnDecl;
use errors::Error;
use resolver::{context, defkinds::FnSig, symbols::DefKind};
use types::{ScopeID, TypeID, defs::TypeDef};

use crate::{bodyemitter::BodyEmitter, ir::FnIR};

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

        for (name, defid) in scope.symbols().iter() {
            let def = self
                .ctx
                .table
                .get_def(*defid)
                .ok_or_else(|| format!("Def {} does not exists", defid.0))?;

            match &def.kind {
                DefKind::Struct(def) => {
                    writeln!(buffer, "typedef struct {0} {0};", name)?;

                    writeln!(definitions, "struct {0} {{", name)?;
                    for (name, typeid) in &def.fields {
                        write!(definitions, "  ")?;
                        self.emit_type(&mut definitions, *typeid, name)?;
                        writeln!(definitions, "")?;
                    }
                    writeln!(definitions, "}};\n")?;
                }
                _ => {}
            }
        }

        writeln!(buffer, "")?;
        buffer.write_all(&definitions)?;
        writeln!(buffer, "")?;

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
        self.emit_type(buffer, fnsig.return_type, &fnsig.name)?;
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
            self.emit_type(buffer, *typeid, name)?;
        }

        write!(buffer, ")")?;

        Ok(())
    }

    pub fn emit_fn_def(
        &self,
        buffer: &mut impl io::Write,
        scope: ScopeID,
        decl: &FnIR,
    ) -> Result<(), Error> {
        self.emit_fn_sig(buffer, &decl.sig)?;
        Ok(())
    }

    pub fn emit_type(
        &self,
        buffer: &mut impl io::Write,
        typeid: TypeID,
        ident: &str,
    ) -> Result<(), Error> {
        let type_def = self
            .ctx
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
                self.emit_type(buffer, *pointee, "")?;
                if !*mutability {
                    write!(buffer, "const ")?;
                }
                write!(buffer, "* {}", ident)
            }
            TypeDef::Array { element, size } => {
                self.emit_type(buffer, *element, ident)?;
                write!(buffer, "[{}]", size)
            }
            TypeDef::FnPointer(_fntype) => {
                todo!("Function Pointer not implemented yet")
            }
            TypeDef::UserDef(defid) => {
                let def = self
                    .ctx
                    .table
                    .get_def(*defid)
                    .ok_or_else(|| format!("Def {} does not exists", defid.0))?;

                write!(buffer, "{} {}", def.name, ident)
            }
        }?;

        Ok(())
    }
}
