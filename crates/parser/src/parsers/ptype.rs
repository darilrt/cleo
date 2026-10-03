use ast::{PathExpr, Type};
use chumsky::{
    IterParser, Parser, extra,
    input::ValueInput,
    prelude::{Recursive, just, recursive},
    select,
    span::SimpleSpan,
};
use lexer::TokenKind;

use crate::{
    errors::{BoxedParser, ParserError},
    parsers::path::path_impl,
};

// type = { ptr_prefix | array_prefix }, basic_type
// basic_type = ["const"], path
// ptr_prefix = ["const"], "*"
// array_prefix = ["const"], "[", usize, "]"
pub fn ptype_impl<'tokens, 'src: 'tokens, I>(
    path: impl chumsky::Parser<'tokens, I, PathExpr, extra::Err<ParserError<'tokens, 'src>>> + Clone,
) -> impl Parser<'tokens, I, Type, extra::Err<ParserError<'tokens, 'src>>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    enum Seg {
        Ptr(bool),
        Array(bool, usize),
    }

    let basic_type = just(TokenKind::Const).or_not().then(path);

    let ptr_prefix = just(TokenKind::Const)
        .or_not()
        .then_ignore(just(TokenKind::Asterisk))
        .map(|cnst| Seg::Ptr(cnst.is_some()));

    let array_prefix = just(TokenKind::Const)
        .or_not()
        .then(
            select! {
                TokenKind::Integer(value) => value.parse::<usize>().unwrap(),
            }
            .delimited_by(just(TokenKind::LeftBracket), just(TokenKind::RightBracket)),
        )
        .map(|(cnst, size)| Seg::Array(cnst.is_some(), size));

    ptr_prefix
        .or(array_prefix)
        .repeated()
        .collect::<Vec<_>>()
        .then(basic_type)
        .map(|(segs, (cnst, path))| {
            segs.into_iter()
                .rev()
                .fold(Type::Path(cnst.is_some(), path), |inner, seg| match seg {
                    Seg::Array(is_const, size) => Type::Array(is_const, size, Box::new(inner)),
                    Seg::Ptr(is_const) => Type::Ptr(is_const, Box::new(inner)),
                })
        })
}

pub fn make_parsers<'tokens, 'src: 'tokens, I>() -> (
    BoxedParser<'tokens, 'src, I, Type>,
    BoxedParser<'tokens, 'src, I, PathExpr>,
)
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    let mut path_parser = Recursive::declare();
    let mut ptype_parser = Recursive::declare();

    path_parser.define(path_impl(ptype_parser.clone()));
    ptype_parser.define(ptype_impl(path_parser.clone()));

    (ptype_parser.boxed(), path_parser.boxed())
}

#[macro_export]
macro_rules! type_parser {
    () => {
        $crate::parsers::ptype::make_parsers().0
    };
}

#[allow(dead_code)]
pub(crate) fn test_parse<'a>(source: &'a str) -> crate::errors::Result<'a, Type> {
    use chumsky::{
        Parser,
        input::Input,
        span::{SimpleSpan, Span},
    };
    use lexer::lex;

    let lexed = lex(source)?;

    let stream = lexed.iter().map(|t| {
        (
            t.kind.clone(),
            SimpleSpan::new((), t.span.start..t.span.end),
        )
    });

    let stream = chumsky::input::Stream::from_iter(stream)
        .map((0..source.len()).into(), |(t, s): (_, _)| (t, s));

    let ptype = recursive(|t| {
        let path = path_impl(t);
        ptype_impl(path)
    });

    let result = ptype.parse(stream);

    if result.has_errors() {
        let err = result
            .errors()
            .map(|e| e.clone().into_owned())
            .collect::<Vec<_>>();
        return Err(crate::errors::Kind::ParseError(err));
    }

    Ok(result.output().unwrap().to_owned())
}

mod test {
    #[allow(unused)]
    use super::*;
    #[allow(unused)]
    #[test]
    fn test_type() {
        use ast::{Ident, Segment};
        let test = test_parse("[1][5]*const A.B[*u8, i32]").unwrap();

        assert_eq!(
            test,
            Type::Array(
                false,
                1,
                Box::new(Type::Array(
                    false,
                    5,
                    Box::new(Type::Ptr(
                        false,
                        Box::new(Type::Path(
                            true,
                            PathExpr {
                                segments: vec![
                                    Segment {
                                        name: Ident {
                                            name: "A".to_string(),
                                        },
                                        generics: None,
                                    },
                                    Segment {
                                        name: Ident {
                                            name: "B".to_string(),
                                        },
                                        generics: Some(vec![
                                            Type::Ptr(
                                                false,
                                                Box::new(Type::Path(
                                                    false,
                                                    PathExpr {
                                                        segments: vec![Segment {
                                                            name: Ident {
                                                                name: "u8".to_string(),
                                                            },
                                                            generics: None,
                                                        }]
                                                    }
                                                ))
                                            ),
                                            Type::Path(
                                                false,
                                                PathExpr {
                                                    segments: vec![Segment {
                                                        name: Ident {
                                                            name: "i32".to_string(),
                                                        },
                                                        generics: None,
                                                    }]
                                                }
                                            ),
                                        ]),
                                    }
                                ]
                            }
                        ))
                    ))
                ))
            )
        );
    }
}
