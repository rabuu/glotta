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
            asm::Instruction::Ret => write!(self.output, "ret"),
        }
    }

    fn emit_operand(&mut self, operand: &asm::Operand) -> io::Result<()> {
        match operand {
            asm::Operand::Immediate(int) => write!(self.output, "{int}"),
            asm::Operand::Register => write!(self.output, "eax"),
        }
    }

    fn newline(&mut self) -> io::Result<()> {
        writeln!(self.output)
    }
}
