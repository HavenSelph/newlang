use std::fmt::{Display, Formatter};
use crate::span::Span;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TokenKind {
    // Misc
    Period,
    Slash,
    Equals,
    SemiColon,

    // Keywords
    Let,

    // Literals
    Identifier,
    StringLiteral,
    IntegerLiteralBin,
    IntegerLiteralHex,
    IntegerLiteralOct,
    IntegerLiteralDec,
    FloatLiteral,

    // Errors and misc tokens
    EOF
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub text: String,
    pub newline_before: bool
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, text: String) -> Self {
        Token {
            kind,
            span,
            text,
            newline_before: false
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Token{{{:?}, {}, {:?}", self.kind, self.span, self.text)?;
        if self.newline_before {
            write!(f, ", nl={}", self.newline_before)?;
        }
        write!(f, "}}")
    }
}