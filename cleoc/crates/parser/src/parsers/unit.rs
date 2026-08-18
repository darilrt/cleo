use chumsky::{IterParser, Parser, input::ValueInput, prelude::end, span::SimpleSpan};
use lexer::TokenKind;

use crate::{ast::Decl, errors::BoxedParser, parsers::decl::decl};

#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    pub decls: Vec<Decl>,
}

pub fn unit<'tokens, 'src: 'tokens, I>() -> BoxedParser<'tokens, 'src, I, Unit>
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    decl()
        .repeated()
        .collect()
        .then_ignore(end())
        .map(|decls| Unit { decls })
        .boxed()
}
