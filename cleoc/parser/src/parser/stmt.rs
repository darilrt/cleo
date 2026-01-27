use chumsky::{Parser, extra, input::ValueInput, span::SimpleSpan};
use lexer::TokenKind;

use crate::{
    errors::ParserError,
    parser::expr::{Expr, expr},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(Expr),
}

// stmt = expr;
pub fn stmt_impl<'tokens, 'src: 'tokens, I>(
    expr: impl Parser<'tokens, I, Expr, extra::Err<ParserError<'tokens, 'src>>> + Clone,
) -> impl Parser<'tokens, I, Stmt, extra::Err<ParserError<'tokens, 'src>>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    expr.map(Stmt::Expr)
}

pub fn stmt<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Stmt, extra::Err<ParserError<'tokens, 'src>>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    stmt_impl(expr())
}
