use crate::tacky;

pub mod asm;
pub mod emitter;

pub fn codegen_program(program: &tacky::Program) -> asm::Program {
    let tacky::Program { function } = program;
    let function = codegen_function_definition(function);
    asm::Program { function }
}

pub fn codegen_function_definition(
    function: &tacky::FunctionDefinition,
) -> asm::FunctionDefinition {
    let tacky::FunctionDefinition { name, body } = function;

    let name = name.to_string();
    let mut instructions = Vec::new();

    // function prologue
    instructions.push(asm::Instruction::Push(asm::Operand::Register(
        asm::Register::RBP,
    )));
    instructions.push(asm::Instruction::Mov {
        src: asm::Operand::Register(asm::Register::RSP),
        dst: asm::Operand::Register(asm::Register::RBP),
    });

    // function body
    for instruction in body {
        codegen_instruction(instruction, &mut instructions);
    }

    asm::FunctionDefinition { name, instructions }
}

pub fn codegen_instruction(
    instruction: &tacky::Instruction,
    instructions: &mut Vec<asm::Instruction>,
) {
    match instruction {
        tacky::Instruction::Return(value) => {
            instructions.push(asm::Instruction::Mov {
                src: codegen_value(value),
                dst: asm::Operand::Register(asm::Register::EAX),
            });

            // function epilogue
            instructions.push(asm::Instruction::Mov {
                src: asm::Operand::Register(asm::Register::RBP),
                dst: asm::Operand::Register(asm::Register::RSP),
            });
            instructions.push(asm::Instruction::Pop(asm::Operand::Register(
                asm::Register::RBP,
            )));

            instructions.push(asm::Instruction::Ret);
        }
        tacky::Instruction::Unary(tacky::Unary { op, src, dst }) => {
            let src = codegen_value(src);
            let dst = codegen_value(dst);

            instructions.push(asm::Instruction::Mov {
                src,
                dst: dst.clone(),
            });
            match op {
                tacky::UnaryOperator::BitwiseNot => instructions.push(asm::Instruction::Not(dst)),
                tacky::UnaryOperator::Negation => instructions.push(asm::Instruction::Neg(dst)),
            }
        }
    }
}

pub fn codegen_value(value: &tacky::Value) -> asm::Operand {
    match value {
        tacky::Value::Constant(constant) => asm::Operand::Immediate(*constant as isize),
        tacky::Value::Variable(identifier) => asm::Operand::Pseudo(identifier.to_string()),
    }
}
