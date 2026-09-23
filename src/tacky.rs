use std::{fmt, io};

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
    Temporary { hint: String, id: usize },
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Identifier::Named(name) => write!(f, "{name}"),
            Identifier::Temporary { hint, id } => write!(f, "{hint}.{id}"),
        }
    }
}

pub struct Emitter<O: io::Write> {
    out: O,
}

impl<O: io::Write> Emitter<O> {
    const INDENT: &str = "    ";

    pub fn new(out: O) -> Self {
        Self { out }
    }

    pub fn emit_program(mut self, program: &Program) -> io::Result<()> {
        let Program { function } = program;
        self.emit_function_definition(function)
    }

    fn emit_function_definition(&mut self, function: &FunctionDefinition) -> io::Result<()> {
        let FunctionDefinition { name, body } = function;

        write!(self.out, "FUNCTION ")?;
        self.emit_identifier(name)?;
        self.newline()?;

        for instruction in body {
            self.indent()?;
            self.emit_instruction(instruction)?;
            self.newline()?;
        }

        Ok(())
    }

    fn emit_identifier(&mut self, identifier: &Identifier) -> io::Result<()> {
        match identifier {
            Identifier::Named(name) => write!(self.out, "{name}"),
            Identifier::Temporary { hint, id } => write!(self.out, "{hint}.{id}"),
        }
    }

    fn emit_instruction(&mut self, instruction: &Instruction) -> io::Result<()> {
        match instruction {
            Instruction::Return(value) => {
                write!(self.out, "RETURN ")?;
                self.emit_value(value)?;
                Ok(())
            }
            Instruction::Unary(Unary { op, src, dst }) => {
                self.emit_value(dst)?;
                write!(self.out, " <- {op:?} ")?;
                self.emit_value(src)?;
                Ok(())
            }
            Instruction::Binary(Binary { op, lhs, rhs, dst }) => {
                self.emit_value(dst)?;
                write!(self.out, " <- {op:?} ")?;
                self.emit_value(lhs)?;
                write!(self.out, ", ")?;
                self.emit_value(rhs)?;
                Ok(())
            }
            Instruction::Copy { src, dst } => {
                self.emit_value(dst)?;
                write!(self.out, " <- ")?;
                self.emit_value(src)?;
                Ok(())
            }
            Instruction::Jump(identifier) => {
                write!(self.out, "JUMP ")?;
                self.emit_identifier(identifier)?;
                Ok(())
            }
            Instruction::JumpIfZero { condition, target } => {
                write!(self.out, "JUMP ")?;
                self.emit_identifier(target)?;
                write!(self.out, " IF ")?;
                self.emit_value(condition)?;
                write!(self.out, " == 0")?;
                Ok(())
            }
            Instruction::JumpIfNotZero { condition, target } => {
                write!(self.out, "JUMP ")?;
                self.emit_identifier(target)?;
                write!(self.out, " IF ")?;
                self.emit_value(condition)?;
                write!(self.out, " != 0")?;
                Ok(())
            }
            Instruction::Label(identifier) => {
                write!(self.out, "LABEL ")?;
                self.emit_identifier(identifier)?;
                Ok(())
            }
        }
    }

    fn emit_value(&mut self, value: &Value) -> io::Result<()> {
        match value {
            Value::Constant(constant) => write!(self.out, "{constant}"),
            Value::Variable(identifier) => self.emit_identifier(identifier),
        }
    }

    fn newline(&mut self) -> io::Result<()> {
        writeln!(self.out)
    }

    fn indent(&mut self) -> io::Result<()> {
        write!(self.out, "{}", Self::INDENT)
    }
}
