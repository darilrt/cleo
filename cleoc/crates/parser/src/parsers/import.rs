use ast::ImportDecl;
use chumsky::{IterParser, Parser, input::ValueInput, prelude::just, span::SimpleSpan};
use lexer::TokenKind;

use crate::{errors::BoxedParser, parsers::ident::ident};

// import_decl = "import", import_path;
pub fn import_decl<'tokens, 'src: 'tokens, I>() -> BoxedParser<'tokens, 'src, I, ImportDecl>
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
        .boxed()
}
