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

#[derive(Debug, Clone)]
pub struct Identifier {
    pub identifier: String,
    pub span: Span,
}

impl Spanned for Identifier {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, Spanned)]
pub enum Expression {
    Constant(IntegerConstant),
    Call(Call),
}

#[derive(Debug, Clone, Spanned)]
pub struct IntegerConstant {
    pub value: i32,
    pub span: Span,
}

#[derive(Debug, Clone, Spanned)]
pub enum Call {
    Builtin(BuiltinCall),
    Function(FunctionCall),
}

#[derive(Debug, Clone, Spanned)]
pub enum BuiltinCall {
    Unary(UnaryOperation),
}

#[derive(Debug, Clone, Spanned)]
pub struct UnaryOperation {
    pub operator: UnaryOperator,
    pub arg: Box<Expression>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, Spanned)]
pub struct UnaryOperator {
    pub kind: UnaryOperatorKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOperatorKind {
    BitwiseNot,
    Negation,
}

#[derive(Debug, Clone, Spanned)]
pub struct FunctionCall {
    pub function_name: Identifier,
    pub args: Vec<Expression>,
    pub span: Span,
}
