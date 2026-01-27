use chumsky::{IterParser, Parser, extra, input::ValueInput, prelude::just, span::SimpleSpan};
use lexer::TokenKind;

use crate::{
    errors::ParserError,
    parsers::{
        block::{Block, block},
        generic_params::{GenericParams, generic_params},
        ident::{Ident, ident},
        ptype::Type,
    },
    test_parser, type_parser,
};

#[derive(Debug, Clone, PartialEq)]
pub struct FnDecl {
    pub inline: bool,
    pub signature: FnSignature,
    pub block: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnSignature {
    pub name: Ident,
    pub generics: Option<GenericParams>,
    pub parameters: Vec<FnParam>,
    pub return_type: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnParam {
    pub name: Ident,
    pub ty: Type,
}

// signature := ident generic_list? parameter_list return_type?
pub fn fn_signature<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, FnSignature, extra::Err<ParserError<'tokens, 'src>>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    let ptype = type_parser!();

    // param := ident ':' type
    let param = ident()
        .then_ignore(just(TokenKind::Colon))
        .then(ptype.clone())
        .map(|(name, ty)| FnParam { name, ty });

    // param_list := param (',' param)* (',')?
    let parameter_list = param
        .separated_by(just(TokenKind::Comma))
        .allow_trailing()
        .collect::<Vec<FnParam>>()
        .delimited_by(just(TokenKind::LeftParen), just(TokenKind::RightParen));

    ident()
        .then(generic_params().or_not())
        .then(parameter_list)
        .then(ptype.clone().or_not())
        .map(
            |(((name, generics), parameters), return_type)| FnSignature {
                name,
                generics,
                parameters,
                return_type,
            },
        )
}

// fn_decl = [ "inline" ], "fn", ident, [ generic_params ], fn_params, [ type ], block;
pub fn fn_decl<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, FnDecl, extra::Err<ParserError<'tokens, 'src>>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    just(TokenKind::Inline)
        .or_not()
        .map(|opt| opt.is_some())
        .then_ignore(just(TokenKind::Fn))
        .then(fn_signature())
        .then(block())
        .map(|((inline, signature), block)| FnDecl {
            inline,
            signature,
            block,
        })
}

test_parser!(fn_decl() => FnDecl);

mod test {

    #[test]
    fn test_fn_decl_parser() {
        use super::*;
        use crate::parsers::generic_params::GenericParam;
        use crate::parsers::path::{PathExpr, Segment};

        let source = "inline fn foo[T: int, U](x: int, y: U) int { }";
        let result = test_parse(source);

        let fn_decl = result.unwrap();

        assert_eq!(
            fn_decl,
            FnDecl {
                inline: true,
                signature: FnSignature {
                    name: Ident {
                        name: "foo".to_string()
                    },
                    generics: Some(GenericParams(vec![
                        GenericParam {
                            name: Ident {
                                name: "T".to_string()
                            },
                            bound: Some(Type::Path(PathExpr {
                                segments: vec![Segment {
                                    name: Ident {
                                        name: "int".to_string()
                                    },
                                    generics: None,
                                }]
                            })),
                        },
                        GenericParam {
                            name: Ident {
                                name: "U".to_string()
                            },
                            bound: None,
                        },
                    ])),
                    parameters: vec![
                        FnParam {
                            name: Ident {
                                name: "x".to_string()
                            },
                            ty: Type::Path(PathExpr {
                                segments: vec![Segment {
                                    name: Ident {
                                        name: "int".to_string()
                                    },
                                    generics: None,
                                }]
                            }),
                        },
                        FnParam {
                            name: Ident {
                                name: "y".to_string()
                            },
                            ty: Type::Path(PathExpr {
                                segments: vec![Segment {
                                    name: Ident {
                                        name: "U".to_string()
                                    },
                                    generics: None,
                                }]
                            }),
                        },
                    ],
                    return_type: Some(Type::Path(PathExpr {
                        segments: vec![Segment {
                            name: Ident {
                                name: "int".to_string()
                            },
                            generics: None,
                        }]
                    })),
                },
                block: Block { statements: vec![] }
            }
        )
    }
}
