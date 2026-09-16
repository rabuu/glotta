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
    pub span: Span,
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

    // binary
    Addition,
    Multiplication,
    Subtraction,
    Division,
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
