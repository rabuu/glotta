use std::io;

const INDENT: &str = "    ";

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
    Add { src: Operand, dst: Operand },
    Sub { src: Operand, dst: Operand },
    IMul { src: Operand, dst: Operand },
    Not(Operand),
    Neg(Operand),
    IDiv(Operand),
    Push(Operand),
    Pop(Operand),
    Cdq,
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
    EDX,
    R10D,
    R11D,
}

impl Register {
    pub const TEMP_SRC: Register = Register::R10D;
    pub const TEMP_DST: Register = Register::R11D;
}

pub struct Emitter<O: io::Write> {
    output: O,
}

impl<O: io::Write> Emitter<O> {
    pub fn new(output: O) -> Self {
        Self { output }
    }

    pub fn emit_program(mut self, program: &Program) -> io::Result<()> {
        let Program { function } = program;

        writeln!(self.output, "section .text\n")?;
        self.emit_function_definition(function)?;

        self.newline()?;
        writeln!(
            self.output,
            "section .note.GNU-stack noalloc noexec nowrite progbits"
        )?;

        self.output.flush()?;

        Ok(())
    }

    fn emit_function_definition(&mut self, function: &FunctionDefinition) -> io::Result<()> {
        let FunctionDefinition { name, instructions } = function;

        writeln!(self.output, "global {name}")?;
        writeln!(self.output, "{name}:")?;
        for instruction in instructions {
            write!(self.output, "{INDENT}")?;
            self.emit_instruction(instruction)?;
            self.newline()?;
        }

        Ok(())
    }

    fn emit_instruction(&mut self, instruction: &Instruction) -> io::Result<()> {
        match instruction {
            Instruction::Mov { src, dst } => {
                write!(self.output, "mov ")?;
                self.emit_operand(dst)?;
                write!(self.output, ", ")?;
                self.emit_operand(src)?;
                Ok(())
            }
            Instruction::Sub { src, dst } => {
                write!(self.output, "sub ")?;
                self.emit_operand(dst)?;
                write!(self.output, ", ")?;
                self.emit_operand(src)?;
                Ok(())
            }
            Instruction::Add { src, dst } => {
                write!(self.output, "add ")?;
                self.emit_operand(dst)?;
                write!(self.output, ", ")?;
                self.emit_operand(src)?;
                Ok(())
            }
            Instruction::IMul { src, dst } => {
                write!(self.output, "imul ")?;
                self.emit_operand(dst)?;
                write!(self.output, ", ")?;
                self.emit_operand(src)?;
                Ok(())
            }
            Instruction::Not(operand) => {
                write!(self.output, "not ")?;
                self.emit_operand(operand)?;
                Ok(())
            }
            Instruction::Neg(operand) => {
                write!(self.output, "neg ")?;
                self.emit_operand(operand)?;
                Ok(())
            }
            Instruction::IDiv(operand) => {
                write!(self.output, "idiv ")?;
                self.emit_operand(operand)?;
                Ok(())
            }
            Instruction::Cdq => write!(self.output, "cdq"),
            Instruction::Push(operand) => {
                write!(self.output, "push ")?;
                self.emit_operand(operand)?;
                Ok(())
            }
            Instruction::Pop(operand) => {
                write!(self.output, "pop ")?;
                self.emit_operand(operand)?;
                Ok(())
            }
            Instruction::Ret => write!(self.output, "ret"),
        }
    }

    fn emit_operand(&mut self, operand: &Operand) -> io::Result<()> {
        match operand {
            Operand::Immediate(int) => write!(self.output, "{int}"),
            Operand::Register(register) => self.emit_register(register),
            Operand::Pseudo(pseudo) => write!(self.output, "<{pseudo}>"),
            Operand::Stack { offset } => {
                let sign = if offset.is_negative() { "-" } else { "+" };
                let abs = offset.abs();
                write!(self.output, "[rbp {sign} {abs}]")
            }
        }
    }

    fn emit_register(&mut self, register: &Register) -> io::Result<()> {
        match register {
            Register::RSP => write!(self.output, "rsp"),
            Register::RBP => write!(self.output, "rbp"),
            Register::EAX => write!(self.output, "eax"),
            Register::EDX => write!(self.output, "edx"),
            Register::R10D => write!(self.output, "r10d"),
            Register::R11D => write!(self.output, "r11d"),
        }
    }

    fn newline(&mut self) -> io::Result<()> {
        writeln!(self.output)
    }
}
