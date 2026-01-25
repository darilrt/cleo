use chumsky::{
    Parser,
    prelude::{just, recursive},
    select,
};
use lexer::TokenKind;

use crate::{
    fn_parser,
    parser::{
        path::{Path, path_impl},
        ptype::ptype_impl,
    },
    path_parser,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Value(Value),
    BinaryOp {
        left: Box<Expr>,
        op: Operator,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(String),
    Float(String),
    Bool(bool),
    String(String),
    Path(Path),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

fn_parser!(expr -> Expr {
    recursive(|expr| {
        // factor := integer | float | string | bool | "(" expr ")" | path
        let factor = select! {
            TokenKind::Integer(v) => Expr::Value(Value::Integer(v.to_string())),
            TokenKind::Float(v) => Expr::Value(Value::Float(v.to_string())),
            TokenKind::String(v) => Expr::Value(Value::String(v.to_string())),
            TokenKind::Bool(v) => Expr::Value(Value::Bool(v == "true")),
        }.or(
            expr.clone().delimited_by(
                just(TokenKind::LeftParen),
                just(TokenKind::RightParen)
            )
        )
        .or(
        //     recursive(|path| {
        //         let ptype_parser = ptype_impl(path.clone());
        //         path_impl(ptype_parser)
            path_parser!().map(|p| {
                Expr::Value(Value::Path(p))
            })
        );

        // divison := factor ("/" factor)*
        let division = recursive(|division| {
            let op = select! {
                TokenKind::Slash => Operator::Div,
            };

            factor.clone()
                .foldl_with(op.then(division).repeated(), |lhs, (op, rhs), _e| {
                    Expr::BinaryOp { left: Box::new(lhs), op, right: Box::new(rhs) }
                })
        });

        // term := division ("*" division)*
        let term = recursive(|term| {
            let op = select! {
                TokenKind::Asterisk => Operator::Mul,
            };

            division.clone()
            .foldl_with(op.then(term).repeated(), |lhs, (op, rhs), _e| {
                Expr::BinaryOp { left: Box::new(lhs), op, right: Box::new(rhs) }
            })
        });

        // addition := term (("+" | "-") term)*
        let addition = recursive(|addition| {
            let op = select! {
                TokenKind::Plus => Operator::Add,
                TokenKind::Minus => Operator::Sub,
            };

            term.clone()
            .foldl_with(op.then(addition).repeated(), |lhs, (op, rhs), _e| {
                Expr::BinaryOp { left: Box::new(lhs), op, right: Box::new(rhs) }
            })
        });

        addition
    })
});

mod test {
    #[allow(unused_imports)]
    use crate::parser::expr::*;

    #[test]
    fn it_works() {
        let ast = super::test_parse("123 * 2").unwrap();

        println!("AST: {:?}", ast);
    }

    #[test]
    fn test_addition() {
        let inputs = ["1 + 2", "3 - 4 + 5", "6 + 7 - 8 + 9"];

        let expected = [
            Expr::BinaryOp {
                left: Box::new(Expr::Value(Value::Integer("1".to_string()))),
                op: Operator::Add,
                right: Box::new(Expr::Value(Value::Integer("2".to_string()))),
            },
            Expr::BinaryOp {
                left: Box::new(Expr::Value(Value::Integer(3.to_string()))),
                op: Operator::Sub,
                right: Box::new(Expr::BinaryOp {
                    left: Box::new(Expr::Value(Value::Integer("4".to_string()))),
                    op: Operator::Add,
                    right: Box::new(Expr::Value(Value::Integer("5".to_string()))),
                }),
            },
            Expr::BinaryOp {
                left: Box::new(Expr::Value(Value::Integer(6.to_string()))),
                op: Operator::Add,
                right: Box::new(Expr::BinaryOp {
                    left: Box::new(Expr::Value(Value::Integer("7".to_string()))),
                    op: Operator::Sub,
                    right: Box::new(Expr::BinaryOp {
                        left: Box::new(Expr::Value(Value::Integer("8".to_string()))),
                        op: Operator::Add,
                        right: Box::new(Expr::Value(Value::Integer("9".to_string()))),
                    }),
                }),
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
                left: Box::new(Expr::Value(Value::Integer("123i32".to_string()))),
                op: Operator::Mul,
                right: Box::new(Expr::Value(Value::Integer("13".to_string()))),
            },
            Expr::BinaryOp {
                left: Box::new(Expr::Value(Value::Float("45.67".to_string()))),
                op: Operator::Mul,
                right: Box::new(Expr::BinaryOp {
                    left: Box::new(Expr::Value(Value::Integer("2".to_string()))),
                    op: Operator::Mul,
                    right: Box::new(Expr::Value(Value::String("\"Test\"".to_string()))),
                }),
            },
            Expr::BinaryOp {
                left: Box::new(Expr::BinaryOp {
                    left: Box::new(Expr::Value(Value::Integer("523".to_string()))),
                    op: Operator::Div,
                    right: Box::new(Expr::Value(Value::Float("3.14".to_string()))),
                }),
                op: Operator::Mul,
                right: Box::new(Expr::Value(Value::Bool(true))),
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
            Expr::Value(Value::Integer("123".to_string())),
            Expr::Value(Value::Float("45.67".to_string())),
            Expr::Value(Value::String("\"Hello, World!\"".to_string())),
            Expr::Value(Value::Bool(true)),
            Expr::Value(Value::Bool(false)),
            Expr::Value(Value::Integer("1".to_string())),
        ];

        for (i, input) in inputs.iter().enumerate() {
            let ast = super::test_parse(input).unwrap();
            assert_eq!(ast, expected[i]);
        }
    }
}
