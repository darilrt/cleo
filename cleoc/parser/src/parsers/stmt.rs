use chumsky::{Parser, extra, input::ValueInput, span::SimpleSpan};
use lexer::TokenKind;

use crate::{
    errors::ParserError,
    parsers::{
        expr::Expr,
        local::{LocalDecl, local_impl},
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(Expr),
    Local(LocalDecl),
}

// stmt = local | expr;
pub fn stmt_impl<'tokens, 'src: 'tokens, I>(
    expr: impl Parser<'tokens, I, Expr, extra::Err<ParserError<'tokens, 'src>>> + Clone,
) -> impl Parser<'tokens, I, Stmt, extra::Err<ParserError<'tokens, 'src>>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    local_impl(expr.clone())
        .map(Stmt::Local)
        .or(expr.map(Stmt::Expr))
}
