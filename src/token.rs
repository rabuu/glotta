use std::fmt;

use crate::span::{Span, Spanned};

use glotta_macros::Spanned;

#[derive(Debug, Clone, Copy, Spanned)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Identifier,

    IntegerLiteral,

    Let,
    Int,

    ParenL,
    ParenR,
    CurlyL,
    CurlyR,

    Equals,
    Colon,
    Semicolon,
    Comma,
    Arrow,
    Hash,

    Comment,
    Whitespace,

    Invalid,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Identifier => write!(f, "identifier"),
            TokenKind::IntegerLiteral => write!(f, "integer literal"),
            TokenKind::Let => write!(f, "`let`"),
            TokenKind::Int => write!(f, "`Int`"),
            TokenKind::ParenL => write!(f, "`(`"),
            TokenKind::ParenR => write!(f, "`)`"),
            TokenKind::CurlyL => write!(f, "`{{`"),
            TokenKind::CurlyR => write!(f, "`}}`"),
            TokenKind::Equals => write!(f, "`=`"),
            TokenKind::Colon => write!(f, "`:`"),
            TokenKind::Semicolon => write!(f, "`;`"),
            TokenKind::Comma => write!(f, "`,`"),
            TokenKind::Arrow => write!(f, "`->`"),
            TokenKind::Hash => write!(f, "`#`"),
            TokenKind::Comment => write!(f, "comment"),
            TokenKind::Whitespace => write!(f, "whitespace"),
            TokenKind::Invalid => write!(f, "invalid token"),
        }
    }
}
