use chumsky::{Parser, extra, input::ValueInput, span::SimpleSpan};
use lexer::TokenKind;

use crate::errors::ParserError;

pub fn $parser_name<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, $ReturnType, extra::Err<ParserError<'tokens, 'src>>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
}
