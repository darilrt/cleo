use chumsky::{
    Boxed, IterParser, Parser, extra,
    input::ValueInput,
    prelude::{just, recursive},
    span::SimpleSpan,
};
use lexer::TokenKind;

use crate::{
    errors::ParserError,
    parser::{
        ident::{Ident, ident},
        ptype::{Type, make_parsers},
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    pub name: Ident,
    pub generics: Option<Vec<Type>>,
}

// path := segment ("." segment)*
pub fn path_impl<'tokens, 'src: 'tokens, I>(
    ptype: impl chumsky::Parser<'tokens, I, Type, extra::Err<ParserError<'tokens, 'src>>> + Clone,
) -> impl Parser<'tokens, I, Path, extra::Err<ParserError<'tokens, 'src>>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    // generic_list := type ("," type)*
    let generic_list = ptype
        .clone()
        .separated_by(just(TokenKind::Comma))
        .allow_trailing()
        .at_least(1)
        .collect::<Vec<_>>();

    // segment := ident [ "[" generic_list "]" ]
    let segment = ident()
        .then(
            generic_list
                .delimited_by(just(TokenKind::LeftBracket), just(TokenKind::RightBracket))
                .or_not(),
        )
        .map(|(name, generics)| Segment { name, generics });

    segment
        .separated_by(just(TokenKind::Dot))
        .at_least(1)
        .collect::<Vec<_>>()
        .map(|segments| Path { segments })
}

#[macro_export]
macro_rules! path_parser {
    () => {
        make_parsers().1
    };
}

#[allow(dead_code)]
pub(crate) fn test_parse<'a>(source: &'a str) -> crate::errors::Result<'a, Path> {
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

    let result = path_parser!().parse(stream);

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
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_path_parser() {
        let res: Path = test_parse("module.Type[i32].method[A.B, C]").unwrap();

        match res {
            Path { segments } => {
                assert_eq!(segments.len(), 3);

                assert_eq!(segments[0].name.name, "module".to_string());
                assert_eq!(segments[0].generics, None);

                assert_eq!(segments[1].name.name, "Type".to_string());
                assert!(segments[1].generics.is_some());

                let generics = segments[1].generics.as_ref().unwrap();
                assert_eq!(generics.len(), 1);
                assert_eq!(
                    generics[0],
                    Type::Path(Path {
                        segments: vec![Segment {
                            name: Ident {
                                name: "i32".to_string(),
                            },
                            generics: None,
                        }]
                    })
                );
                assert_eq!(segments[2].name.name, "method".to_string());
                assert!(segments[2].generics.is_some());
                let generics = segments[2].generics.as_ref().unwrap();
                assert_eq!(generics.len(), 2);
            }
        }
    }
}
