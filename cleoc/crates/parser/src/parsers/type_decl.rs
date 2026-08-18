use chumsky::{IterParser, Parser, input::ValueInput, prelude::just, span::SimpleSpan};
use lexer::TokenKind;

use crate::{
    errors::BoxedParser,
    parsers::{
        fn_decl::{FnSignature, fn_signature},
        generic_params::{GenericParams, generic_params},
        ident::{Ident, ident},
        ptype::Type,
    },
    test_parser, type_parser,
};

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDecl {
    pub name: Ident,
    pub generics: Option<GenericParams>,
    pub body: TypeBody,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeBody {
    Struct(Vec<StructField>),
    Alias(Type),
    Trait(Vec<TraitMethod>),
    Enum(Vec<EnumValue>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumValue {
    pub name: Ident,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: Ident,
    pub field_type: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitMethod {
    pub signature: FnSignature,
}

pub fn type_decl<'tokens, 'src: 'tokens, I>() -> BoxedParser<'tokens, 'src, I, TypeDecl>
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    let ptype = type_parser!();

    // TODO: Add support for tagged unions with union keyword

    let struct_field = ident()
        .then_ignore(just(TokenKind::Colon))
        .then(ptype.clone())
        .map(|(name, field_type)| StructField { name, field_type });

    let struct_decl = just(TokenKind::Struct).ignore_then(
        struct_field
            .separated_by(just(TokenKind::Semicolon))
            .allow_trailing()
            .collect::<Vec<StructField>>()
            .delimited_by(just(TokenKind::LeftBrace), just(TokenKind::RightBrace)),
    );

    let enum_variant = ident().map(|name| EnumValue { name });

    let enum_decl = just(TokenKind::Enum).ignore_then(
        enum_variant
            .separated_by(just(TokenKind::Semicolon))
            .allow_trailing()
            .collect::<Vec<EnumValue>>()
            .delimited_by(just(TokenKind::LeftBrace), just(TokenKind::RightBrace)),
    );

    let trait_method = just(TokenKind::Fn)
        .ignore_then(fn_signature())
        .map(|signature| TraitMethod { signature });

    let trait_decl = just(TokenKind::Trait).ignore_then(
        trait_method
            .separated_by(just(TokenKind::Semicolon))
            .allow_trailing()
            .collect::<Vec<TraitMethod>>()
            .delimited_by(just(TokenKind::LeftBrace), just(TokenKind::RightBrace)),
    );

    let type_body = struct_decl
        .map(TypeBody::Struct)
        .or(trait_decl.map(TypeBody::Trait))
        .or(ptype.clone().map(TypeBody::Alias))
        .or(enum_decl.map(TypeBody::Enum));

    just(TokenKind::Type)
        .ignore_then(ident())
        .then(generic_params().or_not())
        .then_ignore(just(TokenKind::Equal))
        .then(type_body)
        .map(|((name, generics), body)| TypeDecl {
            name,
            generics,
            body,
        })
        .boxed()
}

test_parser!(type_decl() => TypeDecl);

mod test {
    #[allow(unused_imports)]
    use crate::unwrap_or_report;

    #[test]
    fn test_enum() {
        use super::*;

        let source = r#"type Color = enum {
    Red,
    Green,
}"#;

        let result = test_parse(source);

        assert_eq!(
            unwrap_or_report!(result, source),
            TypeDecl {
                name: Ident::new("Color"),
                generics: None,
                body: TypeBody::Enum(vec![
                    EnumValue {
                        name: Ident::new("Red"),
                    },
                    EnumValue {
                        name: Ident::new("Green"),
                    },
                ],),
            }
        );
    }

    #[test]
    fn test_trait() {
        use super::*;
        use crate::parsers::fn_decl::FnSignature;

        let source = r#"type Drawable = trait { 
    fn draw()
    fn size() i32
}"#;

        let result = test_parse(source);

        assert_eq!(
            unwrap_or_report!(result, source),
            TypeDecl {
                name: Ident::new("Drawable"),
                generics: None,
                body: TypeBody::Trait(vec![
                    TraitMethod {
                        signature: FnSignature {
                            name: Ident::new("draw"),
                            generics: None,
                            params: vec![],
                            return_type: None,
                        },
                    },
                    TraitMethod {
                        signature: FnSignature {
                            name: Ident::new("size"),
                            generics: None,
                            params: vec![],
                            return_type: Some(Type::Path(crate::parsers::path::PathExpr {
                                segments: vec![crate::parsers::path::Segment {
                                    name: Ident::new("i32"),
                                    generics: None,
                                },],
                            },)),
                        },
                    }
                ],),
            }
        );
    }

    #[test]
    fn test_alias() {
        use super::*;
        use crate::parsers::path::{PathExpr, Segment};

        let source = r#"type Alias = Point[Int]"#;

        let result = test_parse(source);

        assert_eq!(
            unwrap_or_report!(result, source),
            TypeDecl {
                name: Ident::new("Alias"),
                generics: None,
                body: TypeBody::Alias(Type::Path(PathExpr {
                    segments: vec![Segment {
                        name: Ident::new("Point"),
                        generics: Some(vec![Type::Path(PathExpr {
                            segments: vec![Segment {
                                name: Ident::new("Int"),
                                generics: None,
                            },],
                        },),],),
                    },],
                },),),
            }
        );
    }

    #[test]
    fn test_struct() {
        use super::*;
        use crate::parsers::{
            generic_params::GenericParam,
            path::{PathExpr, Segment},
        };

        let source = r#"type Point[T] = struct {
    x: Int,
    y: Int,
}"#;

        let result = test_parse(source);

        assert!(result.is_ok());

        assert_eq!(
            result.unwrap(),
            TypeDecl {
                name: Ident::new("Point"),
                generics: Some(GenericParams(vec![GenericParam {
                    name: Ident::new("T"),
                    bound: None,
                },],)),
                body: TypeBody::Struct(vec![
                    StructField {
                        name: Ident::new("x"),
                        field_type: Type::Path(PathExpr {
                            segments: vec![Segment {
                                name: Ident::new("Int"),
                                generics: None,
                            },],
                        },),
                    },
                    StructField {
                        name: Ident::new("y"),
                        field_type: Type::Path(PathExpr {
                            segments: vec![Segment {
                                name: Ident::new("Int"),
                                generics: None,
                            },],
                        },),
                    },
                ],),
            }
        );
    }
}
