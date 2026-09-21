use ast::{FnDecl, FnParam, FnSignature, Ident};
use chumsky::{IterParser, Parser, input::ValueInput, prelude::just, span::SimpleSpan};
use lexer::TokenKind;

use crate::{
    errors::BoxedParser,
    parsers::{block::block, generic_params::generic_params, ident::ident},
    test_parser, type_parser,
};

// signature := ident generic_list? parameter_list return_type?
pub fn fn_signature<'tokens, 'src: 'tokens, I>() -> BoxedParser<'tokens, 'src, I, FnSignature>
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    let ptype = type_parser!();

    // self_param := 'self' ':' type
    // Only valid as the first parameter. SelfKw will not match ident(), so
    // writing `self` in any other position is a parse error.
    let self_param = just(TokenKind::SelfKw)
        .ignore_then(just(TokenKind::Colon))
        .ignore_then(ptype.clone())
        .map(|ty| FnParam {
            name: Ident::new("self"),
            ty,
        });

    // regular_param := ident ':' type
    let regular_param = ident()
        .then_ignore(just(TokenKind::Colon))
        .then(ptype.clone())
        .map(|(name, ty)| FnParam { name, ty });

    // parameter_list :=
    //   '(' ')'
    //   '(' self_param (',' regular_param)* ','? ')'
    //   '(' regular_param (',' regular_param)* ','? ')'
    let parameter_list = self_param
        .then(
            just(TokenKind::Comma)
                .ignore_then(
                    regular_param
                        .clone()
                        .separated_by(just(TokenKind::Comma))
                        .allow_trailing()
                        .collect::<Vec<FnParam>>(),
                )
                .or_not()
                .map(|v| v.unwrap_or_default()),
        )
        .map(|(self_p, mut rest)| {
            rest.insert(0, self_p);
            rest
        })
        .or(regular_param
            .separated_by(just(TokenKind::Comma))
            .allow_trailing()
            .collect::<Vec<FnParam>>())
        .delimited_by(just(TokenKind::LeftParen), just(TokenKind::RightParen));

    ident()
        .then(generic_params().or_not())
        .then(parameter_list)
        .then(ptype.clone().or_not())
        .map(
            |(((name, generics), parameters), return_type)| FnSignature {
                name,
                generics,
                params: parameters,
                return_type,
            },
        )
        .boxed()
}

// fn_decl = "fn", fn_signature, block;
pub fn fn_decl<'tokens, 'src: 'tokens, I>() -> BoxedParser<'tokens, 'src, I, FnDecl>
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    just(TokenKind::Fn)
        .ignore_then(fn_signature())
        .then(block())
        .map(|(signature, block)| FnDecl { signature, block })
        .boxed()
}

test_parser!(fn_decl() => FnDecl);

mod test {

    #[test]
    fn test_fn_decl_parser() {
        use super::*;
        use ast::{Block, GenericParam, GenericParams, PathExpr, Segment, Type};

        let source = "fn foo[T: int, U](x: int, y: U) int { }";
        let result = test_parse(source);

        let fn_decl = result.unwrap();

        assert_eq!(
            fn_decl,
            FnDecl {
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
                    params: vec![
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
