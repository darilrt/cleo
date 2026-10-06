use ast::{Decl, Root, TypeBody, TypeDecl};
use errors::Error;
use types::{ScopeID, TypeID, TypeInterner, defs::TypeDef};

use crate::{
    defkinds::{EnumDef, FnSig, StructDef, TraitDef, TypeAliasDef},
    symbols::{DefKind, Definition, SymbolTable},
    unit::UnitRegistry,
};

pub fn gather_type(ty: &TypeDecl) -> Result<Definition, String> {
    Ok(Definition {
        mangled_name: None,
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
    units: &mut UnitRegistry,
) -> Result<(), Error> {
    for decl in ast {
        match decl {
            Decl::Import(decl) => {
                let name = decl.path.last().unwrap().string();

                let import_path = decl
                    .path
                    .iter()
                    .map(|ident| ident.str())
                    .collect::<Vec<_>>()
                    .join(".");

                let unit_scope = units
                    .get(&import_path)
                    .ok_or_else(|| format!("unit {} not found", import_path))?;

                let def = Definition {
                    name: name,
                    mangled_name: None,
                    kind: DefKind::Unit { scope: unit_scope },
                };

                table.define(scope, def)?;
            }
            Decl::Fn(func) => {
                let def = Definition {
                    name: func.signature.name.string(),
                    mangled_name: None,
                    kind: DefKind::Function(FnSig::unresolved(func.signature.name.str())),
                };

                table.define(scope, def)?;
            }

            Decl::Type(ty) => {
                let def = gather_type(&ty)?;

                let Ok(def_id) = table.define(scope, def) else {
                    return Err(format!("failed to define type: {}", ty.name.name).into());
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
        }
    }

    Ok(())
}

pub fn gather(
    ast: &Root,
    scope: ScopeID,
    table: &mut SymbolTable,
    interner: &mut TypeInterner,
    units: &mut UnitRegistry,
) -> Result<(), Error> {
    gather_decl(&ast.decls, scope, table, interner, units)
}
