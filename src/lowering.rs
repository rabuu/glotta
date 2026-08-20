use crate::{ast, tacky};

pub struct Lowerer {
    fresh: usize,
}

impl Lowerer {
    pub fn new() -> Self {
        Self { fresh: 0 }
    }

    pub fn lower_program(&mut self, program: &ast::Program) -> tacky::Program {
        let ast::Program { function } = program;
        let function = self.lower_function_definition(function);
        tacky::Program { function }
    }

    fn lower_function_definition(
        &mut self,
        function: &ast::FunctionDefinition,
    ) -> tacky::FunctionDefinition {
        let ast::FunctionDefinition {
            name,
            body,
            span: _,
        } = function;

        let name = tacky::Identifier::Named(name.identifier.clone());

        let mut instructions = Vec::new();
        let body = self.lower_expression(body, &mut instructions);
        instructions.push(tacky::Instruction::Return(body));

        tacky::FunctionDefinition {
            name,
            body: instructions,
        }
    }

    fn lower_expression(
        &mut self,
        expression: &ast::Expression,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        match expression {
            ast::Expression::Constant(constant) => self.lower_constant(constant, instructions),
            ast::Expression::Call(ast::Call::Builtin(call)) => {
                self.lower_builtin_call(call, instructions)
            }
            ast::Expression::Call(ast::Call::Function(_)) => todo!(),
        }
    }

    fn lower_constant(
        &self,
        constant: &ast::IntegerConstant,
        _instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        let ast::IntegerConstant { value, span: _ } = constant;
        tacky::Value::Constant(*value)
    }

    fn lower_builtin_call(
        &mut self,
        call: &ast::BuiltinCall,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        todo!()
    }

    fn fresh_identifier(&mut self) -> tacky::Identifier {
        let ident = tacky::Identifier::Temporary(self.fresh);
        self.fresh += 1;
        ident
    }

    fn fresh_variable(&mut self) -> tacky::Value {
        let ident = self.fresh_identifier();
        tacky::Value::Variable(ident)
    }
}
