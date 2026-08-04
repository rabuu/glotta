use std::fmt;

use crate::codegen::asm;

const INDENT: &str = "    ";

impl fmt::Display for asm::Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let asm::Program { function } = self;

        writeln!(f, "{function}")?;
        writeln!(f, "{INDENT}.section .not.GNU-stack,\"\",@progbits")?;

        Ok(())
    }
}

impl fmt::Display for asm::FunctionDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let asm::FunctionDefinition { name, instructions } = self;

        writeln!(f, "{INDENT}.globl {name}")?;
        writeln!(f, "{name}:")?;
        for instruction in instructions {
            writeln!(f, "{INDENT}{instruction}")?;
        }

        Ok(())
    }
}

impl fmt::Display for asm::Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            asm::Instruction::Mov { src, dst } => write!(f, "movl {src}, {dst}"),
            asm::Instruction::Ret => write!(f, "ret"),
        }
    }
}

impl fmt::Display for asm::Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            asm::Operand::Immediate(int) => write!(f, "${int}"),
            asm::Operand::Register => write!(f, "%eax"),
        }
    }
}
