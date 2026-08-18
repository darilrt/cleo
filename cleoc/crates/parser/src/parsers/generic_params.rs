use chumsky::{IterParser, Parser, input::ValueInput, prelude::just, span::SimpleSpan};
use lexer::TokenKind;

use crate::{
    errors::BoxedParser,
    parsers::{
        ident::{Ident, ident},
        ptype::Type,
    },
    test_parser, type_parser,
};

#[derive(Debug, Clone, PartialEq)]
pub struct GenericParams(pub Vec<GenericParam>);

#[derive(Debug, Clone, PartialEq)]
pub struct GenericParam {
    pub name: Ident,
    pub bound: Option<Type>,
}

// generic_params = "[", [ generic_param, { ",", generic_param }, [ "," ] ], "]";
pub fn generic_params<'tokens, 'src: 'tokens, I>() -> BoxedParser<'tokens, 'src, I, GenericParams>
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    let ptype = type_parser!();

    // generic_param = ident, [ ":", type ];
    let generic_param = ident()
        .then(just(TokenKind::Colon).ignore_then(ptype.clone()).or_not())
        .map(|(name, bound)| GenericParam { name, bound });

    generic_param
        .separated_by(just(TokenKind::Comma))
        .allow_trailing()
        .collect::<Vec<GenericParam>>()
        .delimited_by(just(TokenKind::LeftBracket), just(TokenKind::RightBracket))
        .map(GenericParams)
        .boxed()
}

test_parser!(generic_params() => GenericParams);

mod test {

    #[test]
    fn test_generic_params() {
        use super::test_parse;
        use crate::parsers::generic_params::{GenericParam, GenericParams};
        use crate::parsers::ident::Ident;
        use crate::parsers::path::{PathExpr, Segment};
        use crate::parsers::ptype::Type;

        let input = "[T, U: *const i32, V]";
        let result = test_parse(input).unwrap();

        let expected = GenericParams(vec![
            GenericParam {
                name: Ident::new("T"),
                bound: None,
            },
            GenericParam {
                name: Ident::new("U"),
                bound: Some(Type::ConstPtr(Box::new(Type::Path(PathExpr {
                    segments: vec![Segment {
                        name: Ident::new("i32"),
                        generics: None,
                    }],
                })))),
            },
            GenericParam {
                name: Ident::new("V"),
                bound: None,
            },
        ]);

        assert_eq!(result, expected);
    }
}
