#[macro_export]
macro_rules! fn_parser {
    ($name:ident -> $output:ty $body:block) => {
        pub fn $name<'tokens, 'src: 'tokens, I>() -> impl chumsky::Parser<'tokens, I, $output, chumsky::extra::Err<crate::errors::ParserError<'tokens, 'src>>> + Clone
        where
            I: chumsky::input::ValueInput<'tokens, Token = lexer::TokenKind<'src>, Span = chumsky::span::SimpleSpan>,
        $body

        #[allow(dead_code)]
        pub(crate) fn test_parse<'a>(source: &'a str) -> crate::errors::Result<'a, $output> {
            use chumsky::{span::{SimpleSpan, Span}, input::Input, Parser};
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

            let result = $name().parse(stream);

            if result.has_errors() {
                let err = result
                    .errors()
                    .map(|e| e.clone().into_owned())
                    .collect::<Vec<_>>();
                return Err(crate::errors::Kind::ParseError(err));
            }

            Ok(result.output().unwrap().to_owned())
        }
    };
}
