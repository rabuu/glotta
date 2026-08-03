use crate::span::Span;

#[derive(Debug)]
pub struct Program {
    pub function: FunctionDefinition,
}

#[derive(Debug)]
pub struct FunctionDefinition {
    pub name: String,
    pub name_span: Span,
    pub body: Expression,
}

#[derive(Debug)]
pub enum Expression {
    Constant { value: i64, span: Span },
}
