use ast::{Decl, FnDecl, TypeBody, TypeDecl, Unit};
use errors::Error;
use types::{
    DefID, ScopeID, TypeID,
    defs::{FnPointerType, TypeDef},
};

use crate::{
    context::Context,
    defkinds::{EnumDef, TraitDef, TypeAliasDef},
    gather::gather,
    symbols::DefKind,
};

pub struct Resolver<'a> {
    pub ctx: &'a mut Context,
}

impl<'a> Resolver<'a> {
    pub fn new(ctx: &'a mut Context) -> Self {
        Self { ctx }
    }

    pub fn resolve(&mut self, scope: ScopeID, unit: &Unit) -> Result<(), Error> {
        gather(unit, scope, &mut self.ctx.table, &mut self.ctx.interner)?;
        self.resolve_unit(unit, scope)?;
        Ok(())
    }

    pub fn resolve_unit(&mut self, unit: &Unit, scope: ScopeID) -> Result<(), Error> {
        for def in &unit.decls {
            self.resolve_decl(scope, def)?;
        }
        Ok(())
    }

    pub fn resolve_decl(&mut self, scope: ScopeID, decl: &Decl) -> Result<(), Error> {
        match decl {
            Decl::Fn(decl) => self.resolve_fn(scope, decl),
            Decl::Pub(decl) => self.resolve_decl(scope, decl),
            Decl::Type(decl) => self.resolve_type(scope, decl),
            Decl::Import(_) => Ok(()),
        }
    }

    pub fn resolve_fn(&mut self, scope: ScopeID, decl: &FnDecl) -> Result<(), Error> {
        let params = decl
            .signature
            .params
            .iter()
            .map(|param| {
                let ty = self.ctx.resolve_id(scope, &param.ty)?;
                Ok((param.name.str().to_string(), ty))
            })
            .collect::<Result<Vec<_>, String>>()?;

        {
            let fn_name = decl.signature.name.str();
            let ret_typeid = match &decl.signature.return_type {
                Some(ty) => self.ctx.resolve_id(scope, ty)?,
                None => self.ctx.interner.intern(TypeDef::Void),
            };

            let defid = self
                .ctx
                .table
                .lookup_local(scope, &fn_name)
                .ok_or_else(|| format!("Function '{}' not found in symbol table", fn_name))?;

            let func_def = self.ctx.table.get_def_mut(defid).ok_or_else(|| {
                format!(
                    "Function definition for '{}' not found in symbol table",
                    fn_name
                )
            })?;

            let DefKind::Function(def) = &mut func_def.kind else {
                return Err(format!(
                    "Expected function definition for '{}', found {:?}",
                    fn_name, func_def.kind
                )
                .into());
            };

            let typeid = self.ctx.interner.intern(TypeDef::FnPointer(FnPointerType {
                params: params.iter().map(|(_, typeid)| typeid.clone()).collect(),
                return_type: ret_typeid,
            }));

            def.params = params.iter().map(|(name, _)| name.clone()).collect();
            def.return_type = ret_typeid;
            def.resolved = true;
            def.typeid = typeid;
        }

        Ok(())
    }

    pub fn resolve_type(&mut self, scope: ScopeID, decl: &TypeDecl) -> Result<(), Error> {
        let TypeDecl {
            name,
            body,
            generics: _,
        } = decl;

        match &body {
            TypeBody::Alias(ty) => {
                let typeid = self.ctx.resolve_id(scope, ty)?;

                let def = self
                    .ctx
                    .table
                    .get_def_mut(self.ctx.table.lookup_local(scope, &name.str()).ok_or_else(
                        || format!("Type '{}' not found in symbol table", name.str()),
                    )?)
                    .ok_or_else(|| {
                        format!(
                            "Type definition for '{}' not found in symbol table",
                            name.str()
                        )
                    })?;

                def.kind = DefKind::TypeAlias(TypeAliasDef {
                    resolved: true,
                    typeid,
                });
            }
            TypeBody::Enum(values) => {
                let def = self
                    .ctx
                    .table
                    .get_def_mut(self.ctx.table.lookup_local(scope, &name.str()).ok_or_else(
                        || format!("Type '{}' not found in symbol table", name.str()),
                    )?)
                    .ok_or_else(|| {
                        format!(
                            "Type definition for '{}' not found in symbol table",
                            name.str()
                        )
                    })?;

                let typeid = def.typeid().ok_or_else(|| {
                    format!("TypeID for enum '{}' not found in symbol table", name.str())
                })?;

                def.kind = DefKind::Enum(EnumDef {
                    resolved: true,
                    values: values.iter().map(|value| value.name.string()).collect(),
                    typeid: typeid,
                });
            }
            TypeBody::Struct(fields) => {
                let fields = fields
                    .iter()
                    .map(|field| {
                        let field_typeid = self.ctx.resolve_id(scope, &field.field_type)?;
                        Ok((field.name.string(), field_typeid))
                    })
                    .collect::<Result<Vec<_>, String>>()?;

                let def = self
                    .ctx
                    .table
                    .get_def_mut(self.ctx.table.lookup_local(scope, &name.str()).ok_or_else(
                        || format!("Type '{}' not found in symbol table", name.str()),
                    )?)
                    .ok_or_else(|| {
                        format!(
                            "Type definition for '{}' not found in symbol table",
                            name.str()
                        )
                    })?;

                def.kind = DefKind::Struct(crate::defkinds::StructDef {
                    resolved: true,
                    fields,
                    typeid: def.typeid().ok_or_else(|| {
                        format!(
                            "TypeID for struct '{}' not found in symbol table",
                            name.str()
                        )
                    })?,
                });
            }
            TypeBody::Trait(_methods) => {
                // TODO: Implement trait resolution

                let def = self
                    .ctx
                    .table
                    .get_def_mut(self.ctx.table.lookup_local(scope, &name.str()).ok_or_else(
                        || format!("Type '{}' not found in symbol table", name.str()),
                    )?)
                    .ok_or_else(|| {
                        format!(
                            "Type definition for '{}' not found in symbol table",
                            name.str()
                        )
                    })?;

                def.kind = DefKind::Trait(TraitDef {
                    resolved: true,
                    typeid: def.typeid().ok_or_else(|| {
                        format!(
                            "TypeID for trait '{}' not found in symbol table",
                            name.str()
                        )
                    })?,
                })
            }
        }

        Ok(())
    }
}

pub enum Resolved {
    Value(TypeID),
    Type(TypeID, DefID),
    EnumValue(TypeID),
    Module(ScopeID),
}
