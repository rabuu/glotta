use std::{collections::VecDeque, fmt};

use logos::{Logos, SpannedIter};
use miette::Diagnostic;
use thiserror::Error;

use crate::span::{Span, Spanned};

#[derive(Debug, Clone, PartialEq, Error, Diagnostic, Default)]
pub enum LexingError {
    #[default]
    #[error("Lexing failed.")]
    UnknownError,

    #[error("Encountered invalid token: `{src}`")]
    InvalidToken {
        src: String,
        #[label]
        span: Span,
    },

    #[error("Failed to parse integer literal: `{src}`")]
    InvalidIntegerLiteral {
        src: String,
        #[label]
        span: Span,
    },
}

#[derive(Debug, Logos)]
#[logos(skip r"[ \t\r\n\f]+")]
#[logos(error(LexingError, callback = |lex| LexingError::InvalidToken { src: lex.slice().to_string(), span: lex.span().into() }))]
pub enum Token<'src> {
    // This uses the following unicode categories:
    // Ll = lowercase letter
    // Lu = uppercase letter
    // No = other numbers
    #[regex(r"[\p{Ll}\p{Lu}_][\p{Ll}\p{Lu}0-9\p{No}_]*")]
    Identifier(&'src str),

    #[regex(r"[+-]?[0-9][0-9_]*", |lex| IntegerLiteral::parse(lex.slice(), lex.span().into()))]
    IntegerLiteral(IntegerLiteral<'src>),

    //
    // keywords
    //
    #[token("fun")]
    Fun,
    #[token("Int")]
    Int,

    //
    // symbols
    //
    #[token("=")]
    Equals,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Identifier,
    IntegerLiteral,
    Fun,
    Int,
    Equals,
}

impl Token<'_> {
    pub fn kind(&self) -> TokenKind {
        match self {
            Token::Identifier(_) => TokenKind::Identifier,
            Token::IntegerLiteral(_) => TokenKind::IntegerLiteral,
            Token::Fun => TokenKind::Fun,
            Token::Int => TokenKind::Int,
            Token::Equals => TokenKind::Equals,
        }
    }
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Identifier(ident) => write!(f, "{ident}"),
            Token::IntegerLiteral(lit) => write!(f, "{}", lit.source),
            Token::Fun => write!(f, "fun"),
            Token::Int => write!(f, "int"),
            Token::Equals => write!(f, "="),
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Identifier => write!(f, "identifier"),
            TokenKind::IntegerLiteral => write!(f, "integer literal"),
            TokenKind::Fun => write!(f, "fun"),
            TokenKind::Int => write!(f, "int"),
            TokenKind::Equals => write!(f, "="),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IntegerLiteral<'src> {
    pub source: &'src str,
    pub literal: i64,
}

impl<'src> IntegerLiteral<'src> {
    /// Parse an integer from a string slice with decimal digits.
    ///
    /// See Rust's [`std::str::FromStr`] on [`i64`].
    /// Additionally, underscores are allowed.
    fn parse(source: &'src str, span: Span) -> Result<Self, LexingError> {
        let literal: i64 =
            source
                .replace('_', "")
                .parse()
                .map_err(|_| LexingError::InvalidIntegerLiteral {
                    src: source.to_string(),
                    span,
                })?;
        Ok(Self { source, literal })
    }
}

type LexerItem<'src> = Result<Spanned<Token<'src>>, LexingError>;

pub struct Lexer<'src> {
    tokens: SpannedIter<'src, Token<'src>>,
    lookahead: VecDeque<LexerItem<'src>>,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            tokens: Token::lexer(source).spanned(),
            lookahead: VecDeque::new(),
        }
    }

    pub fn peek_n(&mut self, n: usize) -> Option<&LexerItem<'src>> {
        while self.lookahead.len() <= n {
            let next = self.next()?;
            self.lookahead.push_back(next);
        }
        self.lookahead.get(n)
    }

    pub fn peek(&mut self) -> Option<&LexerItem<'src>> {
        self.peek_n(0)
    }
}

impl<'src> Iterator for Lexer<'src> {
    type Item = LexerItem<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lookahead.pop_front().or_else(|| {
            self.tokens
                .next()
                .map(|(token, span)| Ok((token?, span.into())))
        })
    }
}
