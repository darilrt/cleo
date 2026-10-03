use ast::{Binding, Expr, Local};
use chumsky::{Parser, extra, input::ValueInput, prelude::just, span::SimpleSpan};
use lexer::TokenKind;

use crate::{
    errors::{BoxedParser, ParserError},
    parsers::{expr::expr, ident::ident},
    test_parser, type_parser,
};

// local = ( "var" | "const" ), ident, [ ":", type ], [ "=", expr ];
pub fn local_impl<'tokens, 'src: 'tokens, I>(
    expr: impl Parser<'tokens, I, Expr, extra::Err<ParserError<'tokens, 'src>>> + Clone + 'tokens,
) -> BoxedParser<'tokens, 'src, I, Local>
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    let local_type = type_parser!().or_not();

    let local_init = just(TokenKind::Equal)
        .ignore_then(expr)
        .or_not()
        .map(|e| e.map(Box::new));

    ident()
        .then_ignore(just(TokenKind::Colon))
        .then(local_type)
        .then(local_init)
        .map(|((name, var_type), initializer)| Local {
            binding: Binding::Var,
            name,
            var_type,
            initializer,
        })
        .boxed()
}

pub fn local<'tokens, 'src: 'tokens, I>() -> BoxedParser<'tokens, 'src, I, Local>
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    local_impl(expr())
}

test_parser!(local() => Local);
