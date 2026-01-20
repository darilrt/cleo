use chumsky::{
    Parser,
    input::Input,
    span::{SimpleSpan, Span},
};

use crate::{
    errors::{self, Kind},
    parser::parser::{Decl, parser},
};

mod expr;
mod ident;
mod parser;
mod path;
mod ptype;

pub fn parse<'a>(source: &'a str) -> errors::Result<'a, Decl> {
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

    let result = parser().parse(stream);

    if result.has_errors() {
        let err = result
            .errors()
            .map(|e| e.clone().into_owned())
            .collect::<Vec<_>>();
        return Err(Kind::ParseError(err));
    }

    Ok(result.output().unwrap().to_owned())
}
