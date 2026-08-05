use crate::ast;

pub mod asm;
pub mod emitter;

pub fn codegen_program(program: &ast::Program) -> Result<asm::Program, ()> {
    let ast::Program { function } = program;
    let function = codegen_function_definition(function)?;
    Ok(asm::Program { function })
}

pub fn codegen_function_definition(
    function: &ast::FunctionDefinition,
) -> Result<asm::FunctionDefinition, ()> {
    let ast::FunctionDefinition {
        name,
        body,
        span: _,
    } = function;

    let mut instructions = Vec::new();
    let body = codegen_expression(body)?;
    instructions.push(asm::Instruction::Mov {
        src: body,
        dst: asm::Operand::Register,
    });
    instructions.push(asm::Instruction::Ret);

    Ok(asm::FunctionDefinition {
        name: name.identifier.clone(),
        instructions,
    })
}

pub fn codegen_expression(expression: &ast::Expression) -> Result<asm::Operand, ()> {
    let ast::Expression { kind, span: _ } = expression;
    match kind {
        ast::ExpressionKind::Constant(literal) => Ok(asm::Operand::Immediate(literal.value)),
    }
}
