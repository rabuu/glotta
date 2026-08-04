mod lexer;
mod token;
mod token_stream;

use miette::Diagnostic;
use thiserror::Error;

use crate::ast;
use crate::span::Span;

pub use lexer::Lexer;
pub use token::{Token, TokenKind};
pub use token_stream::TokenStream;

type Result<T> = std::result::Result<T, ParsingError>;

#[derive(Debug, Error, Diagnostic)]
pub enum ParsingError {
    #[error("Invalid token")]
    InvalidToken {
        #[label]
        span: Span,
    },

    #[error("Expected {expected} but got {got}.")]
    UnexpectedToken {
        expected: String,
        got: TokenKind,
        #[label]
        span: Span,
    },

    #[error("Expected {expected} but got EOF.")]
    UnexpectedEof { expected: String },

    #[error("Expected EOF but got {token}.")]
    ExtraToken {
        token: TokenKind,
        #[label]
        span: Span,
    },

    #[error("Failed to parse integer literal.")]
    InvalidIntegerLiteral {
        #[source]
        err: std::num::ParseIntError,
        #[label]
        span: Span,
    },
}

pub struct Parser<'src> {
    source: &'src str,
    tokens: TokenStream<'src>,
}

impl<'src> Parser<'src> {
    pub fn new(lexer: Lexer<'src>) -> Self {
        Self {
            source: lexer.source(),
            tokens: TokenStream::new(lexer, vec![TokenKind::Whitespace, TokenKind::Comment]),
        }
    }

    fn slice(&self, span: Span) -> &'src str {
        &self.source[span.0]
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token> {
        match self.tokens.next() {
            Some(token) if token.kind == kind => Ok(token),
            Some(token) => Err(ParsingError::UnexpectedToken {
                expected: kind.to_string(),
                got: token.kind,
                span: token.span,
            }),
            None => Err(ParsingError::UnexpectedEof {
                expected: kind.to_string(),
            }),
        }
    }

    fn expect_eof(&mut self) -> Result<()> {
        match self.tokens.next() {
            None => Ok(()),
            Some(token) => Err(ParsingError::ExtraToken {
                token: token.kind,
                span: token.span,
            }),
        }
    }

    pub fn parse_program(mut self) -> Result<ast::Program> {
        let function = self.parse_function_definition()?;
        self.expect_eof()?;
        Ok(ast::Program { function })
    }

    fn parse_function_definition(&mut self) -> Result<ast::FunctionDefinition> {
        let fun = self.expect(TokenKind::Fun)?;
        let name = self.parse_identifier()?;
        self.expect(TokenKind::Colon)?;
        self.expect(TokenKind::Int)?;
        self.expect(TokenKind::Equals)?;
        let body = self.parse_expression()?;
        let span = fun.span.to(body.span);

        Ok(ast::FunctionDefinition { name, body, span })
    }

    fn parse_expression(&mut self) -> Result<ast::Expression> {
        let Some(first_token) = self.tokens.peek() else {
            return Err(ParsingError::UnexpectedEof {
                expected: String::from("an expression"),
            });
        };
        match first_token.kind {
            TokenKind::IntegerLiteral => {
                let literal = self.parse_integer_literal()?;
                let span = literal.span;
                Ok(ast::Expression {
                    kind: ast::ExpressionKind::Constant(literal),
                    span,
                })
            }
            kind => Err(ParsingError::UnexpectedToken {
                expected: String::from("an expression"),
                got: kind,
                span: first_token.span,
            }),
        }
    }

    fn parse_identifier(&mut self) -> Result<ast::Identifier> {
        let token = self.expect(TokenKind::Identifier)?;
        let identifier = self.slice(token.span).to_owned();

        Ok(ast::Identifier {
            identifier,
            span: token.span,
        })
    }

    fn parse_integer_literal(&mut self) -> Result<ast::IntegerLiteral> {
        let token = self.expect(TokenKind::IntegerLiteral)?;
        let source = self.slice(token.span).replace('_', "");

        let value: i64 = source
            .parse()
            .map_err(|err| ParsingError::InvalidIntegerLiteral {
                err,
                span: token.span,
            })?;

        Ok(ast::IntegerLiteral {
            value,
            span: token.span,
        })
    }
}
