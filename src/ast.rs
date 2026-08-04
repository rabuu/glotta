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
}

#[derive(Debug)]
pub struct IntegerLiteral {
    pub value: i64,
    pub span: Span,
}
