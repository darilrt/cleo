use chumsky::{IterParser, Parser, input::ValueInput, prelude::end, span::SimpleSpan};
use lexer::TokenKind;

use ast::Unit;

use crate::{errors::BoxedParser, parsers::decl::decl};

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
