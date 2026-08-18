#[macro_export]
macro_rules! recursive_parser {
    ($firt:ident, $second:ident) => {
        chumsky::recursive(|$first| {
            let tmp = $second($first);
            $first(tmp)
        })
    };
}

#[macro_export]
macro_rules! test_parser {
    ($name:expr => $output:ty) => {
        #[allow(dead_code)]
        pub(crate) fn test_parse<'a>(source: &'a str) -> $crate::errors::Result<'a, $output> {
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

            let result = $name.parse(stream);

            if result.has_errors() {
                let err = result
                    .errors()
                    .map(|e| e.clone().into_owned())
                    .collect::<Vec<_>>();
                return Err($crate::errors::Kind::ParseError(err));
            }

            Ok(result.output().unwrap().to_owned())
        }
    };
}

#[macro_export]
macro_rules! unwrap_or_report {
    ($result:expr, $source:expr) => {{
        use ariadne::{Color, Label, Report, ReportKind, Source};

        match $result {
            Ok(r) => r,
            Err(kind) => {
                match kind {
                    $crate::errors::Kind::LexError(e) => {
                        println!("Lexer Error: {:?}", e);
                    }
                    $crate::errors::Kind::ParseError(errs) => {
                        errs.into_iter().for_each(|e| {
                            Report::build(ReportKind::Error, ((), e.span().into_range()))
                                .with_config(
                                    ariadne::Config::new()
                                        .with_index_type(ariadne::IndexType::Byte),
                                )
                                .with_message(e.clone())
                                .with_label(
                                    Label::new(((), e.span().into_range()))
                                        .with_message(e.clone())
                                        .with_color(Color::Red),
                                )
                                .finish()
                                .print(Source::from(&$source))
                                .unwrap()
                        });
                    }
                }
                panic!("Parsing failed with errors.");
            }
        }
    }};
}

#[macro_export]
macro_rules! unwrap_or_report_file {
    ($result:expr, $file:expr, $source:expr) => {{
        use ariadne::{Color, Label, Report, ReportKind, Source};

        match $result {
            Ok(r) => r,
            Err(kind) => {
                match kind {
                    $crate::errors::Kind::LexError(e) => {
                        println!("Lexer Error: {:?}", e);
                    }
                    $crate::errors::Kind::ParseError(errs) => {
                        errs.into_iter().for_each(|e| {
                            Report::build(ReportKind::Error, ($file, e.span().into_range()))
                                .with_config(
                                    ariadne::Config::new()
                                        .with_index_type(ariadne::IndexType::Byte),
                                )
                                .with_message(e.clone())
                                .with_label(
                                    Label::new(($file, e.span().into_range()))
                                        .with_message(e.clone())
                                        .with_color(Color::Red),
                                )
                                .finish()
                                .print(($file, Source::from(&$source)))
                                .unwrap()
                        });
                    }
                }
                panic!("Parsing failed with errors.");
            }
        }
    }};
}
