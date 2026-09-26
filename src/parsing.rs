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

    #[error("Encountered unknown builtin `{got}`.")]
    UnknownBuiltin {
        got: String,
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
        let span = let_kw.to(&body);

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
            TokenKind::Hash => {
                let builtin_call = self.parse_builtin_call()?;
                Ok(ast::Expression::Call(ast::Call::Builtin(builtin_call)))
            }
            TokenKind::Identifier => {
                let next_token = self.tokens.peek_kind_n(1);
                if next_token == Some(TokenKind::ParenL) {
                    let function_call = self.parse_function_call()?;
                    Ok(ast::Expression::Call(ast::Call::Function(function_call)))
                } else {
                    let variable = self.parse_variable()?;
                    Ok(ast::Expression::Variable(variable))
                }
            }
            TokenKind::Let => {
                let declaration = self.parse_declaration()?;
                Ok(ast::Expression::Declaration(declaration))
            }
            TokenKind::Set => {
                let assignment = self.parse_assignment()?;
                Ok(ast::Expression::Assignment(assignment))
            }
            TokenKind::CurlyL => {
                let block = self.parse_block()?;
                Ok(ast::Expression::Block(block))
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
            id: 0,
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

    fn parse_variable(&mut self) -> Result<ast::Variable> {
        let name = self.parse_identifier()?;
        Ok(ast::Variable { name })
    }

    fn parse_declaration(&mut self) -> Result<ast::Declaration> {
        let let_kw = self.expect(TokenKind::Let)?;
        let variable = self.parse_variable()?;
        self.expect(TokenKind::Equals)?;
        let initializer = self.parse_expression()?;
        let span = let_kw.to(&initializer);

        Ok(ast::Declaration {
            variable,
            initializer: Box::new(initializer),
            span,
        })
    }

    fn parse_assignment(&mut self) -> Result<ast::Assignment> {
        let set_kw = self.expect(TokenKind::Set)?;
        let lhs = self.parse_expression()?;
        self.expect(TokenKind::Equals)?;
        let rhs = self.parse_expression()?;
        let span = set_kw.to(&rhs);

        Ok(ast::Assignment {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            span,
        })
    }

    fn parse_builtin_call(&mut self) -> Result<ast::BuiltinCall> {
        let start = self.expect(TokenKind::Hash)?;

        let builtin = self.parse_identifier()?;
        let kind = match builtin.identifier.as_str() {
            "bit_not" => ast::BuiltinOperatorKind::BitwiseNot,
            "neg" => ast::BuiltinOperatorKind::Negation,
            "not" => ast::BuiltinOperatorKind::Not,
            "add" => ast::BuiltinOperatorKind::Addition,
            "mul" => ast::BuiltinOperatorKind::Multiplication,
            "sub" => ast::BuiltinOperatorKind::Subtraction,
            "div" => ast::BuiltinOperatorKind::Division,
            "rem" => ast::BuiltinOperatorKind::Remainder,
            "and" => ast::BuiltinOperatorKind::And,
            "or" => ast::BuiltinOperatorKind::Or,
            "eq" => ast::BuiltinOperatorKind::Equal,
            "neq" => ast::BuiltinOperatorKind::NotEqual,
            "lt" => ast::BuiltinOperatorKind::LessThan,
            "leq" => ast::BuiltinOperatorKind::LessOrEqual,
            "gt" => ast::BuiltinOperatorKind::GreaterThan,
            "geq" => ast::BuiltinOperatorKind::GreaterOrEqual,
            got => {
                return Err(ParsingError::UnknownBuiltin {
                    got: got.to_string(),
                    span: start.to(builtin),
                });
            }
        };

        let arguments = self.parse_arguments()?;
        let end = arguments.span;

        Ok(ast::BuiltinCall {
            operator: ast::BuiltinOperator {
                kind,
                span: start.to(builtin),
            },
            arguments,
            span: start.to(end),
        })
    }

    fn parse_function_call(&mut self) -> Result<ast::FunctionCall> {
        let function_name = self.parse_identifier()?;
        let arguments = self.parse_arguments()?;
        let span = function_name.to(&arguments);

        Ok(ast::FunctionCall {
            function_name,
            arguments,
            span,
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

        Ok(ast::ArgumentList {
            inner: arguments,
            span: left.to(right),
        })
    }

    fn parse_block(&mut self) -> Result<ast::Block> {
        let start = self.expect(TokenKind::CurlyL)?;

        let mut statements = Vec::new();

        let final_expression = loop {
            let expression = self.parse_expression()?;

            if self.tokens.peek_kind() == Some(TokenKind::Semicolon) {
                self.expect(TokenKind::Semicolon)?;
                statements.push(expression);
            } else {
                break expression;
            }
        };

        let end = self.expect(TokenKind::CurlyR)?;

        Ok(ast::Block {
            statements,
            final_expression: Box::new(final_expression),
            span: start.to(end),
        })
    }
}
