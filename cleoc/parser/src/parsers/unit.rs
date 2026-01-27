use chumsky::{IterParser, Parser, extra, input::ValueInput, prelude::end, span::SimpleSpan};
use lexer::TokenKind;

use crate::{ast::Decl, errors::ParserError, parsers::decl::decl};

#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    pub decls: Vec<Decl>,
}

pub fn unit<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Unit, extra::Err<ParserError<'tokens, 'src>>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    decl()
        .repeated()
        .collect()
        .then_ignore(end())
        .map(|decls| Unit { decls })
}
