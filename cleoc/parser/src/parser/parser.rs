use chumsky::{
    IterParser, Parser,
    prelude::{just, recursive},
    select,
};
use lexer::TokenKind;

use crate::{
    fn_parser,
    parser::expr::{Expr, expr},
};

#[derive(Debug, Clone)]
pub enum Decl {
    Dummy,
    Fn(FnDecl),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Dummy(String),
    Block(Block),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub struct FnDecl {
    pub signature: Signature,
    pub block: Block,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Signature {
    pub name: String,
    pub parameters: Vec<FnArg>,
    pub return_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FnArg {
    pub name: String,
    pub ty: String,
}

fn_parser!(parser -> Decl
{
    let ident = select! { TokenKind::Ident(name) => name };

    let r#type = ident.clone();

    let stmt = ident
        .clone()
        .map(|s| Stmt::Dummy(s.to_string()))
        .or(expr().map(|e| Stmt::Expr(e)));

    let block = recursive(|block| {
        stmt.clone()
            .or(block.map(|b| Stmt::Block(b)))
            .separated_by(just(TokenKind::Comma))
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just(TokenKind::LeftBrace), just(TokenKind::RightBrace))
            .map(|stmts| Block { statements: stmts })
    });

    let fn_arg = ident
        .clone()
        .then_ignore(just(TokenKind::Colon))
        .then(r#type.clone())
        .map(|(name, ty)| FnArg {
            name: name.to_string(),
            ty: ty.to_string(),
        });

    let parameter_list = fn_arg
        .separated_by(just(TokenKind::Comma))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just(TokenKind::LeftParen), just(TokenKind::RightParen));

    let fn_signature = ident
        .clone()
        .then(parameter_list)
        .then(r#type.clone().or_not())
        .map(|((name, parameters), return_type)| Signature {
            name: name.to_string(),
            parameters,
            return_type: return_type.map(|s| s.to_string()),
        });

    let fn_decl = just(TokenKind::Fn)
        .ignore_then(fn_signature)
        .then(block)
        .map(|(signature, block)| FnDecl { signature, block });

    let decl = fn_decl.map(Decl::Fn);

    decl
});

mod test {
    #[test]
    fn it_works() {
        let Ok(ast) = super::test_parse("fn hola(a: b, c: d) kkck { 123 }") else {
            panic!("Parsing failed");
        };

        println!("AST: {:?}", ast);
    }
}
