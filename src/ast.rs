use std::io;

use crate::emitter::{emit, emitln};
use crate::span::{Span, Spanned};
use crate::{ast, emitter};

use glotta_macros::Spanned;

#[derive(Debug, Clone)]
pub struct Program {
    pub function: FunctionDefinition,
}

impl Spanned for Program {
    fn span(&self) -> Span {
        let Self { function } = self;
        function.span()
    }
}

#[derive(Debug, Clone, Spanned)]
pub struct FunctionDefinition {
    pub name: Identifier,
    pub body: Expression,
    pub span: Span,
}

#[derive(Debug, Clone, Spanned)]
pub struct Identifier {
    pub identifier: String,

    /// Unique name ID
    ///
    /// The ID of 0 means "not yet assigned".
    /// Invariant: Before name resolution _every_ ID is 0;
    /// after name resolution _no_ ID is 0.
    pub id: usize,

    pub span: Span,
}

#[derive(Debug, Clone, Spanned)]
pub enum Expression {
    Constant(IntegerConstant),
    Variable(Variable),
    Declaration(Declaration),
    Assignment(Assignment),
    BuiltinCall(BuiltinCall),
    FunctionCall(FunctionCall),
    Block(Block),
}

#[derive(Debug, Clone, Spanned)]
pub struct IntegerConstant {
    pub value: i32,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: Identifier,
}

impl Spanned for Variable {
    fn span(&self) -> Span {
        self.name.span()
    }
}

#[derive(Debug, Clone, Spanned)]
pub struct Declaration {
    pub variable: Variable,
    pub initializer: Box<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone, Spanned)]
pub struct Assignment {
    pub lhs: Box<Expression>,
    pub rhs: Box<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone, Spanned)]
pub struct BuiltinCall {
    pub operator: BuiltinOperator,
    pub arguments: ArgumentList,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, Spanned)]
pub struct BuiltinOperator {
    pub kind: BuiltinOperatorKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub enum BuiltinOperatorKind {
    // unary
    BitwiseNot,
    Negation,
    Not,

    // binary
    Addition,
    Multiplication,
    Subtraction,
    Division,
    Remainder,
    And,
    Or,
    Equal,
    NotEqual,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
}

#[derive(Debug, Clone, Spanned)]
pub struct FunctionCall {
    pub function_name: Identifier,
    pub arguments: ArgumentList,
    pub span: Span,
}

#[derive(Debug, Clone, Spanned)]
pub struct ArgumentList {
    pub arguments: Vec<Expression>,
    pub span: Span,
}

impl ArgumentList {
    pub fn arity(&self) -> usize {
        self.arguments.len()
    }
}

#[derive(Debug, Clone, Spanned)]
pub struct Block {
    pub statements: Vec<Expression>,
    pub final_expression: Box<Expression>,
    pub span: Span,
}

pub struct Emitter<O: io::Write> {
    e: emitter::Emitter<O>,
}

impl<O: io::Write> Emitter<O> {
    pub fn new(out: O) -> Self {
        Self {
            e: emitter::Emitter::new(out),
        }
    }

    pub fn emit(mut self, program: &ast::Program) -> io::Result<()> {
        let ast::Program { function } = program;
        self.emit_function_definition(function)
    }

    fn emit_function_definition(&mut self, function: &ast::FunctionDefinition) -> io::Result<()> {
        let ast::FunctionDefinition {
            name,
            body,
            span: _,
        } = function;

        emit!(self.e, "FUNCTION ")?;
        self.emit_identifier(name)?;
        emitln!(self.e)?;

        self.e.indent();
        self.emit_expression(body)?;
        emitln!(self.e)?;
        self.e.dedent();

        Ok(())
    }

    fn emit_identifier(&mut self, identifier: &ast::Identifier) -> io::Result<()> {
        let ast::Identifier {
            identifier,
            id,
            span: _,
        } = identifier;

        emit!(self.e, "{identifier}")?;

        if *id != 0 {
            emit!(self.e, ".{id}")?;
        }

        Ok(())
    }

    fn emit_expression(&mut self, expression: &ast::Expression) -> io::Result<()> {
        match expression {
            Expression::Constant(constant) => self.emit_integer_constant(constant),
            Expression::Variable(variable) => self.emit_variable(variable),
            Expression::Declaration(declaration) => self.emit_declaration(declaration),
            Expression::Assignment(assignment) => self.emit_assignment(assignment),
            Expression::BuiltinCall(call) => self.emit_builtin_call(call),
            Expression::FunctionCall(call) => self.emit_function_call(call),
            Expression::Block(block) => self.emit_block(block),
        }
    }

    fn emit_integer_constant(&mut self, constant: &ast::IntegerConstant) -> io::Result<()> {
        let ast::IntegerConstant { value, span: _ } = constant;
        emit!(self.e, "{value}")
    }

    fn emit_variable(&mut self, variable: &ast::Variable) -> io::Result<()> {
        let ast::Variable { name } = variable;
        self.emit_identifier(name)
    }

    fn emit_declaration(&mut self, declaration: &ast::Declaration) -> io::Result<()> {
        let ast::Declaration {
            variable,
            initializer,
            span: _,
        } = declaration;

        emit!(self.e, "LET ")?;
        self.emit_variable(variable)?;
        emitln!(self.e)?;

        self.e.indent();
        self.emit_expression(initializer)?;
        self.e.dedent();

        Ok(())
    }

    fn emit_assignment(&mut self, assignment: &ast::Assignment) -> io::Result<()> {
        let ast::Assignment { lhs, rhs, span: _ } = assignment;

        emitln!(self.e, "SET")?;

        self.e.indent();

        self.emit_expression(lhs)?;
        emitln!(self.e)?;
        self.emit_expression(rhs)?;

        self.e.dedent();

        Ok(())
    }

    fn emit_builtin_call(&mut self, call: &ast::BuiltinCall) -> io::Result<()> {
        let ast::BuiltinCall {
            operator,
            arguments,
            span: _,
        } = call;

        emit!(self.e, "BUILTIN CALL ")?;
        self.emit_builtin_operator(operator)?;
        emitln!(self.e)?;

        self.e.indent();
        self.emit_arguments(arguments)?;
        self.e.dedent();

        Ok(())
    }

    fn emit_builtin_operator(&mut self, operator: &ast::BuiltinOperator) -> io::Result<()> {
        let ast::BuiltinOperator { kind, span: _ } = operator;
        emit!(self.e, "{kind:?}")
    }

    fn emit_function_call(&mut self, call: &ast::FunctionCall) -> io::Result<()> {
        let ast::FunctionCall {
            function_name,
            arguments,
            span: _,
        } = call;

        emit!(self.e, "FUNCTION CALL ")?;
        self.emit_identifier(function_name)?;
        emitln!(self.e)?;

        self.e.indent();
        self.emit_arguments(arguments)?;
        self.e.dedent();

        Ok(())
    }

    fn emit_arguments(&mut self, arguments: &ast::ArgumentList) -> io::Result<()> {
        let ast::ArgumentList { arguments, span: _ } = arguments;

        for (i, argument) in arguments.iter().enumerate() {
            self.emit_expression(argument)?;
            if i < arguments.len() - 1 {
                emitln!(self.e)?;
            }
        }

        Ok(())
    }

    fn emit_block(&mut self, block: &ast::Block) -> io::Result<()> {
        let ast::Block {
            statements,
            final_expression,
            span: _,
        } = block;

        emitln!(self.e, "BLOCK")?;

        self.e.indent();

        for statement in statements {
            self.emit_expression(statement)?;
            emitln!(self.e)?;
        }
        self.emit_expression(final_expression)?;

        self.e.dedent();

        Ok(())
    }
}
