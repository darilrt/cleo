mod ast;

pub use ast::*;

pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl From<lexer::Span> for Span {
    fn from(span: lexer::Span) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}
