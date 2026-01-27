use std::borrow::Cow;

use logos::Logos;

use crate::{LexerError, Span, Token, TokenKind};

pub fn lex<'a>(src: &'a str) -> Result<Vec<Token<'a>>, LexerError> {
    let mut lexer = TokenKind::lexer(src);
    let mut tokens = Vec::new();

    let next_token = |lexer: &mut logos::Lexer<'a, TokenKind<'a>>| {
        let kind = lexer.next();
        let span = lexer.span();
        let lexeme = lexer.slice();
        match kind {
            Some(Ok(token_kind)) => Some(Ok(Token {
                kind: token_kind,
                lexeme: Cow::Borrowed(lexeme),
                span: Span {
                    start: span.start,
                    end: span.end,
                },
            })),
            Some(Err(err)) => Some(Err(err)),
            None => None,
        }
    };

    let mut last_emited = Token::unknown();
    let mut ahead_token = None;

    loop {
        let Some(Ok(token)) = ahead_token.take().or_else(|| next_token(&mut lexer)) else {
            break;
        };

        match token.kind {
            TokenKind::Comment => {
                continue;
            }
            TokenKind::NewLine => {
                let ahead = ahead_token.get_or_insert_with(|| {
                    next_token(&mut lexer).unwrap_or(Ok(Token {
                        kind: TokenKind::EOF,
                        lexeme: Cow::Borrowed(""),
                        span: Span {
                            start: src.len(),
                            end: src.len(),
                        },
                    }))
                });

                let ahead = ahead.as_ref().map_err(|_| LexerError::InvalidCharacter {
                    character: '\0',
                    position: token.span.end,
                })?;

                if matches!(
                    ahead.kind,
                    TokenKind::Plus
                        | TokenKind::Minus
                        | TokenKind::Asterisk
                        | TokenKind::Slash
                        | TokenKind::Percent
                        | TokenKind::Dot
                        | TokenKind::DoubleColon
                        | TokenKind::RightParen
                        | TokenKind::RightBrace
                        | TokenKind::RightBracket
                ) {
                    continue;
                }

                match last_emited.kind {
                    TokenKind::Ident(_)
                    | TokenKind::String(_)
                    | TokenKind::Integer(_)
                    | TokenKind::Float(_)
                    | TokenKind::Bool(_)
                    | TokenKind::Break
                    | TokenKind::Return
                    | TokenKind::Continue
                    | TokenKind::PlusPlus
                    | TokenKind::MinusMinus
                    | TokenKind::RightParen
                    | TokenKind::RightBracket
                    | TokenKind::RightBrace => {
                        let new_token = Token {
                            kind: TokenKind::Semicolon,
                            lexeme: Cow::Borrowed(";"),
                            span: token.span.clone(),
                        };
                        last_emited = new_token.clone();
                        tokens.push(new_token);
                        continue;
                    }
                    _ => {
                        continue;
                    }
                }
            }
            _ => {
                last_emited = token.clone();
                tokens.push(token);
            }
        }
    }

    Ok(tokens)
}

mod test {
    #[test]
    fn test_lex_simple() {
        let source = "let x =  Foo { .a = 10 }\nvar print(x)";
        let tokens = super::lex(source).unwrap();

        for tok in tokens {
            println!("{:?}", tok.kind);
        }
    }
}
