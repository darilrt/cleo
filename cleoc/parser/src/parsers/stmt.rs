use chumsky::{Parser, extra, input::ValueInput, prelude::just, span::SimpleSpan};
use lexer::TokenKind;

use crate::{
    ast::Block,
    errors::ParserError,
    parsers::{
        expr::Expr,
        local::{Local, local_impl},
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(Expr),
    Defer(Expr),
    Return(Option<Expr>),
    Break,
    Continue,
    Local(Local),
    Loop(Block),
}

// stmt = local | defer | expr;
pub fn stmt_impl<'tokens, 'src: 'tokens, I>(
    expr: impl Parser<'tokens, I, Expr, extra::Err<ParserError<'tokens, 'src>>> + Clone,
    block: impl Parser<'tokens, I, Block, extra::Err<ParserError<'tokens, 'src>>> + Clone,
) -> impl Parser<'tokens, I, Stmt, extra::Err<ParserError<'tokens, 'src>>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    let loop_stmt = just(TokenKind::Loop).ignore_then(block).map(Stmt::Loop);

    // defer = "defer", expr;
    let defer = just(TokenKind::Defer)
        .ignore_then(expr.clone())
        .map(Stmt::Defer);

    // return_stmt = "return", [ expr ];
    let return_stmt = just(TokenKind::Return)
        .ignore_then(expr.clone().or_not())
        .map(Stmt::Return);

    local_impl(expr.clone())
        .map(Stmt::Local)
        .or(defer)
        .or(return_stmt)
        .or(loop_stmt)
        .or(just(TokenKind::Break).to(Stmt::Break))
        .or(just(TokenKind::Continue).to(Stmt::Continue))
        .or(expr.map(Stmt::Expr))
}
