use chumsky::{
    Boxed, IterParser, Parser, extra,
    input::ValueInput,
    prelude::{Recursive, just, recursive},
    select,
    span::SimpleSpan,
};
use lexer::TokenKind;

use crate::{
    errors::ParserError,
    parser::path::{PathExpr, path_impl},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Ptr(Box<Type>),
    ConstPtr(Box<Type>),
    Path(PathExpr),
    Array(usize, Box<Type>),
}

// type := array ptr path
pub fn ptype_impl<'tokens, 'src: 'tokens, I>(
    path: impl chumsky::Parser<'tokens, I, PathExpr, extra::Err<ParserError<'tokens, 'src>>> + Clone,
) -> impl Parser<'tokens, I, Type, extra::Err<ParserError<'tokens, 'src>>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    enum PtrType {
        Ptr,
        ConstPtr,
    }

    // array := ("[" usize "]")*
    let array = select! {
        TokenKind::Integer(value) => value.parse::<usize>().unwrap(),
    }
    .delimited_by(just(TokenKind::LeftBracket), just(TokenKind::RightBracket))
    .repeated()
    .collect::<Vec<usize>>();

    // ptr := ("*" | ("*" "const"))*
    let ptr = just(TokenKind::Asterisk)
        .then(just(TokenKind::Const).or_not())
        .map(|(_, opt_const)| match opt_const {
            Some(_) => PtrType::ConstPtr,
            None => PtrType::Ptr,
        })
        .repeated()
        .collect::<Vec<_>>();

    array
        .or_not()
        .then(ptr.or_not())
        .then(path)
        .map(|((array, ptr), p)| {
            let mut ty = Type::Path(p);

            if let Some(ptr_kinds) = ptr {
                for pk in ptr_kinds.into_iter().rev() {
                    ty = match pk {
                        PtrType::Ptr => Type::Ptr(Box::new(ty)),
                        PtrType::ConstPtr => Type::ConstPtr(Box::new(ty)),
                    }
                }
            }

            if let Some(array) = array {
                for size in array.into_iter().rev() {
                    ty = Type::Array(size, Box::new(ty));
                }
            }

            ty
        })
}

type BoxedParser<'tokens, 'src, I, T> =
    Boxed<'tokens, 'tokens, I, T, extra::Err<ParserError<'tokens, 'src>>>;

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
        $crate::parser::ptype::make_parsers().0
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
    use crate::parser::{ident::Ident, path::Segment};

    #[test]
    fn test_type() {
        let test = test_parse("[1][5]*const A.B[*u8, i32]").unwrap();

        assert_eq!(
            test,
            Type::Array(
                1,
                Box::new(Type::Array(
                    5,
                    Box::new(Type::ConstPtr(Box::new(Type::Path(PathExpr {
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
                                    Type::Ptr(Box::new(Type::Path(PathExpr {
                                        segments: vec![Segment {
                                            name: Ident {
                                                name: "u8".to_string(),
                                            },
                                            generics: None,
                                        }]
                                    }))),
                                    Type::Path(PathExpr {
                                        segments: vec![Segment {
                                            name: Ident {
                                                name: "i32".to_string(),
                                            },
                                            generics: None,
                                        }]
                                    }),
                                ]),
                            }
                        ]
                    }))))
                ))
            )
        );
    }
}
