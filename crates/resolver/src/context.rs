use ast::Type;
use errors::Error;
use types::{
    ScopeID, TypeID, TypeInterner,
    defs::{Signedness, TypeDef},
};

use crate::{
    methos::MethodTable,
    symbols::{DefKind, Definition, SymbolTable},
    unit::UnitRegistry,
};

pub struct Context {
    pub interner: TypeInterner,
    pub table: SymbolTable,
    pub units: UnitRegistry,
    pub methods: MethodTable,
    pub primitives: Primitives,
}

impl Context {
    pub fn new(primitives: Primitives) -> Self {
        Context {
            interner: TypeInterner::new(),
            table: SymbolTable::new(),
            methods: MethodTable::new(),
            units: UnitRegistry::new(),
            primitives,
        }
    }

    pub fn is_comparable(&self, typeid: TypeID) -> bool {
        self.primitives.is_comparable(typeid)
            || matches!(self.interner.get(typeid), Some(TypeDef::Pointer { .. }))
    }

    pub fn is_coercible(&self, from: TypeID, to: TypeID) -> bool {
        if from == to {
            return true;
        }

        if let Some(TypeDef::Const(inner)) = self.interner.get(to) {
            if self.is_coercible(from, *inner) {
                return true;
            }
        }

        match (self.interner.get(from), self.interner.get(to)) {
            (Some(TypeDef::Int(fw, fs)), Some(TypeDef::Int(tw, ts))) if fs == ts && fw < tw => true,
            (Some(TypeDef::Float(fw)), Some(TypeDef::Float(tw))) if fw < tw => true,
            (Some(TypeDef::Pointer { pointee: fp }), Some(TypeDef::Pointer { pointee: tp })) => {
                if fp == tp {
                    return true;
                }
                if let Some(TypeDef::Const(inner)) = self.interner.get(*tp) {
                    if self.is_coercible(*fp, *inner) {
                        return true;
                    }
                }

                return false;
            }
            _ => false,
        }
    }

    pub fn set_primitives(&mut self, primitives: Primitives) {
        self.primitives = primitives;
    }

    pub fn resolve_id(&mut self, scope: ScopeID, ast: &Type) -> Result<TypeID, Error> {
        match ast {
            Type::Ptr(is_const, node) => {
                let pointee = self.resolve_id(scope, node)?;
                let typeid = self.interner.intern(TypeDef::Pointer { pointee });

                if *is_const {
                    Ok(self.interner.intern(TypeDef::Const(typeid)))
                } else {
                    Ok(typeid)
                }
            }
            Type::Array(is_const, size, node) => {
                let element = self.resolve_id(scope, node)?;
                let typeid = self.interner.intern(TypeDef::Array {
                    element,
                    size: *size,
                });

                if *is_const {
                    Ok(self.interner.intern(TypeDef::Const(typeid)))
                } else {
                    Ok(typeid)
                }
            }
            Type::Path(is_const, expr) => {
                if expr.segments.len() == 0 {
                    panic!("Empty path in type_to_def");
                }

                let mut it = expr.segments.iter();

                let typeid = {
                    let mut last_def: Option<&Definition> = None;

                    loop {
                        let segment = it.next().unwrap();

                        let scope = match last_def {
                            Some(def) => match &def.kind {
                                DefKind::Unit { scope } => scope.clone(),
                                _ => {
                                    return Err(format!(
                                        "Expected module in path, found {:?}",
                                        def.kind
                                    )
                                    .into());
                                }
                            },
                            None => scope,
                        };

                        let def =
                            self.table
                                .lookup(scope, &segment.name.str())
                                .ok_or_else(|| {
                                    format!(
                                        "Type '{}' not found in symbol table at scope {:?}",
                                        segment.name.str(),
                                        scope
                                    )
                                })?;
                        let def = self.table.get_def(def).ok_or_else(|| {
                            format!(
                                "Definition for '{}' not found in symbol table",
                                segment.name.str()
                            )
                        })?;

                        last_def = Some(def);

                        if it.len() == 0 {
                            break;
                        }
                    }

                    match last_def {
                        Some(def) => {
                            let Some(typeid) = def.typeid() else {
                                return Err(format!(
                                    "Definition for '{}' does not have a type",
                                    def.name
                                )
                                .into());
                            };

                            typeid
                        }
                        None => {
                            unreachable!()
                        }
                    }
                };

                if *is_const {
                    Ok(self.interner.intern(TypeDef::Const(typeid)))
                } else {
                    Ok(typeid)
                }
            }
        }
    }

    pub fn type_name(&self, typeid: TypeID) -> String {
        let Some(def) = self.interner.get(typeid) else {
            panic!("Unknown type id: {}", typeid.0);
        };

        match def {
            TypeDef::Const(inner) => {
                let inner_name = self.type_name(*inner);
                format!("const {}", inner_name)
            }
            TypeDef::Int(n, s) => format!(
                "{}{}",
                match s {
                    Signedness::Signed => "i",
                    Signedness::Unsigned => "u",
                },
                n
            ),
            TypeDef::USize => "usize".to_string(),
            TypeDef::ISize => "isize".to_string(),
            TypeDef::Float(n) => format!("f{}", n),
            TypeDef::Bool => "bool".to_string(),
            TypeDef::Empty => "void".to_string(),
            TypeDef::Array { element, size } => {
                let element_name = self.type_name(*element);
                format!("[{}]{}", size, element_name)
            }
            TypeDef::Pointer { pointee } => {
                let pointee_name = self.type_name(*pointee);
                format!("*{}", pointee_name)
            }
            TypeDef::UserDef(id) => self
                .table
                .get_def(id.clone())
                .unwrap_or_else(|| panic!(""))
                .name
                .clone(),
            TypeDef::FnPointer(fntype) => format!(
                "fn({}) {}",
                fntype
                    .params
                    .iter()
                    .map(|typeid| self.type_name(*typeid))
                    .collect::<Vec<String>>()
                    .join(", "),
                self.type_name(fntype.return_type)
            ),
        }
    }

    /// Build a string containing all the types in the interner with their names and definitions
    /// in a readable format.
    /// TypeID(0) -> {name} : {definition}
    pub fn debug(&self, scope: ScopeID, show_primitives: bool) -> String {
        let mut output = String::new();

        let primitive_names = vec![
            "void", "bool", "u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64", "f32", "f64",
        ];

        for (name, defid) in self.table.get_scope(scope).unwrap().symbols().iter() {
            if !show_primitives && primitive_names.contains(&name.as_str()) {
                continue;
            }

            output.push_str(&format!(
                "TypeID({}) -> {} : {:?}\n",
                defid.0,
                name,
                self.table.get_def(*defid).unwrap().kind
            ));
        }

        output
    }
}

#[derive(Debug, Clone, Default)]
pub struct Primitives {
    pub nothing_t: TypeID,
    pub bool_t: TypeID,
    pub u8_t: TypeID,
    pub u16_t: TypeID,
    pub u32_t: TypeID,
    pub u64_t: TypeID,
    pub i8_t: TypeID,
    pub i16_t: TypeID,
    pub i32_t: TypeID,
    pub i64_t: TypeID,
    pub f32_t: TypeID,
    pub f64_t: TypeID,
    pub usize_t: TypeID,
    pub isize_t: TypeID,
}

impl Primitives {
    pub fn is_integer(&self, typeid: TypeID) -> bool {
        typeid == self.i8_t
            || typeid == self.i16_t
            || typeid == self.i32_t
            || typeid == self.i64_t
            || typeid == self.u8_t
            || typeid == self.u16_t
            || typeid == self.u32_t
            || typeid == self.u64_t
            || typeid == self.usize_t
            || typeid == self.isize_t
    }

    pub fn is_negatable(&self, typeid: TypeID) -> bool {
        typeid == self.i8_t
            || typeid == self.i16_t
            || typeid == self.i32_t
            || typeid == self.i64_t
            || typeid == self.isize_t
            || typeid == self.f32_t
            || typeid == self.f64_t
    }

    pub fn is_bool(&self, typeid: TypeID) -> bool {
        typeid == self.bool_t
    }

    pub fn is_numeric(&self, typeid: TypeID) -> bool {
        typeid == self.f32_t || typeid == self.f64_t || self.is_integer(typeid)
    }

    pub fn is_comparable(&self, ty: TypeID) -> bool {
        self.is_numeric(ty) || ty == self.bool_t
    }

    pub fn c_name(&self, ty: TypeID) -> Option<&'static str> {
        Some(match ty {
            t if t == self.bool_t => "bool",
            t if t == self.i8_t => "int8_t",
            t if t == self.i16_t => "int16_t",
            t if t == self.i32_t => "int32_t",
            t if t == self.i64_t => "int64_t",
            t if t == self.u8_t => "uint8_t",
            t if t == self.u16_t => "uint16_t",
            t if t == self.u32_t => "uint32_t",
            t if t == self.u64_t => "uint64_t",
            t if t == self.f32_t => "float",
            t if t == self.f64_t => "double",
            t if t == self.usize_t => "size_t",
            t if t == self.isize_t => "ptrdiff_t",
            _ => return None,
        })
    }
}
