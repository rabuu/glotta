use miette::Diagnostic;
use thiserror::Error;

use crate::ast;
use crate::parser::lexer::{IntegerLiteral, Lexer, LexingError, Token, TokenKind};
use crate::span::{Span, Spanned};

type Result<T> = std::result::Result<T, ParsingError>;

#[derive(Debug, Error, Diagnostic)]
pub enum ParsingError {
    #[error("Invalid token: {0}")]
    InvalidToken(#[from] LexingError),

    #[error("Expected token of kind `{expected}` but got `{got}`.")]
    UnexpectedToken {
        expected: TokenKind,
        got: TokenKind,
        #[label]
        span: Span,
    },

    #[error("Expected {expected} but got `{got}`.")]
    UnexpectedTokenIn {
        expected: String,
        got: TokenKind,
        #[label]
        span: Span,
    },

    #[error("Expected token `{expected}` but got EOF.")]
    UnexpectedEof { expected: TokenKind },

    #[error("Expected some token but got EOF.")]
    UnexpectedEofAny,

    #[error("Expected EOF but got `{token}`.")]
    ExtraToken {
        token: TokenKind,
        #[label]
        span: Span,
    },
}

pub struct Parser<'src> {
    lexer: Lexer<'src>,
}

impl<'src> Parser<'src> {
    pub fn new(lexer: Lexer<'src>) -> Self {
        Self { lexer }
    }

    fn peek_n(&mut self, n: usize) -> Result<TokenKind> {
        match self.lexer.peek_n(n) {
            Some(Ok((token, _))) => Ok(token.kind()),
            Some(Err(err)) => Err(err.clone().into()),
            None => Err(ParsingError::UnexpectedEofAny),
        }
    }

    fn peek(&mut self) -> Result<TokenKind> {
        self.peek_n(0)
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Spanned<Token<'src>>> {
        match self.lexer.next() {
            Some(Ok((token, span))) if token.kind() == kind => Ok((token, span)),
            Some(Ok((token, span))) => Err(ParsingError::UnexpectedToken {
                expected: kind,
                got: token.kind(),
                span,
            }),
            Some(Err(err)) => Err(err.into()),
            None => Err(ParsingError::UnexpectedEof { expected: kind }),
        }
    }

    fn expect_integer_literal(&mut self) -> Result<Spanned<IntegerLiteral<'src>>> {
        let (token, span) = self.expect(TokenKind::IntegerLiteral)?;
        match token {
            Token::IntegerLiteral(literal) => Ok((literal, span)),
            _ => unreachable!(),
        }
    }

    fn expect_identifier(&mut self) -> Result<Spanned<String>> {
        let (token, span) = self.expect(TokenKind::Identifier)?;
        match token {
            Token::Identifier(ident) => Ok((ident.to_owned(), span)),
            _ => unreachable!(),
        }
    }

    fn expect_eof(&mut self) -> Result<()> {
        match self.lexer.next() {
            None => Ok(()),
            Some(Err(err)) => Err(err.into()),
            Some(Ok((token, span))) => Err(ParsingError::ExtraToken {
                token: token.kind(),
                span,
            }),
        }
    }

    pub fn parse_program(mut self) -> Result<ast::Program> {
        let function = self.parse_function_definition()?;
        self.expect_eof()?;
        Ok(ast::Program { function })
    }

    pub fn parse_function_definition(&mut self) -> Result<ast::FunctionDefinition> {
        self.expect(TokenKind::Fun)?;
        let (name, name_span) = self.expect_identifier()?;
        self.expect(TokenKind::Equals)?;
        let body = self.parse_expression()?;

        Ok(ast::FunctionDefinition {
            name,
            name_span,
            body,
        })
    }

    pub fn parse_expression(&mut self) -> Result<ast::Expression> {
        match self.peek()? {
            TokenKind::IntegerLiteral => {
                let (literal, span) = self.expect_integer_literal()?;
                Ok(ast::Expression::Constant {
                    value: literal.literal,
                    span,
                })
            }
            _ => todo!(),
        }
    }
}
