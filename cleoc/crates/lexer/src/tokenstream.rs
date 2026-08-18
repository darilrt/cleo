use std::rc::Rc;

use crate::{LexerError, OwnedToken, Token, TokenKind, lexer::lex};

#[derive(Debug)]
pub enum StreamError {
    UnexpectedToken {
        expected: Option<TokenKind>,
        found: OwnedToken,
    },
    UnexpectedEOF {
        expected: Option<TokenKind>,
    },
}

#[derive(Debug, Clone)]
pub struct TokenStream<'a> {
    tokens: Rc<Vec<Token<'a>>>,
    position: usize,
}

impl<'a> TokenStream<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self {
            tokens: Rc::new(tokens),
            position: 0,
        }
    }

    pub fn from_source(source: &'a str) -> Result<Self, LexerError> {
        Ok(lex(source)?)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    #[inline]
    pub fn is_eof(&self) -> bool {
        self.position >= self.tokens.len()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<Token<'a>> {
        self.tokens.get(index).cloned()
    }

    /// Return the next token in the stream without consuming it.
    #[inline]
    pub fn peek(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.position)
    }

    /// Return the token at the given offset from the current position without consuming it.
    #[inline]
    pub fn peek_at(&self, offset: usize) -> Option<Token<'a>> {
        self.get(self.position + offset)
    }

    /// Consume the next token in the stream.
    pub fn consume(&mut self) {
        self.position += 1;
    }

    /// Expect the next token to be of the given kind, consuming it if so.
    pub fn expect(&mut self, expected: TokenKind) -> Result<Token<'a>, StreamError> {
        if let Some(token) = self.peek() {
            if token.is_kind(&expected) {
                let token = token.clone();
                self.consume();
                Ok(token)
            } else {
                Err(StreamError::UnexpectedToken {
                    expected: Some(expected),
                    found: token.into_owned(),
                })
            }
        } else {
            Err(StreamError::UnexpectedEOF {
                expected: Some(expected),
            })
        }
    }

    /// Check if the next token is of the given kind, without consuming it.
    pub fn check(&self, expected: TokenKind) -> bool {
        if let Some(token) = self.peek() {
            token.is_kind(&expected)
        } else {
            false
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Token<'a>> {
        self.tokens.iter()
    }
}
