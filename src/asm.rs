use std::io;

use crate::emitter;
use emitter::{emit, emitln};

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
    Mov {
        src: Operand,
        dst: Operand,
    },
    Add {
        src: Operand,
        dst: Operand,
    },
    Sub {
        src: Operand,
        dst: Operand,
    },
    IMul {
        src: Operand,
        dst: Operand,
    },
    Cmp {
        src: Operand,
        dst: Operand,
    },
    Not(Operand),
    Neg(Operand),
    IDiv(Operand),
    Push(Operand),
    Pop(Operand),
    Cdq,
    Jmp(String),
    JmpCC {
        flag: ConditionalFlag,
        label: String,
    },
    SetCC {
        flag: ConditionalFlag,
        op: Operand,
    },
    Label(String),
    Ret,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Immediate(isize),
    Register(Register),
    Pseudo(String),
    Stack { offset: isize },
}

impl From<Register> for Operand {
    fn from(value: Register) -> Self {
        Self::Register(value)
    }
}

#[derive(Debug, Clone)]
pub enum Register {
    /// stack pointer
    RSP,

    /// base pointer
    RBP,

    // 1-byte
    AL,
    DL,
    R10B,
    R11B,

    // 4-byte
    EAX,
    EDX,
    R10D,
    R11D,
}

impl Register {
    pub const TEMP_SRC: Register = Register::R10D;
    pub const TEMP_DST: Register = Register::R11D;
}

#[derive(Debug, Clone, Copy)]
pub enum ConditionalFlag {
    /// equal
    E,
    /// not equal
    NE,
    /// greater
    G,
    /// greater or equal
    GE,
    /// less
    L,
    /// less or equal
    LE,
}

pub struct Emitter<O: io::Write> {
    e: emitter::Emitter<O>,
}

impl<O: io::Write> Emitter<O> {
    pub fn new(out: O) -> Self {
        Self {
            e: emitter::Emitter::new(out),
        }
    }

    pub fn emit(mut self, program: &Program) -> io::Result<()> {
        let Program { function } = program;

        emitln!(self.e, "section .text\n")?;
        self.emit_function_definition(function)?;

        emitln!(self.e)?;
        emitln!(
            self.e,
            "section .note.GNU-stack noalloc noexec nowrite progbits"
        )?;

        self.e.flush()?;

        Ok(())
    }

    fn emit_function_definition(&mut self, function: &FunctionDefinition) -> io::Result<()> {
        let FunctionDefinition { name, instructions } = function;

        emitln!(self.e, "global {name}")?;
        emitln!(self.e, "{name}:")?;

        self.e.indent();
        for instruction in instructions {
            self.emit_instruction(instruction)?;
            emitln!(self.e)?;
        }
        self.e.dedent();

        Ok(())
    }

    fn emit_instruction(&mut self, instruction: &Instruction) -> io::Result<()> {
        match instruction {
            Instruction::Mov { src, dst } => {
                emit!(self.e, "mov ")?;
                self.emit_operand(dst)?;
                emit!(self.e, ", ")?;
                self.emit_operand(src)?;
                Ok(())
            }
            Instruction::Sub { src, dst } => {
                emit!(self.e, "sub ")?;
                self.emit_operand(dst)?;
                emit!(self.e, ", ")?;
                self.emit_operand(src)?;
                Ok(())
            }
            Instruction::Add { src, dst } => {
                emit!(self.e, "add ")?;
                self.emit_operand(dst)?;
                emit!(self.e, ", ")?;
                self.emit_operand(src)?;
                Ok(())
            }
            Instruction::IMul { src, dst } => {
                emit!(self.e, "imul ")?;
                self.emit_operand(dst)?;
                emit!(self.e, ", ")?;
                self.emit_operand(src)?;
                Ok(())
            }
            Instruction::Cmp { src, dst } => {
                emit!(self.e, "cmp ")?;
                self.emit_operand(dst)?;
                emit!(self.e, ", ")?;
                self.emit_operand(src)?;
                Ok(())
            }
            Instruction::Not(operand) => {
                emit!(self.e, "not ")?;
                self.emit_operand(operand)?;
                Ok(())
            }
            Instruction::Neg(operand) => {
                emit!(self.e, "neg ")?;
                self.emit_operand(operand)?;
                Ok(())
            }
            Instruction::IDiv(operand) => {
                emit!(self.e, "idiv ")?;
                self.emit_operand(operand)?;
                Ok(())
            }
            Instruction::Cdq => emit!(self.e, "cdq"),
            Instruction::Push(operand) => {
                emit!(self.e, "push ")?;
                self.emit_operand(operand)?;
                Ok(())
            }
            Instruction::Pop(operand) => {
                emit!(self.e, "pop ")?;
                self.emit_operand(operand)?;
                Ok(())
            }
            Instruction::Jmp(label) => emit!(self.e, "jmp .L{label}"),
            Instruction::JmpCC { flag, label } => {
                emit!(self.e, "j")?;
                self.emit_conditional_flag(flag)?;
                emit!(self.e, " .L{label}")?;
                Ok(())
            }
            Instruction::SetCC { flag, op } => {
                emit!(self.e, "set")?;
                self.emit_conditional_flag(flag)?;
                emit!(self.e, " ")?;
                self.emit_operand(op)?;
                Ok(())
            }
            Instruction::Label(label) => {
                self.e.dedent();
                emit!(self.e, ".L{label}:")?;
                self.e.indent();
                Ok(())
            }
            Instruction::Ret => emit!(self.e, "ret"),
        }
    }

    fn emit_operand(&mut self, operand: &Operand) -> io::Result<()> {
        match operand {
            Operand::Immediate(int) => emit!(self.e, "{int}"),
            Operand::Register(register) => self.emit_register(register),
            Operand::Pseudo(pseudo) => emit!(self.e, "<{pseudo}>"),
            Operand::Stack { offset } => {
                let sign = if offset.is_negative() { "-" } else { "+" };
                let abs = offset.abs();
                emit!(self.e, "[rbp {sign} {abs}]")
            }
        }
    }

    fn emit_register(&mut self, register: &Register) -> io::Result<()> {
        match register {
            Register::RSP => emit!(self.e, "rsp"),
            Register::RBP => emit!(self.e, "rbp"),
            Register::AL => emit!(self.e, "al"),
            Register::DL => emit!(self.e, "dl"),
            Register::R10B => emit!(self.e, "r10b"),
            Register::R11B => emit!(self.e, "r11b"),
            Register::EAX => emit!(self.e, "eax"),
            Register::EDX => emit!(self.e, "edx"),
            Register::R10D => emit!(self.e, "r10d"),
            Register::R11D => emit!(self.e, "r11d"),
        }
    }

    fn emit_conditional_flag(&mut self, flag: &ConditionalFlag) -> io::Result<()> {
        match flag {
            ConditionalFlag::E => emit!(self.e, "e"),
            ConditionalFlag::NE => emit!(self.e, "ne"),
            ConditionalFlag::G => emit!(self.e, "g"),
            ConditionalFlag::GE => emit!(self.e, "ge"),
            ConditionalFlag::L => emit!(self.e, "l"),
            ConditionalFlag::LE => emit!(self.e, "le"),
        }
    }
}
