use crate::{DefID, TypeID};

#[derive(Clone, PartialEq, Hash, Eq, Debug)]
pub enum TypeDef {
    Bool,
    Void,
    Int(u8, Signedness),
    Float(u8),
    UserDef(DefID),

    FnPointer(FnPointerType),
    Pointer { pointee: TypeID, mutability: bool },
    Array { element: TypeID, size: usize },
}

#[derive(Clone, PartialEq, Hash, Eq, Debug)]
pub struct FnPointerType {
    pub params: Vec<TypeID>,
    pub return_type: TypeID,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Signedness {
    Signed,
    Unsigned,
}
