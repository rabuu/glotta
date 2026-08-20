use miette::Diagnostic;
use thiserror::Error;

use crate::ast;
use crate::lexing::Lexer;
use crate::span::{Span, Spanned};
use crate::token::{Token, TokenKind};
use crate::token_stream::TokenStream;

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
        error: std::num::ParseIntError,
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
        &self.source[span.inner]
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
        let span = fun.span.to(body.span());

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
                let constant = self.parse_integer_constant()?;
                Ok(ast::Expression::Constant(constant))
            }
            TokenKind::Identifier => {
                // for now, an identifier always means a call.
                // this will change, once variables are implemented.

                let ident = self.parse_identifier()?;
                let start = ident.span;

                let _paren_l = self.expect(TokenKind::ParenL)?;
                let arg = self.parse_expression()?;
                let paren_r = self.expect(TokenKind::ParenR)?;
                let end = paren_r.span;

                let span = start.to(end);

                match ident.identifier.as_str() {
                    "bit_not" => Ok(ast::Expression::Call(ast::Call::Builtin(
                        ast::BuiltinCall::Unary(ast::UnaryOperation {
                            operator: ast::UnaryOperator {
                                kind: ast::UnaryOperatorKind::BitwiseNot,
                                span: ident.span,
                            },
                            arg: Box::new(arg),
                            span,
                        }),
                    ))),
                    "neg" => Ok(ast::Expression::Call(ast::Call::Builtin(
                        ast::BuiltinCall::Unary(ast::UnaryOperation {
                            operator: ast::UnaryOperator {
                                kind: ast::UnaryOperatorKind::Negation,
                                span: ident.span,
                            },
                            arg: Box::new(arg),
                            span,
                        }),
                    ))),
                    _ => Ok(ast::Expression::Call(ast::Call::Function(
                        ast::FunctionCall {
                            function_name: ident,
                            args: vec![arg],
                            span,
                        },
                    ))),
                }
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

    fn parse_integer_constant(&mut self) -> Result<ast::IntegerConstant> {
        let token = self.expect(TokenKind::IntegerLiteral)?;
        let source = self.slice(token.span).replace('_', "");

        let value: i32 = source
            .parse()
            .map_err(|error| ParsingError::InvalidIntegerLiteral {
                error,
                span: token.span,
            })?;

        Ok(ast::IntegerConstant {
            value,
            span: token.span,
        })
    }
}
