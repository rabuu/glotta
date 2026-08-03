use std::fmt;

use crate::{Span, Spanned};
use logos::{Logos, SpannedIter};

#[derive(Debug, Clone, PartialEq)]
pub enum LexingError {
    InvalidToken(Option<Span>),
    InvalidIntegerLiteral(Span),
}

impl Default for LexingError {
    fn default() -> Self {
        Self::InvalidToken(None)
    }
}

#[derive(Debug, Logos)]
#[logos(skip r"[ \t\r\n\f]+")]
#[logos(error(LexingError, callback = |lex| LexingError::InvalidToken(Some(lex.span().into()))))]
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
        let literal: i64 = source
            .replace('_', "")
            .parse()
            .map_err(|_| LexingError::InvalidIntegerLiteral(span))?;
        Ok(Self { source, literal })
    }
}

pub struct Lexer<'src> {
    tokens: SpannedIter<'src, Token<'src>>,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            tokens: Token::lexer(source).spanned(),
        }
    }
}

impl<'src> Iterator for Lexer<'src> {
    type Item = Result<Spanned<Token<'src>>, LexingError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.tokens
            .next()
            .map(|(token, span)| Ok((token?, span.into())))
    }
}
