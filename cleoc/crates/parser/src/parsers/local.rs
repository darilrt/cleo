use chumsky::{Parser, extra, input::ValueInput, prelude::just, span::SimpleSpan};
use lexer::TokenKind;

use crate::{
    ast::{Expr, Ident, Type},
    errors::{BoxedParser, ParserError},
    parsers::{expr::expr, ident::ident},
    test_parser, type_parser,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Local {
    pub binding: Binding,
    pub name: Ident,
    pub var_type: Option<Type>,
    pub initializer: Option<Box<Expr>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Binding {
    Var,
    Const,
}

// local = ( "var" | "const" ), ident, [ ":", type ], [ "=", expr ];
pub fn local_impl<'tokens, 'src: 'tokens, I>(
    expr: impl Parser<'tokens, I, Expr, extra::Err<ParserError<'tokens, 'src>>> + Clone + 'tokens,
) -> BoxedParser<'tokens, 'src, I, Local>
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    let binding = chumsky::select! {
        TokenKind::Var => Binding::Var,
        TokenKind::Const => Binding::Const,
    };

    let local_type = just(TokenKind::Colon).ignore_then(type_parser!()).or_not();

    let local_init = just(TokenKind::Equal)
        .ignore_then(expr)
        .or_not()
        .map(|e| e.map(Box::new));

    binding
        .then(ident())
        .then(local_type)
        .then(local_init)
        .map(|(((binding, name), var_type), initializer)| Local {
            binding,
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

mod test {

    #[test]
    fn test_local_decl() {
        use super::*;
        use crate::unwrap_or_report;
        use crate::{
            ast::ExprValue,
            parsers::path::{PathExpr, Segment},
        };

        let source = "var x: i32 = 42";

        let result = test_parse(source);

        assert_eq!(
            unwrap_or_report!(result, source),
            Local {
                binding: Binding::Var,
                name: Ident::new("x"),
                var_type: Some(Type::Path(PathExpr {
                    segments: vec![Segment {
                        name: Ident::new("i32"),
                        generics: None,
                    }],
                })),
                initializer: Some(Box::new(Expr::Value(ExprValue::Integer("42".to_string())))),
            }
        );
    }
}
