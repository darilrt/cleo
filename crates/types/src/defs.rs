use crate::{DefID, TypeID};

#[derive(Clone, PartialEq, Hash, Eq, Debug)]
pub enum TypeDef {
    Bool,
    Empty,
    Int(u8, Signedness),
    Float(u8),
    UserDef(DefID),
    Const(TypeID),
    FnPointer(FnPointerType),
    Pointer { pointee: TypeID },
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

impl Signedness {
    pub fn to_prefix<'a>(&self) -> &'a str {
        match self {
            Signedness::Signed => "i",
            Signedness::Unsigned => "u",
        }
    }
}
