pub(crate) type ParserError<'tokens, 'src> = chumsky::error::Rich<'tokens, lexer::TokenKind<'src>>;

/// Type-erased parser — breaks the monomorphization chain and dramatically
/// reduces compile times for complex combinator expressions.
pub(crate) type BoxedParser<'tokens, 'src, I, O> = chumsky::Boxed<
    'tokens,
    'tokens,
    I,
    O,
    chumsky::extra::Err<ParserError<'tokens, 'src>>,
>;

pub type Result<'a, T> = std::result::Result<T, Kind<'a, 'a>>;

#[derive(Debug)]
pub enum Kind<'a, 'src> {
    LexError(lexer::LexerError),
    ParseError(Vec<ParserError<'a, 'src>>),
}

impl From<lexer::LexerError> for Kind<'_, '_> {
    fn from(error: lexer::LexerError) -> Self {
        Kind::LexError(error)
    }
}
