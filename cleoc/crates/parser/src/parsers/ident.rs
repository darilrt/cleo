use chumsky::{Parser, extra, input::ValueInput, select, span::SimpleSpan};
use lexer::TokenKind;

use crate::errors::ParserError;

#[derive(Debug, Clone, PartialEq)]
pub struct Ident {
    pub name: String,
}

impl Ident {
    pub fn str(&self) -> &str {
        &self.name
    }

    pub fn string(&self) -> String {
        self.name.clone()
    }
}

pub fn ident<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Ident, extra::Err<ParserError<'tokens, 'src>>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    select! { TokenKind::Ident(name) => Ident { name: name.to_string() } }
}

impl Ident {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}
