use crate::span::Span;

#[derive(Debug)]
pub struct Program {
    pub function: FunctionDefinition,
}

#[derive(Debug)]
pub struct FunctionDefinition {
    pub name: Identifier,
    pub body: Expression,
    pub span: Span,
}

#[derive(Debug)]
pub struct Identifier {
    pub identifier: String,
    pub span: Span,
}

#[derive(Debug)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub span: Span,
}

#[derive(Debug)]
pub enum ExpressionKind {
    Constant(IntegerLiteral),
    Call(CallKind),
}

#[derive(Debug)]
pub struct IntegerLiteral {
    pub value: i64,
    pub span: Span,
}

#[derive(Debug)]
pub enum CallKind {
    Builtin(BuiltinCall),
    Function(FunctionCall),
}

#[derive(Debug)]
pub struct BuiltinCall {
    pub kind: BuiltinCallKind,
    pub span: Span,
}

#[derive(Debug)]
pub enum BuiltinCallKind {
    BitwiseNot { arg: Box<Expression> },
    Negation { arg: Box<Expression> },
}

#[derive(Debug)]
pub struct FunctionCall {
    pub function_name: Identifier,
    pub args: Vec<Expression>,
    pub span: Span,
}
