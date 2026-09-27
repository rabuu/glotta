use std::collections::VecDeque;
use std::fmt;

use crate::lexing::Lexer;
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
    Set,
    If,
    Then,
    Else,
    Loop,
    Break,
    Continue,

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
            TokenKind::Set => write!(f, "`set`"),
            TokenKind::If => write!(f, "`if`"),
            TokenKind::Then => write!(f, "`then`"),
            TokenKind::Else => write!(f, "`else`"),
            TokenKind::Loop => write!(f, "`loop`"),
            TokenKind::Break => write!(f, "`break`"),
            TokenKind::Continue => write!(f, "`continue`"),
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

pub struct TokenStream<'src> {
    lexer: Lexer<'src>,
    lookahead: VecDeque<Token>,
    ignore: Vec<TokenKind>,
}

impl<'src> TokenStream<'src> {
    pub fn new(lexer: Lexer<'src>, ignore: Vec<TokenKind>) -> Self {
        Self {
            lexer,
            lookahead: VecDeque::new(),
            ignore,
        }
    }

    pub fn peek_n(&mut self, n: usize) -> Option<Token> {
        while self.lookahead.len() <= n {
            let token = self.next_token()?;
            self.lookahead.push_back(token);
        }
        self.lookahead.get(n).copied()
    }

    pub fn peek(&mut self) -> Option<Token> {
        self.peek_n(0)
    }

    pub fn peek_kind_n(&mut self, n: usize) -> Option<TokenKind> {
        self.peek_n(n).map(|token| token.kind)
    }

    pub fn peek_kind(&mut self) -> Option<TokenKind> {
        self.peek_kind_n(0)
    }

    fn next_token(&mut self) -> Option<Token> {
        let mut token = self.lexer.next_token()?;
        while self.ignore.contains(&token.kind) {
            token = self.lexer.next_token()?;
        }
        Some(token)
    }
}

impl<'src> Iterator for TokenStream<'src> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.lookahead.pop_front().or_else(|| self.next_token())
    }
}
