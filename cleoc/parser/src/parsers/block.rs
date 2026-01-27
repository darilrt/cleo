use chumsky::{IterParser, Parser, extra, input::ValueInput, prelude::just, span::SimpleSpan};
use lexer::TokenKind;

use crate::{
    errors::ParserError,
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
    expr: impl Parser<'tokens, I, Expr, extra::Err<ParserError<'tokens, 'src>>> + Clone,
) -> impl Parser<'tokens, I, Block, extra::Err<ParserError<'tokens, 'src>>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    stmt_impl(expr)
        .separated_by(just(TokenKind::Semicolon))
        .allow_leading()
        .collect::<Vec<Stmt>>()
        .or_not()
        .delimited_by(just(TokenKind::LeftBrace), just(TokenKind::RightBrace))
        .map(|statements| Block {
            statements: statements.unwrap_or_default(),
        })
}

pub fn block<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Block, extra::Err<ParserError<'tokens, 'src>>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    block_impl(expr())
}
