use crate::ast;

pub mod asm;
pub mod emitter;

pub fn codegen_program(program: &ast::Program) -> asm::Program {
    let ast::Program { function } = program;
    let function = codegen_function_definition(function);
    asm::Program { function }
}

pub fn codegen_function_definition(function: &ast::FunctionDefinition) -> asm::FunctionDefinition {
    let ast::FunctionDefinition {
        name,
        body,
        span: _,
    } = function;

    let mut instructions = Vec::new();
    let body = codegen_expression(body);
    instructions.push(asm::Instruction::Mov {
        src: body,
        dst: asm::Operand::Register,
    });
    instructions.push(asm::Instruction::Ret);

    asm::FunctionDefinition {
        name: name.identifier.clone(),
        instructions,
    }
}

pub fn codegen_expression(expression: &ast::Expression) -> asm::Operand {
    let ast::Expression { kind, span: _ } = expression;
    match kind {
        ast::ExpressionKind::Constant(literal) => asm::Operand::Immediate(literal.value),
    }
}
