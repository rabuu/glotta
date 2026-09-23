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
        asm::Instruction::Push(asm::Register::RBP.into()),
        asm::Instruction::Mov {
            src: asm::Register::RSP.into(),
            dst: asm::Register::RBP.into(),
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
        dst: asm::Register::RSP.into(),
    });

    instructions.append(&mut body_instructions);

    asm::FunctionDefinition { name, instructions }
}

fn codegen_instruction(instruction: &tacky::Instruction, instructions: &mut Vec<asm::Instruction>) {
    match instruction {
        tacky::Instruction::Return(value) => {
            instructions.push(asm::Instruction::Mov {
                src: codegen_value(value),
                dst: asm::Register::EAX.into(),
            });

            // function epilogue
            instructions.push(asm::Instruction::Mov {
                src: asm::Register::RBP.into(),
                dst: asm::Register::RSP.into(),
            });
            instructions.push(asm::Instruction::Pop(asm::Register::RBP.into()));

            instructions.push(asm::Instruction::Ret);
        }
        tacky::Instruction::Unary(tacky::Unary {
            op: tacky::UnaryOperator::Not,
            src,
            dst,
        }) => {
            let src = codegen_value(src);
            let dst = codegen_value(dst);

            instructions.extend([
                asm::Instruction::Cmp(asm::Operand::Immediate(0), src),
                asm::Instruction::Mov {
                    src: asm::Operand::Immediate(0),
                    dst: dst.clone(),
                },
                asm::Instruction::SetCC {
                    flag: asm::ConditionalFlag::E,
                    op: dst,
                },
            ])
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
                tacky::UnaryOperator::Not => unreachable!(),
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
                        dst: asm::Register::EAX.into(),
                    },
                    asm::Instruction::Cdq,
                    asm::Instruction::IDiv(rhs),
                    asm::Instruction::Mov {
                        src: asm::Register::EAX.into(),
                        dst,
                    },
                ]),
                tacky::BinaryOperator::Remainder => instructions.extend([
                    asm::Instruction::Mov {
                        src: lhs,
                        dst: asm::Register::EAX.into(),
                    },
                    asm::Instruction::Cdq,
                    asm::Instruction::IDiv(rhs),
                    asm::Instruction::Mov {
                        src: asm::Register::EDX.into(),
                        dst,
                    },
                ]),
                tacky::BinaryOperator::Equal
                | tacky::BinaryOperator::NotEqual
                | tacky::BinaryOperator::LessThan
                | tacky::BinaryOperator::LessOrEqual
                | tacky::BinaryOperator::GreaterThan
                | tacky::BinaryOperator::GreaterOrEqual => instructions.extend([
                    asm::Instruction::Cmp(rhs, lhs),
                    asm::Instruction::Mov {
                        src: asm::Operand::Immediate(0),
                        dst: dst.clone(),
                    },
                    asm::Instruction::SetCC {
                        flag: binary_operator_to_conditional_flag(op)
                            .expect("conditional binary operators have corresponding flags"),
                        op: dst,
                    },
                ]),
            }
        }
        tacky::Instruction::Copy { src, dst } => {
            let src = codegen_value(src);
            let dst = codegen_value(dst);
            instructions.push(asm::Instruction::Mov { src, dst })
        }
        tacky::Instruction::Jump(target) => {
            instructions.push(asm::Instruction::Jmp(target.to_string()));
        }
        tacky::Instruction::JumpIfZero { condition, target } => {
            let condition = codegen_value(condition);
            instructions.extend([
                asm::Instruction::Cmp(asm::Operand::Immediate(0), condition),
                asm::Instruction::JmpCC {
                    flag: asm::ConditionalFlag::E,
                    label: target.to_string(),
                },
            ])
        }
        tacky::Instruction::JumpIfNotZero { condition, target } => {
            let condition = codegen_value(condition);
            instructions.extend([
                asm::Instruction::Cmp(asm::Operand::Immediate(0), condition),
                asm::Instruction::JmpCC {
                    flag: asm::ConditionalFlag::NE,
                    label: target.to_string(),
                },
            ])
        }
        tacky::Instruction::Label(label) => {
            instructions.push(asm::Instruction::Label(label.to_string()));
        }
    }
}

fn codegen_value(value: &tacky::Value) -> asm::Operand {
    match value {
        tacky::Value::Constant(constant) => asm::Operand::Immediate(*constant as isize),
        tacky::Value::Variable(identifier) => asm::Operand::Pseudo(identifier.to_string()),
    }
}

fn binary_operator_to_conditional_flag(op: &tacky::BinaryOperator) -> Option<asm::ConditionalFlag> {
    use asm::ConditionalFlag::*;
    match op {
        tacky::BinaryOperator::Equal => Some(E),
        tacky::BinaryOperator::NotEqual => Some(NE),
        tacky::BinaryOperator::LessThan => Some(L),
        tacky::BinaryOperator::LessOrEqual => Some(LE),
        tacky::BinaryOperator::GreaterThan => Some(G),
        tacky::BinaryOperator::GreaterOrEqual => Some(GE),
        _ => None,
    }
}

fn replace_pseudo_operands(instructions: &mut Vec<asm::Instruction>) -> usize {
    let mut current_offset: usize = 0;
    let mut offset_map: HashMap<String, usize> = HashMap::new();

    for instruction in instructions {
        match instruction {
            asm::Instruction::Mov { src: one, dst: two }
            | asm::Instruction::Add { src: one, dst: two }
            | asm::Instruction::Sub { src: one, dst: two }
            | asm::Instruction::IMul { src: one, dst: two }
            | asm::Instruction::Cmp(one, two) => {
                replace_pseudo_operand(one, &mut current_offset, &mut offset_map);
                replace_pseudo_operand(two, &mut current_offset, &mut offset_map);
            }
            asm::Instruction::Not(op)
            | asm::Instruction::Neg(op)
            | asm::Instruction::IDiv(op)
            | asm::Instruction::Push(op)
            | asm::Instruction::Pop(op)
            | asm::Instruction::SetCC { flag: _, op } => {
                replace_pseudo_operand(op, &mut current_offset, &mut offset_map);
            }
            asm::Instruction::Ret
            | asm::Instruction::Cdq
            | asm::Instruction::Jmp(_)
            | asm::Instruction::JmpCC { flag: _, label: _ }
            | asm::Instruction::Label(_) => (),
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
                    dst: asm::Register::TEMP_SRC.into(),
                };

                instructions.insert(
                    i + 1,
                    asm::Instruction::Mov {
                        src: asm::Register::TEMP_SRC.into(),
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
                    dst: asm::Register::TEMP_SRC.into(),
                };

                instructions.insert(
                    i + 1,
                    asm::Instruction::Add {
                        src: asm::Register::TEMP_SRC.into(),
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
                    dst: asm::Register::TEMP_SRC.into(),
                };

                instructions.insert(
                    i + 1,
                    asm::Instruction::Sub {
                        src: asm::Register::TEMP_SRC.into(),
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
                    dst: asm::Register::TEMP_DST.into(),
                };

                instructions.insert(
                    i + 1,
                    asm::Instruction::IMul {
                        src,
                        dst: asm::Register::TEMP_DST.into(),
                    },
                );

                instructions.insert(
                    i + 2,
                    asm::Instruction::Mov {
                        src: asm::Register::TEMP_DST.into(),
                        dst: asm::Operand::Stack { offset: dst_offset },
                    },
                );
            }

            // rewrite idiv instructions with constant operand
            asm::Instruction::IDiv(asm::Operand::Immediate(imm)) => {
                instructions[i] = asm::Instruction::Mov {
                    src: asm::Operand::Immediate(imm),
                    dst: asm::Register::TEMP_SRC.into(),
                };

                instructions.insert(
                    i + 1,
                    asm::Instruction::IDiv(asm::Register::TEMP_SRC.into()),
                );
            }

            _ => (),
        }
    }
}
