use crate::span::{Span, Spanned};

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
    Call(Call),
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
pub enum Call {
    Builtin(BuiltinCall),
    Function(FunctionCall),
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
    pub inner: Vec<Expression>,
    pub span: Span,
}

impl ArgumentList {
    pub fn arity(&self) -> usize {
        self.inner.len()
    }
}

#[derive(Debug, Clone, Spanned)]
pub struct Block {
    pub statements: Vec<Expression>,
    pub final_expression: Box<Expression>,
    pub span: Span,
}
