use std::io;

use crate::codegen::asm;

const INDENT: &str = "    ";

pub struct Emitter<O: io::Write> {
    output: O,
}

impl<O: io::Write> Emitter<O> {
    pub fn new(output: O) -> Self {
        Self { output }
    }

    pub fn emit_program(mut self, program: &asm::Program) -> io::Result<()> {
        let asm::Program { function } = program;

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

    fn emit_function_definition(&mut self, function: &asm::FunctionDefinition) -> io::Result<()> {
        let asm::FunctionDefinition { name, instructions } = function;

        writeln!(self.output, "global {name}")?;
        writeln!(self.output, "{name}:")?;
        for instruction in instructions {
            write!(self.output, "{INDENT}")?;
            self.emit_instruction(instruction)?;
            self.newline()?;
        }

        Ok(())
    }

    fn emit_instruction(&mut self, instruction: &asm::Instruction) -> io::Result<()> {
        match instruction {
            asm::Instruction::Mov { src, dst } => {
                write!(self.output, "mov ")?;
                self.emit_operand(dst)?;
                write!(self.output, ", ")?;
                self.emit_operand(src)?;
                Ok(())
            }
            asm::Instruction::Sub { src, dst } => {
                write!(self.output, "sub ")?;
                self.emit_operand(dst)?;
                write!(self.output, ", ")?;
                self.emit_operand(src)?;
                Ok(())
            }
            asm::Instruction::Not(operand) => {
                write!(self.output, "not ")?;
                self.emit_operand(operand)?;
                Ok(())
            }
            asm::Instruction::Neg(operand) => {
                write!(self.output, "neg ")?;
                self.emit_operand(operand)?;
                Ok(())
            }
            asm::Instruction::Push(operand) => {
                write!(self.output, "push ")?;
                self.emit_operand(operand)?;
                Ok(())
            }
            asm::Instruction::Pop(operand) => {
                write!(self.output, "pop ")?;
                self.emit_operand(operand)?;
                Ok(())
            }
            asm::Instruction::Ret => write!(self.output, "ret"),
        }
    }

    fn emit_operand(&mut self, operand: &asm::Operand) -> io::Result<()> {
        match operand {
            asm::Operand::Immediate(int) => write!(self.output, "{int}"),
            asm::Operand::Register(register) => self.emit_register(register),
            asm::Operand::Pseudo(pseudo) => write!(self.output, "<{pseudo}>"),
            asm::Operand::Stack { offset } => {
                let sign = if offset.is_negative() { "-" } else { "+" };
                let abs = offset.abs();
                write!(self.output, "[rbp {sign} {abs}]")
            }
        }
    }

    fn emit_register(&mut self, register: &asm::Register) -> io::Result<()> {
        match register {
            asm::Register::RSP => write!(self.output, "rsp"),
            asm::Register::RBP => write!(self.output, "rbp"),
            asm::Register::EAX => write!(self.output, "eax"),
            asm::Register::R10D => write!(self.output, "r10d"),
        }
    }

    fn newline(&mut self) -> io::Result<()> {
        writeln!(self.output)
    }
}
