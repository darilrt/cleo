use crate::Span;

pub struct Module {
    pub declarations: Vec<Decl>,
}

pub enum Decl {
    /// A function declaration.
    /// `fn name(params) ReturnType { body }`
    Fn(FnDecl),
}

pub struct FnDecl {
    pub signature: Signature,
}

pub struct Signature {
    pub name: Ident,
    // TODO: parameters, return type, etc.
}

pub struct Ident {
    pub literal: String,
    pub span: Span,
}
