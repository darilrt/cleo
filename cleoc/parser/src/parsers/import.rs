use chumsky::{IterParser, Parser, extra, input::ValueInput, prelude::just, span::SimpleSpan};
use lexer::TokenKind;

use crate::{ast::Ident, errors::ParserError, parsers::ident::ident};

#[derive(Debug, Clone, PartialEq)]
pub struct ImportDecl {
    pub path: Vec<Ident>,
}

// import_decl = "import", import_path;
pub fn import_decl<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, ImportDecl, extra::Err<ParserError<'tokens, 'src>>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    // import_path = ident, { ".", ident };
    let import_path = ident()
        .separated_by(just(TokenKind::Dot))
        .at_least(1)
        .collect()
        .then_ignore(just(TokenKind::Semicolon).or_not());

    just(TokenKind::Import)
        .ignore_then(import_path)
        .map(|path| ImportDecl { path })
}
