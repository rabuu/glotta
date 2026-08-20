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
    rewrite_invalid_instructions(&mut body_instructions);

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
        tacky::Instruction::Binary(tacky::Binary { op, lhs, rhs, dst }) => {
            let lhs = codegen_value(lhs);
            let rhs = codegen_value(rhs);
            let dst = codegen_value(dst);

            match op {
                tacky::BinaryOperator::Addition => instructions.extend([
                    asm::Instruction::Mov {
                        src: lhs,
                        dst: dst.clone(),
                    },
                    asm::Instruction::Add { src: rhs, dst },
                ]),
                tacky::BinaryOperator::Multiplication => instructions.extend([
                    asm::Instruction::Mov {
                        src: lhs,
                        dst: dst.clone(),
                    },
                    asm::Instruction::IMul { src: rhs, dst },
                ]),
                tacky::BinaryOperator::Subtraction => instructions.extend([
                    asm::Instruction::Mov {
                        src: lhs,
                        dst: dst.clone(),
                    },
                    asm::Instruction::Sub { src: rhs, dst },
                ]),
                tacky::BinaryOperator::Division => instructions.extend([
                    asm::Instruction::Mov {
                        src: lhs,
                        dst: asm::Operand::Register(asm::Register::EAX),
                    },
                    asm::Instruction::Cdq,
                    asm::Instruction::IDiv(rhs),
                    asm::Instruction::Mov {
                        src: asm::Operand::Register(asm::Register::EAX),
                        dst,
                    },
                ]),
            }
        }
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
            asm::Instruction::Mov { src, dst }
            | asm::Instruction::Add { src, dst }
            | asm::Instruction::Sub { src, dst }
            | asm::Instruction::IMul { src, dst } => {
                replace_pseudo_operand(src, &mut current_offset, &mut offset_map);
                replace_pseudo_operand(dst, &mut current_offset, &mut offset_map);
            }
            asm::Instruction::Not(operand)
            | asm::Instruction::Neg(operand)
            | asm::Instruction::IDiv(operand)
            | asm::Instruction::Push(operand)
            | asm::Instruction::Pop(operand) => {
                replace_pseudo_operand(operand, &mut current_offset, &mut offset_map);
            }
            asm::Instruction::Ret | asm::Instruction::Cdq => (),
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

fn rewrite_invalid_instructions(instructions: &mut Vec<asm::Instruction>) {
    // NOTE: this is very inefficient, shifting all Vec elements for every insert

    for i in 0..instructions.len() {
        match instructions[i] {
            // rewrite mov instructions with both operands in stack position
            asm::Instruction::Mov {
                src: asm::Operand::Stack { offset: src_offset },
                dst: asm::Operand::Stack { offset: dst_offset },
            } => {
                instructions[i] = asm::Instruction::Mov {
                    src: asm::Operand::Stack { offset: src_offset },
                    dst: asm::Operand::Register(asm::Register::TEMP_SRC),
                };

                instructions.insert(
                    i + 1,
                    asm::Instruction::Mov {
                        src: asm::Operand::Register(asm::Register::TEMP_SRC),
                        dst: asm::Operand::Stack { offset: dst_offset },
                    },
                );
            }

            // rewrite add instructions with both operands in stack position
            asm::Instruction::Add {
                src: asm::Operand::Stack { offset: src_offset },
                dst: asm::Operand::Stack { offset: dst_offset },
            } => {
                instructions[i] = asm::Instruction::Mov {
                    src: asm::Operand::Stack { offset: src_offset },
                    dst: asm::Operand::Register(asm::Register::TEMP_SRC),
                };

                instructions.insert(
                    i + 1,
                    asm::Instruction::Add {
                        src: asm::Operand::Register(asm::Register::TEMP_SRC),
                        dst: asm::Operand::Stack { offset: dst_offset },
                    },
                );
            }

            // rewrite sub instructions with both operands in stack position
            asm::Instruction::Sub {
                src: asm::Operand::Stack { offset: src_offset },
                dst: asm::Operand::Stack { offset: dst_offset },
            } => {
                instructions[i] = asm::Instruction::Mov {
                    src: asm::Operand::Stack { offset: src_offset },
                    dst: asm::Operand::Register(asm::Register::TEMP_SRC),
                };

                instructions.insert(
                    i + 1,
                    asm::Instruction::Sub {
                        src: asm::Operand::Register(asm::Register::TEMP_SRC),
                        dst: asm::Operand::Stack { offset: dst_offset },
                    },
                );
            }

            // rewrite imul instructions with dst operand in stack position
            asm::Instruction::IMul {
                ref src,
                dst: asm::Operand::Stack { offset: dst_offset },
            } => {
                let src = src.clone();

                instructions[i] = asm::Instruction::Mov {
                    src: asm::Operand::Stack { offset: dst_offset },
                    dst: asm::Operand::Register(asm::Register::TEMP_DST),
                };

                instructions.insert(
                    i + 1,
                    asm::Instruction::IMul {
                        src,
                        dst: asm::Operand::Register(asm::Register::TEMP_DST),
                    },
                );

                instructions.insert(
                    i + 2,
                    asm::Instruction::Mov {
                        src: asm::Operand::Register(asm::Register::TEMP_DST),
                        dst: asm::Operand::Stack { offset: dst_offset },
                    },
                );
            }

            // rewrite idiv instructions with constant operand
            asm::Instruction::IDiv(asm::Operand::Immediate(imm)) => {
                instructions[i] = asm::Instruction::Mov {
                    src: asm::Operand::Immediate(imm),
                    dst: asm::Operand::Register(asm::Register::TEMP_SRC),
                };

                instructions.insert(
                    i + 1,
                    asm::Instruction::IDiv(asm::Operand::Register(asm::Register::TEMP_SRC)),
                );
            }

            _ => (),
        }
    }
}
