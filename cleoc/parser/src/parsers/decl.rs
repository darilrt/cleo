use chumsky::{Parser, extra, input::ValueInput, span::SimpleSpan};
use lexer::TokenKind;

use crate::{
    ast::TypeDecl,
    errors::ParserError,
    parsers::{
        fn_decl::{FnDecl, fn_decl},
        type_decl::type_decl,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum Decl {
    FnDecl(FnDecl),
    TypeDecl(TypeDecl),
}

pub fn decl<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Decl, extra::Err<ParserError<'tokens, 'src>>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    type_decl()
        .map(Decl::TypeDecl)
        .or(fn_decl().map(Decl::FnDecl))
}
