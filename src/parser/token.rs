use std::fmt;

use crate::span::Span;

#[derive(Debug, Clone, Copy)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Identifier,

    IntegerLiteral,

    Fun,
    Int,

    Equals,
    Colon,

    Comment,
    Whitespace,

    Invalid,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Identifier => write!(f, "identifier"),
            TokenKind::IntegerLiteral => write!(f, "integer literal"),
            TokenKind::Fun => write!(f, "`fun`"),
            TokenKind::Int => write!(f, "`Int`"),
            TokenKind::Equals => write!(f, "`=`"),
            TokenKind::Colon => write!(f, "`:`"),
            TokenKind::Comment => write!(f, "comment"),
            TokenKind::Whitespace => write!(f, "whitespace"),
            TokenKind::Invalid => write!(f, "invalid token"),
        }
    }
}
