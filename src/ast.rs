use std::io;

use crate::elaboration::SymbolTable;
use crate::emitter::{emit, emitln};
use crate::span::{Span, Spanned};
use crate::{ast, emitter};

use glotta_macros::Spanned;

#[derive(Debug, Clone, Spanned)]
pub struct Program {
    pub functions: Vec<FunctionDefinition>,
    pub span: Span,
}

#[derive(Debug, Clone, Spanned)]
pub struct FunctionDefinition {
    pub name: Name,
    pub parameters: ParameterList,
    pub body: Expression,
    pub span: Span,
}

#[derive(Debug, Clone, Spanned)]
pub struct ParameterList {
    pub parameters: Vec<Parameter>,
    pub span: Span,
}

impl ParameterList {
    pub fn arity(&self) -> usize {
        self.parameters.len()
    }
}

#[derive(Debug, Clone, Spanned)]
pub struct Parameter {
    pub variable: Variable,
    // pub typ: Typ,
    pub span: Span,
}

#[derive(Debug, Clone, Spanned)]
pub struct Name {
    /// The ID gets assigned in the name resolution pass.
    /// Invariant: Before name resolution _every_ ID is 0;
    /// after name resolution _no_ ID is 0.
    pub id: usize,
    pub identifier: String,
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
    Conditional(Conditional),
    Block(Block),
    Loop(Loop),
    Break(Break),
    Continue(Continue),
}

#[derive(Debug, Clone, Spanned)]
pub struct IntegerConstant {
    pub value: i32,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: Name,
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
    pub function_name: Name,
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
pub struct Conditional {
    pub condition: Box<Expression>,
    pub then_branch: Box<Expression>,
    pub else_branch: Box<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone, Spanned)]
pub struct Block {
    pub statements: Vec<Expression>,
    pub final_expression: Box<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone, Spanned)]
pub struct Loop {
    /// The ID gets assigned in the loop labeling pass.
    /// Invariant: Before loop labeling _every_ ID is 0;
    /// afterwards _no_ ID is 0.
    pub id: usize,
    pub body: Box<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone, Spanned)]
pub struct Break {
    /// The ID gets assigned in the loop labeling pass.
    /// Invariant: Before loop labeling _every_ ID is 0;
    /// afterwards _no_ ID is 0.
    pub id: usize,
    pub span: Span,
}

#[derive(Debug, Clone, Spanned)]
pub struct Continue {
    /// The ID gets assigned in the loop labeling pass.
    /// Invariant: Before loop labeling _every_ ID is 0;
    /// afterwards _no_ ID is 0.
    pub id: usize,
    pub span: Span,
}

pub struct Emitter<O: io::Write> {
    e: emitter::Emitter<O>,
    symbol_table: Option<SymbolTable>,
}

impl<O: io::Write> Emitter<O> {
    pub fn new(out: O) -> Self {
        Self {
            e: emitter::Emitter::new(out, 2),
            symbol_table: None,
        }
    }

    pub fn with_symbol_table(mut self, symbol_table: SymbolTable) -> Self {
        self.symbol_table = Some(symbol_table);
        self
    }

    pub fn emit(mut self, program: &ast::Program) -> io::Result<()> {
        let ast::Program { functions, span: _ } = program;

        for function in functions {
            self.emit_function_definition(function)?;
        }

        Ok(())
    }

    fn emit_function_definition(&mut self, function: &ast::FunctionDefinition) -> io::Result<()> {
        let ast::FunctionDefinition {
            name,
            parameters,
            body,
            span: _,
        } = function;

        emit!(self.e, "FUNCTION ")?;
        self.emit_name(name)?;
        self.emit_parameters(parameters)?;
        emitln!(self.e)?;

        self.e.indent();
        self.emit_expression(body)?;
        emitln!(self.e)?;
        self.e.dedent();

        Ok(())
    }

    fn emit_parameters(&mut self, parameters: &ast::ParameterList) -> io::Result<()> {
        let ast::ParameterList {
            parameters,
            span: _,
        } = parameters;

        emit!(self.e, "(")?;
        for (i, parameter) in parameters.iter().enumerate() {
            if i > 0 {
                emit!(self.e, ", ")?;
            }
            self.emit_parameter(parameter)?;
        }
        emit!(self.e, ")")?;

        Ok(())
    }

    fn emit_parameter(&mut self, parameter: &ast::Parameter) -> io::Result<()> {
        let ast::Parameter { variable, span: _ } = parameter;
        self.emit_variable(variable)
    }

    fn emit_name(&mut self, name: &ast::Name) -> io::Result<()> {
        let ast::Name {
            identifier,
            id,
            span: _,
        } = name;

        emit!(self.e, "{identifier}")?;

        if *id != 0 {
            emit!(self.e, ".{id}")?;

            if let Some(symbol_table) = &self.symbol_table {
                if let Some(typ) = symbol_table.get(id) {
                    emit!(self.e, "[{typ}]")?;
                }
            }
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
            Expression::Conditional(conditional) => self.emit_conditional(conditional),
            Expression::Block(block) => self.emit_block(block),
            Expression::Loop(loop_expr) => self.emit_loop(loop_expr),
            Expression::Break(break_expr) => self.emit_break(break_expr),
            Expression::Continue(continue_expr) => self.emit_continue(continue_expr),
        }
    }

    fn emit_integer_constant(&mut self, constant: &ast::IntegerConstant) -> io::Result<()> {
        let ast::IntegerConstant { value, span: _ } = constant;
        emit!(self.e, "{value}")
    }

    fn emit_variable(&mut self, variable: &ast::Variable) -> io::Result<()> {
        let ast::Variable { name } = variable;
        self.emit_name(name)
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
        self.emit_name(function_name)?;

        self.e.indent();
        self.emit_arguments(arguments)?;
        self.e.dedent();

        Ok(())
    }

    fn emit_arguments(&mut self, arguments: &ast::ArgumentList) -> io::Result<()> {
        let ast::ArgumentList { arguments, span: _ } = arguments;

        for argument in arguments {
            emitln!(self.e)?;
            self.emit_expression(argument)?;
        }

        Ok(())
    }

    fn emit_conditional(&mut self, conditional: &ast::Conditional) -> io::Result<()> {
        let ast::Conditional {
            condition,
            then_branch,
            else_branch,
            span: _,
        } = conditional;

        emitln!(self.e, "CONDITIONAL")?;

        self.e.indent();

        emitln!(self.e, "IF")?;
        self.e.indent();
        self.emit_expression(condition)?;
        emitln!(self.e)?;
        self.e.dedent();

        emitln!(self.e, "THEN")?;
        self.e.indent();
        self.emit_expression(then_branch)?;
        emitln!(self.e)?;
        self.e.dedent();

        emitln!(self.e, "ELSE")?;
        self.e.indent();
        self.emit_expression(else_branch)?;
        self.e.dedent();

        self.e.dedent();

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

    fn emit_loop(&mut self, loop_expr: &ast::Loop) -> io::Result<()> {
        let ast::Loop { id, body, span: _ } = loop_expr;

        emit!(self.e, "LOOP")?;
        if *id != 0 {
            emit!(self.e, " {id}")?;
        }
        emitln!(self.e)?;

        self.e.indent();
        self.emit_expression(body)?;
        self.e.dedent();

        Ok(())
    }

    fn emit_break(&mut self, break_expr: &ast::Break) -> io::Result<()> {
        let ast::Break { id, span: _ } = break_expr;

        emit!(self.e, "BREAK")?;
        if *id != 0 {
            emit!(self.e, " {id}")?;
        }

        Ok(())
    }

    fn emit_continue(&mut self, continue_expr: &ast::Continue) -> io::Result<()> {
        let ast::Continue { id, span: _ } = continue_expr;

        emit!(self.e, "CONTINUE")?;
        if *id != 0 {
            emit!(self.e, " {id}")?;
        }

        Ok(())
    }
}
