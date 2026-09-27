use std::{fmt, io};

use crate::emitter;
use crate::emitter::{emit, emitln};

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
    Function(String),
    Variable { id: usize, name: String },
    Loop { id: usize, kind: String },
    Temporary { id: usize, hint: String },
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Identifier::Function(name) => write!(f, "{name}"),
            Identifier::Variable { id, name } => write!(f, ".var.{id}.{name}"),
            Identifier::Loop { id, kind } => write!(f, ".loop.{id}.{kind}"),
            Identifier::Temporary { id, hint } => write!(f, ".tmp.{id}.{hint}"),
        }
    }
}

pub struct Emitter<O: io::Write> {
    e: emitter::Emitter<O>,
}

impl<O: io::Write> Emitter<O> {
    pub fn new(out: O) -> Self {
        Self {
            e: emitter::Emitter::new(out, 4),
        }
    }

    pub fn emit(mut self, program: &Program) -> io::Result<()> {
        let Program { function } = program;
        self.emit_function_definition(function)?;
        self.e.flush()?;
        Ok(())
    }

    fn emit_function_definition(&mut self, function: &FunctionDefinition) -> io::Result<()> {
        let FunctionDefinition { name, body } = function;

        emitln!(self.e, "FUNCTION {name}")?;

        self.e.indent();
        for instruction in body {
            self.emit_instruction(instruction)?;
            emitln!(self.e)?;
        }
        self.e.dedent();

        Ok(())
    }

    fn emit_instruction(&mut self, instruction: &Instruction) -> io::Result<()> {
        match instruction {
            Instruction::Return(value) => {
                emit!(self.e, "RETURN ")?;
                self.emit_value(value)?;
                Ok(())
            }
            Instruction::Unary(Unary { op, src, dst }) => {
                self.emit_value(dst)?;
                emit!(self.e, " <- {op:?} ")?;
                self.emit_value(src)?;
                Ok(())
            }
            Instruction::Binary(Binary { op, lhs, rhs, dst }) => {
                self.emit_value(dst)?;
                emit!(self.e, " <- {op:?} ")?;
                self.emit_value(lhs)?;
                emit!(self.e, ", ")?;
                self.emit_value(rhs)?;
                Ok(())
            }
            Instruction::Copy { src, dst } => {
                self.emit_value(dst)?;
                emit!(self.e, " <- ")?;
                self.emit_value(src)?;
                Ok(())
            }
            Instruction::Jump(target) => emit!(self.e, "JUMP {target}"),
            Instruction::JumpIfZero { condition, target } => {
                emit!(self.e, "JUMP {target} IF ")?;
                self.emit_value(condition)?;
                emit!(self.e, " == 0")?;
                Ok(())
            }
            Instruction::JumpIfNotZero { condition, target } => {
                emit!(self.e, "JUMP {target} IF ")?;
                self.emit_value(condition)?;
                emit!(self.e, " != 0")?;
                Ok(())
            }
            Instruction::Label(label) => emit!(self.e, "LABEL {label}"),
        }
    }

    fn emit_value(&mut self, value: &Value) -> io::Result<()> {
        match value {
            Value::Constant(constant) => emit!(self.e, "{constant}"),
            Value::Variable(identifier) => emit!(self.e, "{identifier}"),
        }
    }
}
