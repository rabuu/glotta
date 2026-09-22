use std::fmt;

#[derive(Debug, Clone)]
pub struct Program {
    pub function: FunctionDefinition,
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub name: Identifier,
    pub body: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Return(Value),
    Unary(Unary),
    Binary(Binary),
    Copy {
        src: Value,
        dst: Value,
    },
    Jump(Identifier),
    JumpIfZero {
        condition: Value,
        target: Identifier,
    },
    JumpIfNotZero {
        condition: Value,
        target: Identifier,
    },
    Label(Identifier),
}

#[derive(Debug, Clone)]
pub struct Unary {
    pub op: UnaryOperator,
    pub src: Value,
    pub dst: Value,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOperator {
    BitwiseNot,
    Negation,
    Not,
}

#[derive(Debug, Clone)]
pub struct Binary {
    pub op: BinaryOperator,
    pub lhs: Value,
    pub rhs: Value,
    pub dst: Value,
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOperator {
    Addition,
    Multiplication,
    Subtraction,
    Division,
    Remainder,
    Equal,
    NotEqual,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
}

#[derive(Debug, Clone)]
pub enum Value {
    Constant(i32),
    Variable(Identifier),
}

#[derive(Debug, Clone)]
pub enum Identifier {
    Named(String),
    Temporary(usize),
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Identifier::Named(name) => write!(f, "{name}"),
            Identifier::Temporary(tmp) => write!(f, "tmp.{tmp}"),
        }
    }
}
