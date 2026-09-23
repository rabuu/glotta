use std::io;

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
    out: O,
}

impl<O: io::Write> Emitter<O> {
    const INDENT: &str = "    ";

    pub fn new(out: O) -> Self {
        Self { out }
    }

    pub fn emit_program(mut self, program: &Program) -> io::Result<()> {
        let Program { function } = program;

        writeln!(self.out, "section .text\n")?;
        self.emit_function_definition(function)?;

        self.newline()?;
        writeln!(
            self.out,
            "section .note.GNU-stack noalloc noexec nowrite progbits"
        )?;

        self.out.flush()?;

        Ok(())
    }

    fn emit_function_definition(&mut self, function: &FunctionDefinition) -> io::Result<()> {
        let FunctionDefinition { name, instructions } = function;

        writeln!(self.out, "global {name}")?;
        writeln!(self.out, "{name}:")?;
        for instruction in instructions {
            self.indent()?;
            self.emit_instruction(instruction)?;
            self.newline()?;
        }

        Ok(())
    }

    fn emit_instruction(&mut self, instruction: &Instruction) -> io::Result<()> {
        match instruction {
            Instruction::Mov { src, dst } => {
                write!(self.out, "mov ")?;
                self.emit_operand(dst)?;
                write!(self.out, ", ")?;
                self.emit_operand(src)?;
                Ok(())
            }
            Instruction::Sub { src, dst } => {
                write!(self.out, "sub ")?;
                self.emit_operand(dst)?;
                write!(self.out, ", ")?;
                self.emit_operand(src)?;
                Ok(())
            }
            Instruction::Add { src, dst } => {
                write!(self.out, "add ")?;
                self.emit_operand(dst)?;
                write!(self.out, ", ")?;
                self.emit_operand(src)?;
                Ok(())
            }
            Instruction::IMul { src, dst } => {
                write!(self.out, "imul ")?;
                self.emit_operand(dst)?;
                write!(self.out, ", ")?;
                self.emit_operand(src)?;
                Ok(())
            }
            Instruction::Cmp { src, dst } => {
                write!(self.out, "cmp ")?;
                self.emit_operand(dst)?;
                write!(self.out, ", ")?;
                self.emit_operand(src)?;
                Ok(())
            }
            Instruction::Not(operand) => {
                write!(self.out, "not ")?;
                self.emit_operand(operand)?;
                Ok(())
            }
            Instruction::Neg(operand) => {
                write!(self.out, "neg ")?;
                self.emit_operand(operand)?;
                Ok(())
            }
            Instruction::IDiv(operand) => {
                write!(self.out, "idiv ")?;
                self.emit_operand(operand)?;
                Ok(())
            }
            Instruction::Cdq => write!(self.out, "cdq"),
            Instruction::Push(operand) => {
                write!(self.out, "push ")?;
                self.emit_operand(operand)?;
                Ok(())
            }
            Instruction::Pop(operand) => {
                write!(self.out, "pop ")?;
                self.emit_operand(operand)?;
                Ok(())
            }
            Instruction::Jmp(label) => write!(self.out, "jmp {label}"),
            Instruction::JmpCC { flag, label } => {
                write!(self.out, "j")?;
                self.emit_conditional_flag(flag)?;
                write!(self.out, " {label}")?;
                Ok(())
            }
            Instruction::SetCC { flag, op } => {
                write!(self.out, "set")?;
                self.emit_conditional_flag(flag)?;
                write!(self.out, " ")?;
                self.emit_operand(op)?;
                Ok(())
            }
            Instruction::Label(label) => write!(self.out, "{label}:"),
            Instruction::Ret => write!(self.out, "ret"),
        }
    }

    fn emit_operand(&mut self, operand: &Operand) -> io::Result<()> {
        match operand {
            Operand::Immediate(int) => write!(self.out, "{int}"),
            Operand::Register(register) => self.emit_register(register),
            Operand::Pseudo(pseudo) => write!(self.out, "<{pseudo}>"),
            Operand::Stack { offset } => {
                let sign = if offset.is_negative() { "-" } else { "+" };
                let abs = offset.abs();
                write!(self.out, "[rbp {sign} {abs}]")
            }
        }
    }

    fn emit_register(&mut self, register: &Register) -> io::Result<()> {
        match register {
            Register::RSP => write!(self.out, "rsp"),
            Register::RBP => write!(self.out, "rbp"),
            Register::EAX => write!(self.out, "eax"),
            Register::EDX => write!(self.out, "edx"),
            Register::R10D => write!(self.out, "r10d"),
            Register::R11D => write!(self.out, "r11d"),
        }
    }

    fn emit_conditional_flag(&mut self, flag: &ConditionalFlag) -> io::Result<()> {
        match flag {
            ConditionalFlag::E => write!(self.out, "e"),
            ConditionalFlag::NE => write!(self.out, "ne"),
            ConditionalFlag::G => write!(self.out, "g"),
            ConditionalFlag::GE => write!(self.out, "ge"),
            ConditionalFlag::L => write!(self.out, "l"),
            ConditionalFlag::LE => write!(self.out, "le"),
        }
    }

    fn newline(&mut self) -> io::Result<()> {
        writeln!(self.out)
    }

    fn indent(&mut self) -> io::Result<()> {
        write!(self.out, "{}", Self::INDENT)
    }
}
