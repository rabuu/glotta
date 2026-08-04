use std::collections::VecDeque;
use std::str::Chars;

use crate::parser::token::{Token, TokenKind};
use crate::span::Span;

pub struct Lexer<'src> {
    source: &'src str,
    chars: Chars<'src>,
    lookahead: VecDeque<char>,
    pos: usize,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            source,
            chars: source.chars(),
            lookahead: VecDeque::new(),
            pos: 0,
        }
    }

    pub fn source(&self) -> &'src str {
        self.source
    }

    pub fn next_token(&mut self) -> Option<Token> {
        let start = self.pos;
        let first_char = self.bump()?;

        let kind = match first_char {
            '/' => match self.peek() {
                Some('/') => {
                    self.bump();
                    self.eat_while(|c| c == '\n');
                    TokenKind::Comment
                }
                _ => TokenKind::Invalid,
            },
            c if c.is_whitespace() => {
                self.eat_while(char::is_whitespace);
                TokenKind::Whitespace
            }
            c if is_identifier_start(c) => {
                self.eat_while(is_identifier);
                match self.slice_from(start) {
                    "fun" => TokenKind::Fun,
                    "Int" => TokenKind::Int,
                    _ => TokenKind::Identifier,
                }
            }
            c if c.is_digit(10) || c == '-' => {
                self.eat_while(is_integer_literal);
                TokenKind::IntegerLiteral
            }
            '=' => TokenKind::Equals,
            ':' => TokenKind::Colon,
            _ => TokenKind::Invalid,
        };

        Some(Token {
            kind,
            span: self.span_from(start),
        })
    }

    fn peek_n(&mut self, n: usize) -> Option<char> {
        while self.lookahead.len() <= n {
            let c = self.chars.next()?;
            self.lookahead.push_back(c);
        }
        self.lookahead.get(n).copied()
    }

    fn peek(&mut self) -> Option<char> {
        self.peek_n(0)
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.lookahead.pop_front().or_else(|| self.chars.next())?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn span_from(&self, start: usize) -> Span {
        (start..self.pos).into()
    }

    fn slice_from(&self, start: usize) -> &'src str {
        &self.source[start..self.pos]
    }

    fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    fn eat_while(&mut self, pred: impl Fn(char) -> bool) {
        while let Some(c) = self.peek()
            && pred(c)
            && !self.is_eof()
        {
            self.bump();
        }
    }
}

fn is_identifier_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_identifier(c: char) -> bool {
    c.is_alphabetic() || c.is_numeric() || c == '_'
}

fn is_integer_literal(c: char) -> bool {
    c.is_ascii_digit() || c == '_'
}
