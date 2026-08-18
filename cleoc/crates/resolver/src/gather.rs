use parser::ast::{Decl, TypeBody, TypeDecl, Unit};
use types::{ScopeID, TypeID, TypeInterner, defs::TypeDef};

use crate::{
    defkinds::{EnumDef, FnSig, StructDef, TraitDef, TypeAliasDef},
    symbols::{DefKind, Definition, SymbolTable},
};

pub fn gather_type(ty: &TypeDecl) -> Result<Definition, String> {
    Ok(Definition {
        name: ty.name.name.clone(),
        kind: match &ty.body {
            TypeBody::Struct(_) => DefKind::Struct(StructDef {
                fields: Vec::new(),
                resolved: false,
                typeid: TypeID(0),
            }),
            TypeBody::Trait(_) => DefKind::Trait(TraitDef {
                resolved: false,
                typeid: TypeID(0),
            }),
            TypeBody::Enum(_) => DefKind::Enum(EnumDef {
                resolved: false,
                values: Vec::new(),
                typeid: TypeID(0),
            }),
            TypeBody::Alias(_) => DefKind::TypeAlias(TypeAliasDef {
                resolved: false,
                typeid: TypeID(0),
            }),
        },
    })
}

pub fn gather_decl(
    ast: &[Decl],
    scope: ScopeID,
    table: &mut SymbolTable,
    interner: &mut TypeInterner,
) -> Result<(), String> {
    for decl in ast {
        match decl {
            Decl::Fn(func) => {
                let def = Definition {
                    name: func.signature.name.name.clone(),
                    kind: DefKind::Function(FnSig {
                        name: func.signature.name.name.clone(),
                        params: Vec::new(),
                        return_type: TypeID(0),
                        resolved: false,
                        typeid: TypeID(0),
                    }),
                };

                let Ok(_) = table.define(scope, def) else {
                    return Err(format!(
                        "failed to define function: {}",
                        func.signature.name.name
                    ));
                };
            }

            Decl::Type(ty) => {
                let def = gather_type(&ty)?;

                let Ok(def_id) = table.define(scope, def) else {
                    return Err(format!("failed to define type: {}", ty.name.name));
                };

                let def = table.get_def_mut(def_id).unwrap();
                let typeid = interner.intern(TypeDef::UserDef(def_id));

                match &mut def.kind {
                    DefKind::Enum(def) => def.typeid = typeid,
                    DefKind::Struct(def) => def.typeid = typeid,
                    DefKind::TypeAlias(def) => def.typeid = typeid,
                    DefKind::Trait(def) => def.typeid = typeid,
                    _ => {}
                }
            }

            Decl::Import(_decl) => {}

            Decl::Pub(_decl) => {}
        }
    }

    Ok(())
}

pub fn gather(
    ast: &Unit,
    scope: ScopeID,
    table: &mut SymbolTable,
    interner: &mut TypeInterner,
) -> Result<(), String> {
    gather_decl(&ast.decls, scope, table, interner)
}
