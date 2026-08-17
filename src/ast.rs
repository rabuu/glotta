use crate::span::Span;

#[derive(Debug, Clone)]
pub struct Program {
    pub function: FunctionDefinition,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ExpressionKind {
    Constant(IntegerLiteral),
    Call(CallKind),
}

#[derive(Debug, Clone)]
pub struct IntegerLiteral {
    pub value: i64,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum CallKind {
    Builtin(BuiltinCall),
    Function(FunctionCall),
}

#[derive(Debug, Clone)]
pub struct BuiltinCall {
    pub kind: BuiltinCallKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum BuiltinCallKind {
    BitwiseNot { arg: Box<Expression> },
    Negation { arg: Box<Expression> },
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub function_name: Identifier,
    pub args: Vec<Expression>,
    pub span: Span,
}
