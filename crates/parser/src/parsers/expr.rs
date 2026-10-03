use ast::{
    AssignKind, Expr, ExprAccess, ExprAssign, ExprCall, ExprIf, ExprInit, ExprInitField,
    IntegerSuffix, Literal, Operator, Segment,
};
use chumsky::{
    IterParser, Parser,
    error::Rich,
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
            TokenKind::Integer(v) => v,
        }
        .try_map(|v, span| {
            parse_int_literal(v)
                .map(Expr::Value)
                .map_err(|e| Rich::custom(span, e))
        })
        .or(select! {
            TokenKind::Float(v) => Expr::Value(Literal::Float(v.to_string())),
            TokenKind::String(v) => Expr::Value(Literal::String(v.to_string())),
            TokenKind::Bool(v) => Expr::Value(Literal::Bool(v == "true")),
        })
        .or(expr
            .clone()
            .delimited_by(just(TokenKind::LeftParen), just(TokenKind::RightParen)))
        .or(struct_init.map(Expr::Init))
        .or(path.clone().map(Expr::Path))
        .or(if_expr)
        .or(loop_expr)
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
                            callee: Box::new(acc),
                            args,
                        }),
                        Accessor::FieldAccess(seg) => Expr::Access(ExprAccess {
                            inner: Box::new(acc),
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
                TokenKind::Percent => Operator::Mod,
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

        assign.or(logic_or)
    })
    .boxed()
}

fn parse_int_literal(raw: &str) -> Result<Literal, String> {
    let (num, suffix) = if let Some(s) = raw.strip_suffix("u8") {
        (s, Some(IntegerSuffix::U8))
    } else if let Some(s) = raw.strip_suffix("u16") {
        (s, Some(IntegerSuffix::U16))
    } else if let Some(s) = raw.strip_suffix("u32") {
        (s, Some(IntegerSuffix::U32))
    } else if let Some(s) = raw.strip_suffix("u64") {
        (s, Some(IntegerSuffix::U64))
    } else if let Some(s) = raw.strip_suffix("i8") {
        (s, Some(IntegerSuffix::I8))
    } else if let Some(s) = raw.strip_suffix("i16") {
        (s, Some(IntegerSuffix::I16))
    } else if let Some(s) = raw.strip_suffix("i32") {
        (s, Some(IntegerSuffix::I32))
    } else if let Some(s) = raw.strip_suffix("i64") {
        (s, Some(IntegerSuffix::I64))
    } else {
        (raw, None)
    };

    let value = if let Some(hex) = num.strip_prefix("0x").or_else(|| num.strip_prefix("0X")) {
        u128::from_str_radix(hex, 16)
    } else if let Some(bin) = num.strip_prefix("0b").or_else(|| num.strip_prefix("0B")) {
        u128::from_str_radix(bin, 2)
    } else if num.starts_with('0') && num.len() > 1 && num.chars().all(|c| matches!(c, '0'..='7')) {
        u128::from_str_radix(num, 8) // octal estilo C; opcional
    } else {
        num.parse::<u128>()
    }
    .map_err(|e| e.to_string())?;

    Ok(Literal::Integer {
        value,
        suffix: suffix,
    })
}

// fn parse_float_literal(raw: &str) -> Result<(f64, Option<FloatSuffix>), String> {
//     let (num, suffix) = if let Some(s) = raw.strip_suffix("f32") {
//         (s, Some(FloatSuffix::F32))
//     } else if let Some(s) = raw.strip_suffix("f64") {
//         (s, Some(FloatSuffix::F64))
//     } else {
//         (raw, None)
//     };
//     let value = num.parse::<f64>().map_err(|e| e.to_string())?;
//     Ok((value, suffix))
// }

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
