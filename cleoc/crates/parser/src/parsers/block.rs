use chumsky::{
    IterParser, Parser, extra,
    input::ValueInput,
    prelude::{just, recursive},
    span::SimpleSpan,
};
use lexer::TokenKind;

use crate::{
    errors::{BoxedParser, ParserError},
    parsers::{
        expr::{Expr, expr},
        stmt::{Stmt, stmt_impl},
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Stmt>,
}

// block = '{', { stmt, ';' }, '}'
pub fn block_impl<'tokens, 'src: 'tokens, I>(
    expr: impl Parser<'tokens, I, Expr, extra::Err<ParserError<'tokens, 'src>>> + Clone + 'tokens,
) -> BoxedParser<'tokens, 'src, I, Block>
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    recursive(|block| {
        stmt_impl(expr, block)
            .separated_by(just(TokenKind::Semicolon))
            .allow_leading()
            .collect::<Vec<Stmt>>()
            .or_not()
            .delimited_by(just(TokenKind::LeftBrace), just(TokenKind::RightBrace))
            .map(|statements| Block {
                statements: statements.unwrap_or_default(),
            })
    })
    .boxed()
}

pub fn block<'tokens, 'src: 'tokens, I>() -> BoxedParser<'tokens, 'src, I, Block>
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    block_impl(expr())
}
