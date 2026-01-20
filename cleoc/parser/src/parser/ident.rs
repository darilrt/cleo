use chumsky::select;
use lexer::TokenKind;

use crate::fn_parser;

#[derive(Debug, Clone, PartialEq)]
pub struct Ident {
    pub name: String,
}

fn_parser!(ident -> Ident {
    select! { TokenKind::Ident(name) => Ident { name: name.to_string() } }
});
