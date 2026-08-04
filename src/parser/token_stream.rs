use std::collections::VecDeque;

use crate::parser::{
    lexer::Lexer,
    token::{Token, TokenKind},
};

pub struct TokenStream<'src> {
    lexer: Lexer<'src>,
    lookahead: VecDeque<Token>,
    ignore_whitespace: bool,
}

impl<'src> TokenStream<'src> {
    pub fn new(lexer: Lexer<'src>, ignore_whitespace: bool) -> Self {
        Self {
            lexer,
            lookahead: VecDeque::new(),
            ignore_whitespace,
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

    fn next_token(&mut self) -> Option<Token> {
        let mut token = self.lexer.next_token()?;
        if self.ignore_whitespace {
            while token.kind == TokenKind::Whitespace {
                token = self.lexer.next_token()?;
            }
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
