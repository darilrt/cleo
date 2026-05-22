use std::collections::HashMap;

// ─── Type Interning ───────────────────────────────────────────────────────────

// A TypeId is just an index into the TypeRegistry.
// Comparing two types is comparing two u32 — O(1), no allocations.
// Copy because it is passed by value constantly, like an integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub u32);

// The structural description of a type.
// Hash + Eq are derived so it can be used as a key in the deduplication cache.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeInfo {
    // Language primitives
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Bool,

    // The "no value" type — functions that return nothing.
    Void,

    // `*T` — mutable pointer to `inner`.
    // `*const T` — immutable pointer, mutable: false.
    Pointer { mutable: bool, inner: TypeId },

    // `[N]T` — fixed-size array whose size is known at compile time.
    Array { size: usize, inner: TypeId },

    // A user-defined named type: struct, enum, trait, or alias.
    // Resolved by looking up the name in ModuleScope.types.
    Named(String),
}

// Central type registry.
// `types` is the source of truth — the index IS the TypeId.
// `cache` is the reverse index for deduplication: if `*i32` was already
// interned, it returns the same TypeId instead of creating a new one.
pub struct TypeRegistry {
    types: Vec<TypeInfo>,
    cache: HashMap<TypeInfo, TypeId>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        let mut registry = TypeRegistry {
            types: Vec::new(),
            cache: HashMap::new(),
        };
        // Pre-populate primitives so their TypeIds are always the same.
        // Order matters: TypeId(0) = I8, TypeId(1) = I16, etc.
        // This allows the TypeId::I32 style constants below.
        registry.intern_builtin(TypeInfo::I8);
        registry.intern_builtin(TypeInfo::I16);
        registry.intern_builtin(TypeInfo::I32);
        registry.intern_builtin(TypeInfo::I64);
        registry.intern_builtin(TypeInfo::U8);
        registry.intern_builtin(TypeInfo::U16);
        registry.intern_builtin(TypeInfo::U32);
        registry.intern_builtin(TypeInfo::U64);
        registry.intern_builtin(TypeInfo::F32);
        registry.intern_builtin(TypeInfo::F64);
        registry.intern_builtin(TypeInfo::Bool);
        registry.intern_builtin(TypeInfo::Void);
        registry
    }

    fn intern_builtin(&mut self, info: TypeInfo) {
        let id = TypeId(self.types.len() as u32);
        self.cache.insert(info.clone(), id);
        self.types.push(info);
    }

    // Interns a type: returns its existing TypeId if already present, otherwise adds it.
    // This is the function the collector calls whenever it encounters a type in the AST.
    pub fn intern(&mut self, info: TypeInfo) -> TypeId {
        if let Some(&id) = self.cache.get(&info) {
            return id;
        }
        let id = TypeId(self.types.len() as u32);
        self.cache.insert(info.clone(), id);
        self.types.push(info);
        id
    }

    // Returns the TypeInfo for a given TypeId.
    // Panics if the id does not exist — should never happen if TypeIds are only
    // created through intern().
    pub fn get(&self, id: TypeId) -> &TypeInfo {
        &self.types[id.0 as usize]
    }
}

// Constants for primitive TypeIds.
// Safe because new() pre-populates them in a fixed order.
// New primitives must always be appended, never inserted.
impl TypeId {
    pub const I8: TypeId = TypeId(0);
    pub const I16: TypeId = TypeId(1);
    pub const I32: TypeId = TypeId(2);
    pub const I64: TypeId = TypeId(3);
    pub const U8: TypeId = TypeId(4);
    pub const U16: TypeId = TypeId(5);
    pub const U32: TypeId = TypeId(6);
    pub const U64: TypeId = TypeId(7);
    pub const F32: TypeId = TypeId(8);
    pub const F64: TypeId = TypeId(9);
    pub const BOOL: TypeId = TypeId(10);
    pub const VOID: TypeId = TypeId(11);
}

// ─── Local variables ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Binding {
    Var,   // mutable
    Const, // immutable
}

// A variable declared inside a function or block.
// `ty` is None when the type is omitted and must be inferred later.
#[derive(Debug, Clone)]
pub struct VarSymbol {
    pub name: String,
    pub binding: Binding,
    pub ty: Option<TypeId>,
}

// ─── Module-level types ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FieldSymbol {
    pub name: String,
    pub ty: TypeId,
}

#[derive(Debug, Clone)]
pub struct TraitMethodSymbol {
    pub name: String,
    pub params: Vec<ParamSymbol>,
    pub return_type: Option<TypeId>,
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    Struct(Vec<FieldSymbol>),
    Enum(Vec<String>),
    Trait(Vec<TraitMethodSymbol>),
    Alias(TypeId),
}

#[derive(Debug, Clone)]
pub struct TypeSymbol {
    pub name: String,
    pub generics: Vec<String>,
    pub kind: TypeKind,
    pub is_pub: bool,
}

// ─── Module-level functions ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ParamSymbol {
    pub name: String,
    pub ty: TypeId,
    pub is_self: bool,
}

#[derive(Debug, Clone)]
pub struct GenericSymbol {
    pub name: String,
    pub bound: Option<TypeId>,
}

#[derive(Debug, Clone)]
pub struct FnSymbol {
    pub name: String,
    pub generics: Vec<GenericSymbol>,
    pub params: Vec<ParamSymbol>,
    pub return_type: Option<TypeId>,
    // If the first param is named "self", this is the TypeId of the receiver type.
    // `fn print(self: *Name)` → receiver = Some(id of *Name)
    pub receiver: Option<TypeId>,
    // The trait this function implements, if it has `for TraitName`.
    pub trait_impl: Option<String>,
    pub is_inline: bool,
    pub is_pub: bool,
}

// ─── Scopes ───────────────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct ModuleScope {
    pub types: HashMap<String, TypeSymbol>,
    // Vec because multiple functions can share the same name with different `for Trait`.
    pub functions: HashMap<String, Vec<FnSymbol>>,
}

#[derive(Debug, Default)]
pub struct LocalScope {
    pub variables: HashMap<String, VarSymbol>,
}

// ─── Symbol table ─────────────────────────────────────────────────────────────

pub struct SymbolTable {
    pub module: ModuleScope,
    // Central type registry. Lives here because both the collector (reading the AST)
    // and the analyzer (checking types) need access to it.
    pub registry: TypeRegistry,
    locals: Vec<LocalScope>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            module: ModuleScope::default(),
            registry: TypeRegistry::new(),
            locals: Vec::new(),
        }
    }
}

#[allow(unused)]
mod test {
    use super::*;

    #[test]
    fn test_primitives_are_stable() {
        let table = SymbolTable::new();

        assert_eq!(table.registry.get(TypeId::I32), &TypeInfo::I32);
        assert_eq!(table.registry.get(TypeId::BOOL), &TypeInfo::Bool);
        assert_eq!(table.registry.get(TypeId::VOID), &TypeInfo::Void);
    }

    #[test]
    fn test_intern_deduplication() {
        let mut table = SymbolTable::new();

        let id1 = table.registry.intern(TypeInfo::Named("Point".to_string()));
        let id2 = table.registry.intern(TypeInfo::Named("Point".to_string()));
        assert_eq!(id1, id2);

        let id3 = table.registry.intern(TypeInfo::Named("Vec".to_string()));
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_pointer_intern() {
        let mut table = SymbolTable::new();

        let ptr_mut = table.registry.intern(TypeInfo::Pointer {
            mutable: true,
            inner: TypeId::I32,
        });
        let ptr_const = table.registry.intern(TypeInfo::Pointer {
            mutable: false,
            inner: TypeId::I32,
        });
        assert_ne!(ptr_mut, ptr_const);

        let ptr_mut2 = table.registry.intern(TypeInfo::Pointer {
            mutable: true,
            inner: TypeId::I32,
        });
        assert_eq!(ptr_mut, ptr_mut2);
    }
}
