use crate::span::{Span, Spanned};

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

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub name: Identifier,
    pub body: Expression,
    pub span: Span,
}

impl Spanned for FunctionDefinition {
    fn span(&self) -> Span {
        self.span
    }
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

#[derive(Debug, Clone)]
pub enum Expression {
    Constant(IntegerConstant),
    Call(Call),
}

impl Spanned for Expression {
    fn span(&self) -> Span {
        match self {
            Expression::Constant(constant) => constant.span(),
            Expression::Call(call) => call.span(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IntegerConstant {
    pub value: i32,
    pub span: Span,
}

impl Spanned for IntegerConstant {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone)]
pub enum Call {
    Builtin(BuiltinCall),
    Function(FunctionCall),
}

impl Spanned for Call {
    fn span(&self) -> Span {
        match self {
            Call::Builtin(call) => call.span(),
            Call::Function(call) => call.span(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BuiltinCall {
    Unary(UnaryOperation),
}

impl Spanned for BuiltinCall {
    fn span(&self) -> Span {
        match self {
            BuiltinCall::Unary(op) => op.span(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnaryOperation {
    pub operator: UnaryOperator,
    pub arg: Box<Expression>,
    pub span: Span,
}

impl Spanned for UnaryOperation {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UnaryOperator {
    pub kind: UnaryOperatorKind,
    pub span: Span,
}

impl Spanned for UnaryOperator {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOperatorKind {
    BitwiseNot,
    Negation,
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub function_name: Identifier,
    pub args: Vec<Expression>,
    pub span: Span,
}

impl Spanned for FunctionCall {
    fn span(&self) -> Span {
        self.span
    }
}
