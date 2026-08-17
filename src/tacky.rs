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
}

#[derive(Debug, Clone)]
pub enum Value {
    Constant(i32),
    Variable(Identifier),
}

#[derive(Debug, Clone)]
pub enum Identifier {
    Temporary(usize),
}
