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
    Fn(FnDecl),
    Type(TypeDecl),
    Import(ImportDecl),
    Pub(Box<Decl>),
}

pub fn decl<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Decl, extra::Err<ParserError<'tokens, 'src>>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    let decls = type_decl()
        .map(Decl::Type)
        .or(fn_decl().map(Decl::Fn))
        .or(import_decl().map(Decl::Import))
        .then_ignore(just(TokenKind::Semicolon).or_not());

    just(TokenKind::Pub)
        .ignore_then(decls.clone())
        .map(|d| Decl::Pub(Box::new(d)))
        .or(decls)
}
