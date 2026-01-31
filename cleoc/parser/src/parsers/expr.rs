use chumsky::{
    IterParser, Parser, extra,
    input::ValueInput,
    prelude::{just, recursive},
    select,
    span::SimpleSpan,
};
use lexer::TokenKind;

use crate::{
    ast::{Ident, Type},
    errors::ParserError,
    parsers::{
        block::{Block, block_impl},
        ident::ident,
        path::{PathExpr, Segment, segment},
    },
    path_parser, type_parser,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Value(ExprValue),
    BinaryOp {
        left: Box<Expr>,
        op: Operator,
        right: Box<Expr>,
    },
    UnaryOp {
        op: Operator,
        expr: Box<Expr>,
    },
    Call(ExprCall),
    Access(ExprAccess),
    Path(PathExpr),
    If(ExprIf),
    Init(ExprInit),
    Cast(ExprCast),
    Assign(ExprAssign),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprAssign {
    pub left: Box<Expr>,
    pub kind: AssignKind,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignKind {
    Equal,    // =
    AddEqual, // +=
    SubEqual, // -=
    MulEqual, // *=
    DivEqual, // /=
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprCast {
    pub expr: Box<Expr>,
    pub to_type: Box<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprInit {
    pub path: PathExpr,
    pub fields: Vec<ExprInitField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprInitField {
    pub name: Ident,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprIf {
    pub condition: Box<Expr>,
    pub then_branch: Block,
    pub else_branch: Option<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprValue {
    Integer(String),
    Float(String),
    Bool(bool),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprAccess {
    pub expr: Box<Expr>,
    pub segment: Segment,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprCall {
    pub expr: Box<Expr>,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /

    Deref, // *
    Ref,   // &
    Neg,   // -
    Not,   // !

    Equal,        // ==
    NotEqual,     // !=
    Less,         // <
    Greater,      // >
    LessEqual,    // <=
    GreaterEqual, // >=

    And, // &&
    Or,  // ||
}

// expr = if_expr | addition;
pub fn expr<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Expr, extra::Err<ParserError<'tokens, 'src>>> + Clone
where
    I: ValueInput<'tokens, Token = TokenKind<'src>, Span = SimpleSpan>,
{
    recursive(|expr| {
        let block = block_impl(expr.clone());
        let path = path_parser!();
        let ptype = type_parser!();

        let struct_init_fields = just(TokenKind::Dot)
            .ignore_then(ident())
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

        // primary := integer | float | string | bool | "(" expr ")" | strcut_init | path
        let primary = select! {
            TokenKind::Integer(v) => Expr::Value(ExprValue::Integer(v.to_string())),
            TokenKind::Float(v) => Expr::Value(ExprValue::Float(v.to_string())),
            TokenKind::String(v) => Expr::Value(ExprValue::String(v.to_string())),
            TokenKind::Bool(v) => Expr::Value(ExprValue::Bool(v == "true")),
        }
        .or(expr
            .clone()
            .delimited_by(just(TokenKind::LeftParen), just(TokenKind::RightParen)))
        .or(struct_init.map(Expr::Init))
        .or(path.clone().map(Expr::Path));

        // call := "(", [ expr, { ",", expr }, [ "," ] ], ")"
        let call = expr
            .clone()
            .separated_by(just(TokenKind::Comma))
            .allow_trailing()
            .collect::<Vec<Expr>>()
            .delimited_by(just(TokenKind::LeftParen), just(TokenKind::RightParen));

        // field_access := ".", segment
        let field_access = just(TokenKind::Dot)
            .ignore_then(segment(ptype.clone()))
            .map(|seg| seg);

        enum Accessor {
            Call(Vec<Expr>),
            FieldAccess(Segment),
        }

        // accessor := call | field_access
        let accessor = field_access
            .map(Accessor::FieldAccess)
            .or(call.map(Accessor::Call));

        // factor := primary, { accessor }
        let factor = primary
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
            });

        // cast = factor, [ "as", ptype ];
        let cast = factor
            .clone()
            .then(just(TokenKind::As).ignore_then(ptype.clone()).or_not())
            .map(|(expr, to_type)| {
                if let Some(_to_type) = to_type {
                    Expr::Cast(ExprCast {
                        expr: Box::new(expr),
                        to_type: Box::new(_to_type),
                    })
                } else {
                    expr
                }
            });

        // unary = ( "*" | "&" | "!" | "-" ), cast | cast
        let unary = select! {
            TokenKind::Asterisk => Operator::Deref,
            TokenKind::And => Operator::Ref,
            TokenKind::Minus => Operator::Neg,
            TokenKind::Not => Operator::Not,
        }
        .then(cast.clone())
        .map(|(op, expr)| Expr::UnaryOp {
            op,
            expr: Box::new(expr),
        })
        .or(cast.clone());

        // divison := unary, { "/", unary }
        let division = recursive(|division| {
            let op = select! {
                TokenKind::Slash => Operator::Div,
            };

            unary
                .clone()
                .foldl_with(op.then(division).repeated(), |lhs, (op, rhs), _e| {
                    Expr::BinaryOp {
                        left: Box::new(lhs),
                        op,
                        right: Box::new(rhs),
                    }
                })
        });

        // multiplication := division, { "*", division }
        let multiplication = recursive(|multiplication| {
            let op = select! {
                TokenKind::Asterisk => Operator::Mul,
            };

            division
                .clone()
                .foldl_with(op.then(multiplication).repeated(), |lhs, (op, rhs), _e| {
                    Expr::BinaryOp {
                        left: Box::new(lhs),
                        op,
                        right: Box::new(rhs),
                    }
                })
        });

        // addition := divison, { ("+" | "-"), divison }
        let addition = recursive(|addition| {
            let op = select! {
                TokenKind::Plus => Operator::Add,
                TokenKind::Minus => Operator::Sub,
            };

            multiplication
                .clone()
                .foldl_with(op.then(addition).repeated(), |lhs, (op, rhs), _e| {
                    Expr::BinaryOp {
                        left: Box::new(lhs),
                        op,
                        right: Box::new(rhs),
                    }
                })
        });

        let comp_op = select! {
            TokenKind::EqualEqual => Operator::Equal,
            TokenKind::NotEqual => Operator::NotEqual,
            TokenKind::LessThanEqual => Operator::LessEqual,
            TokenKind::GreaterThanEqual => Operator::GreaterEqual,
            TokenKind::LessThan => Operator::Less,
            TokenKind::GreaterThan => Operator::Greater,
        };

        // comparation = addition, [ ( "==" \| "!=" | "<=" | ">=" | "<" | ">" ), addition ];
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

        // logic_and = comparation, { "&&", comparation };
        let logic_and = recursive(|logic_and| {
            let op = select! {
                TokenKind::AndAnd => Operator::And,
            };

            comparation
                .clone()
                .foldl_with(op.then(logic_and).repeated(), |lhs, (op, rhs), _e| {
                    Expr::BinaryOp {
                        left: Box::new(lhs),
                        op,
                        right: Box::new(rhs),
                    }
                })
        });

        // logic_or = logic_and, { "||", logic_and };
        let logic_or = recursive(|logic_or| {
            let op = select! {
                TokenKind::OrOr => Operator::Or,
            };

            logic_and
                .clone()
                .foldl_with(op.then(logic_or).repeated(), |lhs, (op, rhs), _e| {
                    Expr::BinaryOp {
                        left: Box::new(lhs),
                        op,
                        right: Box::new(rhs),
                    }
                })
        });

        // assign_kind = "= | "+=" | "-=" | "*=" | "/="
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

        if_expr.or(assign).or(logic_or)
    })
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
    use super::*;
    use crate::parsers::expr::*;
    use crate::parsers::{ident::Ident, path::Segment};
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
    fn test_cast() {
        let source = "value as i32";
        let result = super::test_parse(source);

        let expected = Expr::Cast(ExprCast {
            expr: Box::new(Expr::Path(PathExpr {
                segments: vec![Segment {
                    name: Ident::new("value"),
                    generics: None,
                }],
            })),
            to_type: Box::new(Type::Path(PathExpr {
                segments: vec![Segment {
                    name: Ident::new("i32"),
                    generics: None,
                }],
            })),
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
            Expr::BinaryOp {
                left: Box::new(Expr::Value(ExprValue::Integer(3.to_string()))),
                op: Operator::Sub,
                right: Box::new(Expr::BinaryOp {
                    left: Box::new(Expr::Value(ExprValue::Integer("4".to_string()))),
                    op: Operator::Add,
                    right: Box::new(Expr::Value(ExprValue::Integer("5".to_string()))),
                }),
            },
            Expr::BinaryOp {
                left: Box::new(Expr::Value(ExprValue::Integer(6.to_string()))),
                op: Operator::Add,
                right: Box::new(Expr::BinaryOp {
                    left: Box::new(Expr::Value(ExprValue::Integer("7".to_string()))),
                    op: Operator::Sub,
                    right: Box::new(Expr::BinaryOp {
                        left: Box::new(Expr::Value(ExprValue::Integer("8".to_string()))),
                        op: Operator::Add,
                        right: Box::new(Expr::Value(ExprValue::Integer("9".to_string()))),
                    }),
                }),
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
            Expr::BinaryOp {
                left: Box::new(Expr::Value(ExprValue::Float("45.67".to_string()))),
                op: Operator::Mul,
                right: Box::new(Expr::BinaryOp {
                    left: Box::new(Expr::Value(ExprValue::Integer("2".to_string()))),
                    op: Operator::Mul,
                    right: Box::new(Expr::Value(ExprValue::String("\"Test\"".to_string()))),
                }),
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
}
