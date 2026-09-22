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
        let ast::BuiltinCall {
            operator,
            arguments,
            span: _,
        } = call;

        match operator.kind {
            ast::BuiltinOperatorKind::BitwiseNot | ast::BuiltinOperatorKind::Negation => {
                assert_eq!(arguments.arity(), 1, "Known by type checking");
                let arg = &arguments.inner[0];

                let src = self.lower_expression(arg, instructions);
                let dst = self.fresh_variable();
                let op = self
                    .lower_unary_operator(operator)
                    .expect("pattern matching ensures this is a unary operator");

                instructions.push(tacky::Instruction::Unary(tacky::Unary {
                    op,
                    src,
                    dst: dst.clone(),
                }));

                dst
            }
            ast::BuiltinOperatorKind::Addition
            | ast::BuiltinOperatorKind::Multiplication
            | ast::BuiltinOperatorKind::Subtraction
            | ast::BuiltinOperatorKind::Division
            | ast::BuiltinOperatorKind::Remainder => {
                assert_eq!(arguments.arity(), 2, "Known by type checking");
                let lhs = &arguments.inner[0];
                let rhs = &arguments.inner[1];

                let lhs = self.lower_expression(lhs, instructions);
                let rhs = self.lower_expression(rhs, instructions);
                let dst = self.fresh_variable();
                let op = self
                    .lower_binary_operator(operator)
                    .expect("pattern matching ensures this is a binary operator");

                instructions.push(tacky::Instruction::Binary(tacky::Binary {
                    op,
                    lhs,
                    rhs,
                    dst: dst.clone(),
                }));

                dst
            }
        }
    }

    fn lower_unary_operator(
        &self,
        operator: &ast::BuiltinOperator,
    ) -> Option<tacky::UnaryOperator> {
        match operator.kind {
            ast::BuiltinOperatorKind::BitwiseNot => Some(tacky::UnaryOperator::BitwiseNot),
            ast::BuiltinOperatorKind::Negation => Some(tacky::UnaryOperator::Negation),
            _ => None,
        }
    }

    fn lower_binary_operator(
        &self,
        operator: &ast::BuiltinOperator,
    ) -> Option<tacky::BinaryOperator> {
        match operator.kind {
            ast::BuiltinOperatorKind::Addition => Some(tacky::BinaryOperator::Addition),
            ast::BuiltinOperatorKind::Multiplication => Some(tacky::BinaryOperator::Multiplication),
            ast::BuiltinOperatorKind::Subtraction => Some(tacky::BinaryOperator::Subtraction),
            ast::BuiltinOperatorKind::Division => Some(tacky::BinaryOperator::Division),
            ast::BuiltinOperatorKind::Remainder => Some(tacky::BinaryOperator::Remainder),
            _ => None,
        }
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
