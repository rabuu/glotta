#[derive(Debug, Clone)]
pub struct Program {
    pub function: FunctionDefinition,
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub name: String,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Mov { src: Operand, dst: Operand },
    Sub { src: Operand, dst: Operand },
    Not(Operand),
    Neg(Operand),
    Push(Operand),
    Pop(Operand),
    Ret,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Immediate(isize),
    Register(Register),
    Pseudo(String),
    Stack { offset: isize },
}

#[derive(Debug, Clone)]
pub enum Register {
    /// stack pointer
    RSP,

    /// base pointer
    RBP,

    EAX,
    R10D,
}
