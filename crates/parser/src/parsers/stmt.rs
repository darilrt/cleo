use ast::{Expr, Stmt};
use chumsky::{Parser, extra, input::ValueInput, prelude::just, span::SimpleSpan};
use lexer::TokenKind;

use crate::{
    errors::{BoxedParser, ParserError},
    parsers::local::local_impl,
};

// stmt = local | defer | return | break | continue | expr;
pub fn stmt_impl<'tokens, 'src: 'tokens, I>(
    expr: impl Parser<'tokens, I, Expr, extra::Err<ParserError<'tokens, 'src>>> + Clone + 'tokens,
) -> BoxedParser<'tokens, 'src, I, Stmt>
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
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
        .or(just(TokenKind::Break).to(Stmt::Break))
        .or(just(TokenKind::Continue).to(Stmt::Continue))
        .or(expr.map(Stmt::Expr))
        .boxed()
}
