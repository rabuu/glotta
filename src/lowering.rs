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
        let ast::Expression { kind, span: _ } = expression;

        match kind {
            ast::ExpressionKind::Constant(literal) => self.lower_constant(literal, instructions),
            ast::ExpressionKind::Call(ast::CallKind::Builtin(call)) => {
                self.lower_builtin_call(call, instructions)
            }
            ast::ExpressionKind::Call(ast::CallKind::Function(_)) => todo!(),
        }
    }

    fn lower_constant(
        &self,
        literal: &ast::IntegerLiteral,
        _instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        let ast::IntegerLiteral { value, span: _ } = literal;
        tacky::Value::Constant(*value)
    }

    fn lower_builtin_call(
        &mut self,
        call: &ast::BuiltinCall,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        let ast::BuiltinCall { kind, span: _ } = call;
        match kind {
            ast::BuiltinCallKind::Unary(ast::UnaryOperation {
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
        match operator {
            ast::UnaryOperator::BitwiseNot => tacky::UnaryOperator::BitwiseNot,
            ast::UnaryOperator::Negation => tacky::UnaryOperator::Negation,
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
