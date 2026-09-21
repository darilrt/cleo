use ast::Ident;
use chumsky::{Parser, extra, input::ValueInput, select, span::SimpleSpan};
use lexer::TokenKind;

use crate::errors::ParserError;

pub fn ident<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Ident, extra::Err<ParserError<'tokens, 'src>>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    select! { TokenKind::Ident(name) => Ident { name: name.to_string() } }
}
