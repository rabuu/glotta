use std::collections::HashMap;

use crate::{asm, tacky};

/// An Int takes four bytes in memory.
/// For now, every variable is an Int.
const INT_BYTES: usize = 4;

pub fn codegen_program(program: &tacky::Program) -> asm::Program {
    let tacky::Program { function } = program;
    let function = codegen_function_definition(function);
    asm::Program { function }
}

fn codegen_function_definition(function: &tacky::FunctionDefinition) -> asm::FunctionDefinition {
    let tacky::FunctionDefinition { name, body } = function;

    let name = name.to_string();

    let prologue = vec![
        asm::Instruction::Push(asm::Operand::Register(asm::Register::RBP)),
        asm::Instruction::Mov {
            src: asm::Operand::Register(asm::Register::RSP),
            dst: asm::Operand::Register(asm::Register::RBP),
        },
    ];

    let mut body_instructions = Vec::new();
    for instruction in body {
        codegen_instruction(instruction, &mut body_instructions);
    }

    let required_stack_space = replace_pseudo_operands(&mut body_instructions);
    rewrite_invalid_movs(&mut body_instructions);

    let mut instructions = prologue;

    // allocate stack space
    instructions.push(asm::Instruction::Sub {
        src: asm::Operand::Immediate(required_stack_space as isize),
        dst: asm::Operand::Register(asm::Register::RSP),
    });

    instructions.append(&mut body_instructions);

    asm::FunctionDefinition { name, instructions }
}

fn codegen_instruction(instruction: &tacky::Instruction, instructions: &mut Vec<asm::Instruction>) {
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
        tacky::Instruction::Binary(_) => todo!(),
    }
}

fn codegen_value(value: &tacky::Value) -> asm::Operand {
    match value {
        tacky::Value::Constant(constant) => asm::Operand::Immediate(*constant as isize),
        tacky::Value::Variable(identifier) => asm::Operand::Pseudo(identifier.to_string()),
    }
}

fn replace_pseudo_operands(instructions: &mut Vec<asm::Instruction>) -> usize {
    let mut current_offset: usize = 0;
    let mut offset_map: HashMap<String, usize> = HashMap::new();

    for instruction in instructions {
        match instruction {
            asm::Instruction::Mov { src, dst } | asm::Instruction::Sub { src, dst } => {
                replace_pseudo_operand(src, &mut current_offset, &mut offset_map);
                replace_pseudo_operand(dst, &mut current_offset, &mut offset_map);
            }
            asm::Instruction::Not(operand)
            | asm::Instruction::Neg(operand)
            | asm::Instruction::Push(operand)
            | asm::Instruction::Pop(operand) => {
                replace_pseudo_operand(operand, &mut current_offset, &mut offset_map);
            }
            asm::Instruction::Ret => (),
        }
    }

    current_offset
}

fn replace_pseudo_operand(
    operand: &mut asm::Operand,
    current_offset: &mut usize,
    offset_map: &mut HashMap<String, usize>,
) {
    if let asm::Operand::Pseudo(pseudo) = operand {
        let offset = match offset_map.get(pseudo) {
            Some(offset) => *offset,
            None => {
                *current_offset += INT_BYTES;
                offset_map.insert(pseudo.clone(), *current_offset);
                *current_offset
            }
        };

        let offset = -(offset as isize);
        *operand = asm::Operand::Stack { offset };
    }
}

fn rewrite_invalid_movs(instructions: &mut Vec<asm::Instruction>) {
    // NOTE: this is very inefficient, shifting all Vec elements for every insert

    for i in 0..instructions.len() {
        if let asm::Instruction::Mov {
            src: asm::Operand::Stack { offset: src_offset },
            dst: asm::Operand::Stack { offset: dst_offset },
        } = instructions[i]
        {
            instructions[i] = asm::Instruction::Mov {
                src: asm::Operand::Stack { offset: src_offset },
                dst: asm::Operand::Register(asm::Register::TEMP),
            };

            instructions.insert(
                i + 1,
                asm::Instruction::Mov {
                    src: asm::Operand::Register(asm::Register::TEMP),
                    dst: asm::Operand::Stack { offset: dst_offset },
                },
            );
        }
    }
}
