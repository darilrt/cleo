use chumsky::{Parser, extra, input::ValueInput, prelude::just, span::SimpleSpan};
use lexer::TokenKind;

use crate::{
    ast::TypeDecl,
    errors::ParserError,
    parsers::{
        fn_decl::{FnDecl, fn_decl},
        import::{ImportDecl, import_decl},
        type_decl::type_decl,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum Decl {
    FnDecl(FnDecl),
    TypeDecl(TypeDecl),
    ImportDecl(ImportDecl),
}

pub fn decl<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Decl, extra::Err<ParserError<'tokens, 'src>>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    type_decl()
        .map(Decl::TypeDecl)
        .or(fn_decl().map(Decl::FnDecl))
        .or(import_decl().map(Decl::ImportDecl))
        .then_ignore(just(TokenKind::Semicolon).or_not())
}
