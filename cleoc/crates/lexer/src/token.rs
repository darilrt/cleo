use std::{borrow::Cow, fmt::Display};

use logos::Logos;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub lexeme: Cow<'a, str>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedToken<'a> {
    pub kind: TokenKind<'a>,
    pub lexeme: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Logos)]
#[logos(skip r"[ \t\r]+")]
pub enum TokenKind<'a> {
    /// Unknown or invalid token.
    Unknown,

    /// Newline character.
    #[token("\n")]
    NewLine,

    /// Single-line comment starting with `//`.
    #[regex(r"//[^\n]*", allow_greedy = true)]
    Comment,

    /// Identifier or variable name.
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident(&'a str),

    // Keywords
    /// Keyword `fn` to declare a function.
    #[token("fn")]
    Fn,
    /// Keyword `var` to declare a mutable variable.
    #[token("var")]
    Var,
    /// Keyword `if` for conditional branching.
    #[token("if")]
    If,
    /// Keyword `else` for conditional alternative.
    #[token("else")]
    Else,
    /// Keyword `while` for loop condition.
    #[token("while")]
    While,
    /// Keyword `for` for iteration.
    #[token("for")]
    For,
    /// Keyword `loop` for infinite loops.
    #[token("loop")]
    Loop,
    /// Keyword `return` to exit a function.
    #[token("return")]
    Return,
    /// Keyword `struct` to define a structure type.
    #[token("struct")]
    Struct,
    /// Keyword `enum` to define an enumeration type.
    #[token("enum")]
    Enum,
    /// Keyword `const` to declare a constant.
    #[token("const")]
    Const,
    /// Keyword `break` to exit a loop.
    #[token("break")]
    Break,
    /// Keyword `continue` to skip to next loop iteration.
    #[token("continue")]
    Continue,
    /// Keyword `type` for type aliases.
    #[token("type")]
    Type,
    /// Keyword `null` for null value.
    #[token("null")]
    Null,
    /// Keyword `trait` to define a trait interface.
    #[token("trait")]
    Trait,
    /// Keyword `match` for pattern matching.
    #[token("match")]
    Match,
    /// Keyword `import` to include external modules.
    #[token("import")]
    Import,
    /// Keyword `defer` to schedule code execution at scope exit.
    #[token("defer")]
    Defer,
    /// Keyword `pub` to mark items as public.
    #[token("pub")]
    Pub,
    /// Keyword `self` — the current instance. Only valid as the first parameter of a function.
    #[token("self")]
    SelfKw,

    // Literals
    /// Integer literal with optional suffix (decimal, hex, binary, octal).
    #[regex(r"((0[xX][0-9a-fA-F]+|0b[01]+|0[0-7]+|[0-9]+)(u8|u16|u32|u64|i8|i16|i32|i64)?)")]
    Integer(&'a str),
    /// Floating-point literal with optional exponent.
    #[regex(r"([0-9]+\.[0-9]*|[0-9]*\.[0-9]+)([eE][+-]?[0-9]+)?")]
    Float(&'a str),
    /// String literal enclosed in double quotes.
    #[regex(r#""([^"\\]|\\.)*""#)]
    String(&'a str),
    /// Boolean literal `true` or `false`.
    #[regex(r"(true|false)")]
    Bool(&'a str),

    // Operators
    /// Arithmetic addition operator `+`.
    #[token("+")]
    Plus,
    /// Arithmetic subtraction operator `-`.
    #[token("-")]
    Minus,
    /// Arithmetic multiplication operator `*`.
    #[token("*")]
    Asterisk,
    /// Arithmetic division operator `/`.
    #[token("/")]
    Slash,
    /// Arithmetic modulo operator `%`.
    #[token("%")]
    Percent,
    /// Increment operator `++`.
    #[token("++")]
    PlusPlus,
    /// Decrement operator `--`.
    #[token("--")]
    MinusMinus,
    /// Equality comparison operator `==`.
    #[token("==")]
    EqualEqual,
    /// Inequality comparison operator `!=`.
    #[token("!=")]
    NotEqual,
    /// Less-than comparison operator `<`.
    #[token("<")]
    LessThan,
    /// Less-than-or-equal comparison operator `<=`.
    #[token("<=")]
    LessThanEqual,
    /// Greater-than comparison operator `>`.
    #[token(">")]
    GreaterThan,
    /// Greater-than-or-equal comparison operator `>=`.
    #[token(">=")]
    GreaterThanEqual,
    /// Assignment operator `=`.
    #[token("=")]
    Equal,
    /// Addition assignment operator `+=`.
    #[token("+=")]
    PlusEqual,
    /// Subtraction assignment operator `-=`.
    #[token("-=")]
    MinusEqual,
    /// Multiplication assignment operator `*=`.
    #[token("*=")]
    AsteriskEqual,
    /// Division assignment operator `/=`.
    #[token("/=")]
    SlashEqual,
    /// Modulo assignment operator `%=`.
    #[token("%=")]
    PercentEqual,
    /// Logical AND operator `&&`.
    #[token("&&")]
    AndAnd,
    /// Logical OR operator `||`.
    #[token("||")]
    OrOr,
    /// Logical NOT operator `!`.
    #[token("!")]
    Not,
    /// Bitwise AND operator `&`.
    #[token("&")]
    And,
    /// Bitwise OR operator `|`.
    #[token("|")]
    Or,
    /// Bitwise XOR operator `^`.
    #[token("^")]
    Caret,
    /// Bitwise left shift operator `<<`.
    #[token("<<")]
    ShiftLeft,
    /// Bitwise right shift operator `>>`.
    #[token(">>")]
    ShiftRight,
    /// Bitwise NOT operator `~`.
    #[token("~")]
    Tilde,

    // Punctuation
    /// Left parenthesis `(` for grouping.
    #[token("(")]
    LeftParen,
    /// Right parenthesis `)` for grouping.
    #[token(")")]
    RightParen,
    /// Left brace `{` for code blocks.
    #[token("{")]
    LeftBrace,
    /// Right brace `}` for code blocks.
    #[token("}")]
    RightBrace,
    /// Left bracket `[` for array indexing.
    #[token("[")]
    LeftBracket,
    /// Right bracket `]` for array indexing.
    #[token("]")]
    RightBracket,
    /// Comma `,` for separating elements.
    #[token(",")]
    Comma,
    /// Semicolon `;` for statement termination.
    #[token(";")]
    Semicolon,
    /// Colon `:` for type annotations.
    #[token(":")]
    Colon,
    /// Dot `.` for member access.
    #[token(".")]
    Dot,
    /// Arrow `->` for function return type.
    #[token("->")]
    Arrow,
    /// Fat arrow `=>` for match arms.
    #[token("=>")]
    FatArrow,

    // Symbols
    /// Hash symbol `#`.
    #[token("#")]
    Hash,
    /// At symbol `@`.
    #[token("@")]
    At,
    /// Dollar symbol `$`.
    #[token("$")]
    Dollar,
    /// Question mark `?`.
    #[token("?")]
    Question,
    /// Ellipsis `...` for variadic arguments or ranges.
    #[token("...")]
    Ellipsis,

    // End of file
    /// End of file marker.
    #[token("\0")]
    EOF,
}

impl<'a> Token<'a> {
    pub fn unknown() -> Self {
        Self {
            kind: TokenKind::Unknown,
            lexeme: Cow::Borrowed(""),
            span: Span { start: 0, end: 0 },
        }
    }

    pub fn is_kind(&self, kind: &TokenKind) -> bool {
        &self.kind == kind
    }

    pub fn is_literal(&self) -> bool {
        matches!(
            self.kind,
            TokenKind::Integer(_) | TokenKind::Float(_) | TokenKind::String(_) | TokenKind::Bool(_)
        )
    }

    /// Convert to an owned token.
    pub fn into_owned(&self) -> OwnedToken<'a> {
        OwnedToken {
            kind: self.kind.clone(),
            lexeme: self.lexeme.to_string(),
            span: self.span.clone(),
        }
    }
}

impl<'a> Display for TokenKind<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Ident(name) => write!(f, "Identifier({})", name),
            TokenKind::Integer(value) => write!(f, "Integer({})", value),
            TokenKind::Float(value) => write!(f, "Float({})", value),
            TokenKind::String(value) => write!(f, "String({})", value),
            TokenKind::Bool(value) => write!(f, "Bool({})", value),
            t => write!(f, "{:?}", t),
        }
    }
}

#[allow(dead_code, unused_imports)]
mod test {
    use logos::Logos;

    use crate::{Token, TokenKind};

    #[test]
    fn test_lexer() {
        let source = "
        // This is a comment
        fn print(self: *Name) for Printable {
            println(\"Name: %s %s\", self->first, self->last)
        }

        fn func1(self: *Name) for TwoFunctions {
            println(\"Function 1 called for %s %s\", self->first, self->last)
        }
        ";

        let mut lexer = TokenKind::lexer(source);

        while let Some(Ok(token)) = lexer.next() {
            println!("{:?}: {}", token, &source[lexer.span()]);
        }
    }
}
