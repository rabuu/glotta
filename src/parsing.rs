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
        let let_kw = self.expect(TokenKind::Let)?;
        let name = self.parse_identifier()?;
        self.expect(TokenKind::Colon)?;
        self.expect(TokenKind::ParenL)?;
        self.expect(TokenKind::ParenR)?;
        self.expect(TokenKind::Arrow)?;
        self.expect(TokenKind::Int)?;
        self.expect(TokenKind::Equals)?;
        let body = self.parse_expression()?;
        let span = let_kw.span.to(body.span());

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
                let start = ident.span();

                let arguments = self.parse_arguments()?;
                let end = arguments.span();

                let span = start.to(end);

                let builtin_call = match ident.identifier.as_str() {
                    "bit_not" => Some(ast::BuiltinOperatorKind::BitwiseNot),
                    "neg" => Some(ast::BuiltinOperatorKind::Negation),
                    "add" => Some(ast::BuiltinOperatorKind::Addition),
                    "mul" => Some(ast::BuiltinOperatorKind::Multiplication),
                    "sub" => Some(ast::BuiltinOperatorKind::Subtraction),
                    "div" => Some(ast::BuiltinOperatorKind::Division),
                    _ => None,
                };

                let call = match builtin_call {
                    Some(kind) => ast::Call::Builtin(ast::BuiltinCall {
                        operator: ast::BuiltinOperator {
                            kind,
                            span: ident.span(),
                        },
                        arguments,
                        span,
                    }),
                    None => ast::Call::Function(ast::FunctionCall {
                        function_name: ident,
                        arguments,
                        span,
                    }),
                };

                Ok(ast::Expression::Call(call))
            }
            got => Err(ParsingError::UnexpectedToken {
                expected: String::from("an expression"),
                got,
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

    fn parse_arguments(&mut self) -> Result<ast::ArgumentList> {
        let left = self.expect(TokenKind::ParenL)?;

        let mut arguments = Vec::new();

        loop {
            match self.tokens.peek().map(|tok| tok.kind) {
                Some(TokenKind::ParenR) => break,
                Some(_) => {
                    let expr = self.parse_expression()?;
                    arguments.push(expr);

                    match self.tokens.peek() {
                        Some(token) if token.kind == TokenKind::ParenR => break,
                        Some(token) if token.kind == TokenKind::Comma => {
                            self.tokens.next().unwrap();
                            continue;
                        }
                        Some(token) => {
                            return Err(ParsingError::UnexpectedToken {
                                expected: format!("{} or {}", TokenKind::ParenR, TokenKind::Comma),
                                got: token.kind,
                                span: token.span,
                            });
                        }
                        None => {
                            return Err(ParsingError::UnexpectedEof {
                                expected: format!("{} or {}", TokenKind::ParenR, TokenKind::Comma),
                            });
                        }
                    }
                }
                None => {
                    return Err(ParsingError::UnexpectedEof {
                        expected: "argument list".to_string(),
                    });
                }
            }
        }

        let right = self.expect(TokenKind::ParenR)?;

        return Ok(ast::ArgumentList {
            inner: arguments,
            span: left.span.to(right.span),
        });
    }
}
