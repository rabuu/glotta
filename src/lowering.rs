use crate::{ast, tacky};

pub struct Lowerer {
    fresh: usize,
}

impl Lowerer {
    pub fn new() -> Self {
        Self { fresh: 0 }
    }

    pub fn lower_expression(
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
        match call {
            ast::BuiltinCall::Unary(ast::UnaryOperation {
                operator,
                arg,
                span: _,
            }) => {
                let src = self.lower_expression(arg, instructions);
                let dst = self.fresh_variable();
                let op = self.lower_unary_operator(operator);

                let instruction = tacky::Instruction::Unary(tacky::Unary {
                    op,
                    src,
                    dst: dst.clone(),
                });
                instructions.push(instruction);

                return dst;
            }
        }
    }

    fn lower_unary_operator(&self, operator: &ast::UnaryOperator) -> tacky::UnaryOperator {
        match operator.kind {
            ast::UnaryOperatorKind::BitwiseNot => tacky::UnaryOperator::BitwiseNot,
            ast::UnaryOperatorKind::Negation => tacky::UnaryOperator::Negation,
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
