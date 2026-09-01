use ast::{
    AssignKind, Expr, ExprAccess, ExprAssign, ExprCall, ExprIf, ExprInit, ExprInitField, ExprValue,
    Operator, Segment,
};
use chumsky::{
    IterParser, Parser,
    input::ValueInput,
    prelude::{just, recursive},
    select,
    span::SimpleSpan,
};
use lexer::TokenKind;

use crate::{
    errors::BoxedParser,
    parsers::{block::block_impl, ident::ident, path::segment},
    path_parser, type_parser,
};

// expr = if_expr | assign;
pub fn expr<'tokens, 'src: 'tokens, I>() -> BoxedParser<'tokens, 'src, I, Expr>
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    recursive(|expr| {
        let block = block_impl(expr.clone());
        let path = path_parser!();
        let ptype = type_parser!();

        let struct_init_fields = ident()
            .then_ignore(just(TokenKind::Equal))
            .then(expr.clone())
            .map(|(seg, value)| ExprInitField {
                name: seg,
                value: Box::new(value),
            });

        let struct_init = path
            .clone()
            .then(
                struct_init_fields
                    .separated_by(just(TokenKind::Comma))
                    .allow_trailing()
                    .collect()
                    .delimited_by(just(TokenKind::LeftBrace), just(TokenKind::RightBrace)),
            )
            .map(|(path, fields)| ExprInit { path, fields });

        // Control Flow
        let if_expr = just(TokenKind::If)
            .ignore_then(expr.clone())
            .then(block.clone())
            .then(just(TokenKind::Else).ignore_then(block.clone()).or_not())
            .map(|((condition, then_branch), else_branch)| {
                Expr::If(ExprIf {
                    condition: Box::new(condition),
                    then_branch,
                    else_branch,
                })
            });

        let loop_expr = just(TokenKind::Loop).ignore_then(block).map(Expr::Loop);

        // primary := integer | float | string | bool | "(" expr ")" | struct_init | path
        // Boxed to cut the monomorphization chain — it has many .or() branches.
        let primary: BoxedParser<'tokens, 'src, I, Expr> = select! {
            TokenKind::Integer(v) => Expr::Value(ExprValue::Integer(v.to_string())),
            TokenKind::Float(v) => Expr::Value(ExprValue::Float(v.to_string())),
            TokenKind::String(v) => Expr::Value(ExprValue::String(v.to_string())),
            TokenKind::Bool(v) => Expr::Value(ExprValue::Bool(v == "true")),
        }
        .or(expr
            .clone()
            .delimited_by(just(TokenKind::LeftParen), just(TokenKind::RightParen)))
        .or(struct_init.map(Expr::Init))
        .or(path.clone().map(Expr::Path))
        .boxed();

        // call := "(", [ expr, { ",", expr }, [ "," ] ], ")"
        let call = expr
            .clone()
            .separated_by(just(TokenKind::Comma))
            .allow_trailing()
            .collect::<Vec<Expr>>()
            .delimited_by(just(TokenKind::LeftParen), just(TokenKind::RightParen));

        // field_access := ".", segment
        let field_access = just(TokenKind::Dot).ignore_then(segment(ptype.clone()));

        enum Accessor {
            Call(Vec<Expr>),
            FieldAccess(Segment),
        }

        // accessor := call | field_access
        let accessor = field_access
            .map(Accessor::FieldAccess)
            .or(call.map(Accessor::Call));

        // factor := primary, { accessor }
        // Boxed to erase the type after folding accessors.
        let factor: BoxedParser<'tokens, 'src, I, Expr> = primary
            .then(accessor.repeated().collect::<Vec<Accessor>>())
            .map(|(base, accessors)| {
                accessors
                    .into_iter()
                    .fold(base, |acc, accessor| match accessor {
                        Accessor::Call(args) => Expr::Call(ExprCall {
                            expr: Box::new(acc),
                            args,
                        }),
                        Accessor::FieldAccess(seg) => Expr::Access(ExprAccess {
                            expr: Box::new(acc),
                            segment: seg,
                        }),
                    })
            })
            .boxed();

        // unary = ( "*" | "&" | "!" | "-" ), factor | factor
        let unary: BoxedParser<'tokens, 'src, I, Expr> = select! {
            TokenKind::Asterisk => Operator::Deref,
            TokenKind::And => Operator::Ref,
            TokenKind::Minus => Operator::Neg,
            TokenKind::Not => Operator::Not,
        }
        .then(factor.clone())
        .map(|(op, expr)| Expr::UnaryOp {
            op,
            expr: Box::new(expr),
        })
        .or(factor.clone())
        .boxed();

        // term := unary, { ("*" | "/"), term}
        let term = unary.clone().foldl_with(
            select! {
                TokenKind::Asterisk => Operator::Mul,
                TokenKind::Slash => Operator::Div,
            }
            .then(unary.clone())
            .repeated(),
            |lhs, (op, rhs), _e| Expr::BinaryOp {
                left: Box::new(lhs),
                op,
                right: Box::new(rhs),
            },
        );

        // addition := term, { ("+" | "-"), term }
        let addition = term.clone().foldl_with(
            select! {
                TokenKind::Plus => Operator::Add,
                TokenKind::Minus => Operator::Sub,
            }
            .then(term.clone())
            .repeated(),
            |lhs, (op, rhs), _e| Expr::BinaryOp {
                left: Box::new(lhs),
                op,
                right: Box::new(rhs),
            },
        );

        let comp_op = select! {
            TokenKind::EqualEqual => Operator::Equal,
            TokenKind::NotEqual => Operator::NotEqual,
            TokenKind::LessThanEqual => Operator::LessEqual,
            TokenKind::GreaterThanEqual => Operator::GreaterEqual,
            TokenKind::LessThan => Operator::Less,
            TokenKind::GreaterThan => Operator::Greater,
        };

        // comparation = addition, [ ( "==" | "!=" | "<=" | ">=" | "<" | ">" ), addition ];
        let comparation = addition
            .clone()
            .then(comp_op.then(addition.clone()).or_not())
            .map(|(left, right)| {
                if let Some((op, right)) = right {
                    Expr::BinaryOp {
                        left: Box::new(left),
                        op,
                        right: Box::new(right),
                    }
                } else {
                    left
                }
            });

        // logic_and = comparation, { "&&", comparation }
        let logic_and = comparation.clone().foldl_with(
            select! { TokenKind::AndAnd => Operator::And }
                .then(comparation.clone())
                .repeated(),
            |lhs, (op, rhs), _e| Expr::BinaryOp {
                left: Box::new(lhs),
                op,
                right: Box::new(rhs),
            },
        );

        // logic_or = logic_and, { "||", logic_and }
        let logic_or = logic_and.clone().foldl_with(
            select! { TokenKind::OrOr => Operator::Or }
                .then(logic_and.clone())
                .repeated(),
            |lhs, (op, rhs), _e| Expr::BinaryOp {
                left: Box::new(lhs),
                op,
                right: Box::new(rhs),
            },
        );

        // assign_kind = "=" | "+=" | "-=" | "*=" | "/="
        let assign_kind = select! {
            TokenKind::Equal => AssignKind::Equal,
            TokenKind::PlusEqual => AssignKind::AddEqual,
            TokenKind::MinusEqual => AssignKind::SubEqual,
            TokenKind::AsteriskEqual => AssignKind::MulEqual,
            TokenKind::SlashEqual => AssignKind::DivEqual,
        };

        // assign = logic_or, [ assign_kind, expr ];
        let assign = logic_or
            .clone()
            .then(assign_kind.then(expr.clone()).or_not())
            .map(|(left, right)| {
                if let Some((kind, rhs)) = right {
                    Expr::Assign(ExprAssign {
                        left: Box::new(left),
                        kind,
                        right: Box::new(rhs),
                    })
                } else {
                    left
                }
            });

        if_expr.or(loop_expr).or(assign).or(logic_or)
    })
    .boxed()
}

#[allow(dead_code)]
pub(crate) fn test_parse<'a>(source: &'a str) -> crate::errors::Result<'a, Expr> {
    use chumsky::{
        Parser,
        input::Input,
        span::{SimpleSpan, Span},
    };
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

    let result = expr().parse(stream);

    if result.has_errors() {
        let err = result
            .errors()
            .map(|e| e.clone().into_owned())
            .collect::<Vec<_>>();
        return Err(crate::errors::Kind::ParseError(err));
    }

    Ok(result.output().unwrap().to_owned())
}

#[allow(unused_imports)]
mod test {
    use ast::{Ident, PathExpr};

    use super::*;
    use crate::parsers::expr::*;
    use crate::unwrap_or_report;

    #[test]
    fn it_works() {
        let ast = super::test_parse("(Nehuen).chupar[Pito]()").unwrap();

        println!("From: (Nehuen).chupar[Pito]()");
        println!("AST: {:?}", ast);
    }

    #[test]
    fn test_asign() {
        let source = "a = 1 + 2";
        let result = super::test_parse(source);

        let expected = Expr::Assign(ExprAssign {
            left: Box::new(Expr::Path(PathExpr {
                segments: vec![Segment {
                    name: Ident::new("a"),
                    generics: None,
                }],
            })),
            kind: AssignKind::Equal,
            right: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::Value(ExprValue::Integer("1".to_string()))),
                op: Operator::Add,
                right: Box::new(Expr::Value(ExprValue::Integer("2".to_string()))),
            }),
        });

        assert_eq!(unwrap_or_report!(result, source), expected);
    }

    #[test]
    fn test_struct_init() {
        let source = "Foo { .a = 42, .b = true }";
        let result = super::test_parse(source);

        let expected = Expr::Init(ExprInit {
            path: PathExpr {
                segments: vec![Segment {
                    name: Ident::new("Foo"),
                    generics: None,
                }],
            },
            fields: vec![
                ExprInitField {
                    name: Ident::new("a"),
                    value: Box::new(Expr::Value(ExprValue::Integer("42".to_string()))),
                },
                ExprInitField {
                    name: Ident::new("b"),
                    value: Box::new(Expr::Value(ExprValue::Bool(true))),
                },
            ],
        });

        assert_eq!(unwrap_or_report!(result, source), expected);
    }

    #[test]
    fn test_access_call() {
        let ast = super::test_parse("obj().method(arg1, arg2)").unwrap();

        let expected = Expr::Call(ExprCall {
            expr: Box::new(Expr::Access(ExprAccess {
                expr: Box::new(Expr::Call(ExprCall {
                    expr: Box::new(Expr::Path(PathExpr {
                        segments: vec![Segment {
                            name: Ident {
                                name: "obj".to_string(),
                            },
                            generics: None,
                        }],
                    })),
                    args: vec![],
                })),
                segment: Segment {
                    name: Ident {
                        name: "method".to_string(),
                    },
                    generics: None,
                },
            })),
            args: vec![
                Expr::Path(PathExpr {
                    segments: vec![Segment {
                        name: Ident {
                            name: "arg1".to_string(),
                        },
                        generics: None,
                    }],
                }),
                Expr::Path(PathExpr {
                    segments: vec![Segment {
                        name: Ident {
                            name: "arg2".to_string(),
                        },
                        generics: None,
                    }],
                }),
            ],
        });

        assert_eq!(ast, expected);
    }

    #[test]
    fn test_addition() {
        let inputs = ["1 + 2", "3 - 4 + 5", "6 + 7 - 8 + 9", "a.b + c.d"];

        let expected = [
            Expr::BinaryOp {
                left: Box::new(Expr::Value(ExprValue::Integer("1".to_string()))),
                op: Operator::Add,
                right: Box::new(Expr::Value(ExprValue::Integer("2".to_string()))),
            },
            // (3 - 4) + 5
            Expr::BinaryOp {
                left: Box::new(Expr::BinaryOp {
                    left: Box::new(Expr::Value(ExprValue::Integer("3".to_string()))),
                    op: Operator::Sub,
                    right: Box::new(Expr::Value(ExprValue::Integer("4".to_string()))),
                }),
                op: Operator::Add,
                right: Box::new(Expr::Value(ExprValue::Integer("5".to_string()))),
            },
            // ((6 + 7) - 8) + 9
            Expr::BinaryOp {
                left: Box::new(Expr::BinaryOp {
                    left: Box::new(Expr::BinaryOp {
                        left: Box::new(Expr::Value(ExprValue::Integer("6".to_string()))),
                        op: Operator::Add,
                        right: Box::new(Expr::Value(ExprValue::Integer("7".to_string()))),
                    }),
                    op: Operator::Sub,
                    right: Box::new(Expr::Value(ExprValue::Integer("8".to_string()))),
                }),
                op: Operator::Add,
                right: Box::new(Expr::Value(ExprValue::Integer("9".to_string()))),
            },
            Expr::BinaryOp {
                left: Box::new(Expr::Path(PathExpr {
                    segments: vec![
                        Segment {
                            name: Ident {
                                name: "a".to_string(),
                            },
                            generics: None,
                        },
                        Segment {
                            name: Ident {
                                name: "b".to_string(),
                            },
                            generics: None,
                        },
                    ],
                })),
                op: Operator::Add,
                right: Box::new(Expr::Path(PathExpr {
                    segments: vec![
                        Segment {
                            name: Ident {
                                name: "c".to_string(),
                            },
                            generics: None,
                        },
                        Segment {
                            name: Ident {
                                name: "d".to_string(),
                            },
                            generics: None,
                        },
                    ],
                })),
            },
        ];

        for (i, input) in inputs.iter().enumerate() {
            let ast = super::test_parse(input).unwrap();
            assert_eq!(ast, expected[i]);
        }
    }

    #[test]
    fn test_term() {
        let inputs = ["123i32 * 13", "45.67 * 2 * \"Test\"", "523 / 3.14 * true"];

        let expected = [
            Expr::BinaryOp {
                left: Box::new(Expr::Value(ExprValue::Integer("123i32".to_string()))),
                op: Operator::Mul,
                right: Box::new(Expr::Value(ExprValue::Integer("13".to_string()))),
            },
            // (45.67 * 2) * "Test"
            Expr::BinaryOp {
                left: Box::new(Expr::BinaryOp {
                    left: Box::new(Expr::Value(ExprValue::Float("45.67".to_string()))),
                    op: Operator::Mul,
                    right: Box::new(Expr::Value(ExprValue::Integer("2".to_string()))),
                }),
                op: Operator::Mul,
                right: Box::new(Expr::Value(ExprValue::String("\"Test\"".to_string()))),
            },
            Expr::BinaryOp {
                left: Box::new(Expr::BinaryOp {
                    left: Box::new(Expr::Value(ExprValue::Integer("523".to_string()))),
                    op: Operator::Div,
                    right: Box::new(Expr::Value(ExprValue::Float("3.14".to_string()))),
                }),
                op: Operator::Mul,
                right: Box::new(Expr::Value(ExprValue::Bool(true))),
            },
        ];

        for (i, input) in inputs.iter().enumerate() {
            let ast = super::test_parse(input).unwrap();
            assert_eq!(ast, expected[i]);
        }
    }

    #[test]
    fn test_values() {
        let inputs = ["123", "45.67", "\"Hello, World!\"", "true", "false", "(1)"];

        let expected = [
            Expr::Value(ExprValue::Integer("123".to_string())),
            Expr::Value(ExprValue::Float("45.67".to_string())),
            Expr::Value(ExprValue::String("\"Hello, World!\"".to_string())),
            Expr::Value(ExprValue::Bool(true)),
            Expr::Value(ExprValue::Bool(false)),
            Expr::Value(ExprValue::Integer("1".to_string())),
        ];

        for (i, input) in inputs.iter().enumerate() {
            let ast = super::test_parse(input).unwrap();
            assert_eq!(ast, expected[i]);
        }
    }

    #[test]
    fn test_sub_left_associative() {
        // 10 - 3 - 2  should be  (10 - 3) - 2  = 5
        let ast = super::test_parse("10 - 3 - 2").unwrap();
        let expected = Expr::BinaryOp {
            left: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::Value(ExprValue::Integer("10".to_string()))),
                op: Operator::Sub,
                right: Box::new(Expr::Value(ExprValue::Integer("3".to_string()))),
            }),
            op: Operator::Sub,
            right: Box::new(Expr::Value(ExprValue::Integer("2".to_string()))),
        };
        assert_eq!(ast, expected);
    }

    #[test]
    fn test_div_left_associative() {
        // 12 / 3 / 2  should be  (12 / 3) / 2  = 2
        let ast = super::test_parse("12 / 3 / 2").unwrap();
        let expected = Expr::BinaryOp {
            left: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::Value(ExprValue::Integer("12".to_string()))),
                op: Operator::Div,
                right: Box::new(Expr::Value(ExprValue::Integer("3".to_string()))),
            }),
            op: Operator::Div,
            right: Box::new(Expr::Value(ExprValue::Integer("2".to_string()))),
        };
        assert_eq!(ast, expected);
    }

    #[test]
    fn test_mixed_add_sub_left_associative() {
        // 6 - 2 + 1  should be  (6 - 2) + 1  = 5
        let ast = super::test_parse("6 - 2 + 1").unwrap();
        let expected = Expr::BinaryOp {
            left: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::Value(ExprValue::Integer("6".to_string()))),
                op: Operator::Sub,
                right: Box::new(Expr::Value(ExprValue::Integer("2".to_string()))),
            }),
            op: Operator::Add,
            right: Box::new(Expr::Value(ExprValue::Integer("1".to_string()))),
        };
        assert_eq!(ast, expected);
    }
}
