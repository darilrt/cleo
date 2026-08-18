use chumsky::{IterParser, Parser, extra, input::ValueInput, prelude::just, span::SimpleSpan};
use lexer::TokenKind;

use crate::{
    errors::{BoxedParser, ParserError},
    parsers::{
        ident::{Ident, ident},
        ptype::Type,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct PathExpr {
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    pub name: Ident,
    pub generics: Option<Vec<Type>>,
}

// segment := ident [ "[" generic_list "]" ]
pub fn segment<'tokens, 'src: 'tokens, I>(
    ptype: impl chumsky::Parser<'tokens, I, Type, extra::Err<ParserError<'tokens, 'src>>> + Clone + 'tokens,
) -> BoxedParser<'tokens, 'src, I, Segment>
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

    ident()
        .then(
            generic_list
                .delimited_by(just(TokenKind::LeftBracket), just(TokenKind::RightBracket))
                .or_not(),
        )
        .map(|(name, generics)| Segment { name, generics })
        .boxed()
}

// path := segment ("." segment)*
//       | "self" ("." segment)*
//
// `self` is only valid as the first segment of a path (e.g. self.field, self.method()).
// It cannot appear after a dot — that is a semantic error caught by the analyzer.
pub fn path_impl<'tokens, 'src: 'tokens, I>(
    ptype: impl chumsky::Parser<'tokens, I, Type, extra::Err<ParserError<'tokens, 'src>>> + Clone + 'tokens,
) -> BoxedParser<'tokens, 'src, I, PathExpr>
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    let seg = segment(ptype);

    // `self` as first segment, followed by zero or more ".segment" continuations.
    let self_path = just(TokenKind::SelfKw)
        .map(|_| Segment { name: Ident::new("self"), generics: None })
        .then(
            just(TokenKind::Dot)
                .ignore_then(seg.clone())
                .repeated()
                .collect::<Vec<_>>(),
        )
        .map(|(first, rest)| {
            let mut segments = vec![first];
            segments.extend(rest);
            PathExpr { segments }
        });

    // Regular path: one or more segments separated by dots.
    let regular_path = seg
        .separated_by(just(TokenKind::Dot))
        .at_least(1)
        .collect::<Vec<_>>()
        .map(|segments| PathExpr { segments });

    self_path.or(regular_path).boxed()
}

#[macro_export]
macro_rules! path_parser {
    () => {
        $crate::parsers::ptype::make_parsers().1
    };
}

#[allow(dead_code)]
pub(crate) fn test_parse<'a>(source: &'a str) -> crate::errors::Result<'a, PathExpr> {
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
        let res: PathExpr = test_parse("module.Type[i32].method[A.B, C]").unwrap();

        match res {
            PathExpr { segments } => {
                assert_eq!(segments.len(), 3);

                assert_eq!(segments[0].name.name, "module".to_string());
                assert_eq!(segments[0].generics, None);

                assert_eq!(segments[1].name.name, "Type".to_string());
                assert!(segments[1].generics.is_some());

                let generics = segments[1].generics.as_ref().unwrap();
                assert_eq!(generics.len(), 1);
                assert_eq!(
                    generics[0],
                    Type::Path(PathExpr {
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
