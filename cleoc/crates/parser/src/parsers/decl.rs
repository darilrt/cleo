use ast::Decl;
use chumsky::{Parser, input::ValueInput, prelude::just, span::SimpleSpan};
use lexer::TokenKind;

use crate::{
    errors::BoxedParser,
    parsers::{fn_decl::fn_decl, import::import_decl, type_decl::type_decl},
};

pub fn decl<'tokens, 'src: 'tokens, I>() -> BoxedParser<'tokens, 'src, I, Decl>
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
        .boxed()
}
