use crate::{ast, tacky};

#[derive(Default)]
pub struct Lowerer {
    fresh: usize,
}

impl Lowerer {
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

        let name = tacky::Identifier::Function(name.identifier.clone());

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
            ast::Expression::Variable(variable) => todo!(),
            ast::Expression::Declaration(declaration) => todo!(),
            ast::Expression::Assignment(assignment) => todo!(),
            ast::Expression::Call(ast::Call::Builtin(call)) => {
                self.lower_builtin_call(call, instructions)
            }
            ast::Expression::Call(ast::Call::Function(_)) => todo!(),
            ast::Expression::Block(block) => self.lower_block(block, instructions),
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
            ast::BuiltinOperatorKind::BitwiseNot
            | ast::BuiltinOperatorKind::Negation
            | ast::BuiltinOperatorKind::Not => {
                assert_eq!(arguments.arity(), 1, "Known by type checking");
                let arg = &arguments.inner[0];

                let src = self.lower_expression(arg, instructions);
                let dst = self.tmp_variable();
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
            | ast::BuiltinOperatorKind::Remainder
            | ast::BuiltinOperatorKind::Equal
            | ast::BuiltinOperatorKind::NotEqual
            | ast::BuiltinOperatorKind::LessThan
            | ast::BuiltinOperatorKind::LessOrEqual
            | ast::BuiltinOperatorKind::GreaterThan
            | ast::BuiltinOperatorKind::GreaterOrEqual => {
                assert_eq!(arguments.arity(), 2, "Known by type checking");
                let lhs = &arguments.inner[0];
                let rhs = &arguments.inner[1];

                let lhs = self.lower_expression(lhs, instructions);
                let rhs = self.lower_expression(rhs, instructions);
                let dst = self.tmp_variable();
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
            ast::BuiltinOperatorKind::And => {
                assert_eq!(arguments.arity(), 2, "Known by type checking");
                let lhs = &arguments.inner[0];
                let rhs = &arguments.inner[1];

                let false_label = self.tmp_name("and.false");
                let end_label = self.tmp_name("and.end");
                let result = self.tmp_variable();

                let lhs = self.lower_expression(lhs, instructions);
                instructions.push(tacky::Instruction::JumpIfZero {
                    condition: lhs,
                    target: false_label.clone(),
                });

                let rhs = self.lower_expression(rhs, instructions);
                instructions.push(tacky::Instruction::JumpIfZero {
                    condition: rhs,
                    target: false_label.clone(),
                });

                instructions.extend([
                    tacky::Instruction::Copy {
                        src: tacky::Value::Constant(1),
                        dst: result.clone(),
                    },
                    tacky::Instruction::Jump(end_label.clone()),
                    tacky::Instruction::Label(false_label),
                    tacky::Instruction::Copy {
                        src: tacky::Value::Constant(0),
                        dst: result.clone(),
                    },
                    tacky::Instruction::Label(end_label),
                ]);

                result
            }
            ast::BuiltinOperatorKind::Or => {
                assert_eq!(arguments.arity(), 2, "Known by type checking");
                let lhs = &arguments.inner[0];
                let rhs = &arguments.inner[1];

                let true_label = self.tmp_name("or.true");
                let end_label = self.tmp_name("or.end");
                let result = self.tmp_variable();

                let lhs = self.lower_expression(lhs, instructions);
                instructions.push(tacky::Instruction::JumpIfNotZero {
                    condition: lhs,
                    target: true_label.clone(),
                });

                let rhs = self.lower_expression(rhs, instructions);
                instructions.push(tacky::Instruction::JumpIfNotZero {
                    condition: rhs,
                    target: true_label.clone(),
                });

                instructions.extend([
                    tacky::Instruction::Copy {
                        src: tacky::Value::Constant(0),
                        dst: result.clone(),
                    },
                    tacky::Instruction::Jump(end_label.clone()),
                    tacky::Instruction::Label(true_label),
                    tacky::Instruction::Copy {
                        src: tacky::Value::Constant(1),
                        dst: result.clone(),
                    },
                    tacky::Instruction::Label(end_label),
                ]);

                result
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
            ast::BuiltinOperatorKind::Not => Some(tacky::UnaryOperator::Not),
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
            ast::BuiltinOperatorKind::Equal => Some(tacky::BinaryOperator::Equal),
            ast::BuiltinOperatorKind::NotEqual => Some(tacky::BinaryOperator::NotEqual),
            ast::BuiltinOperatorKind::LessThan => Some(tacky::BinaryOperator::LessThan),
            ast::BuiltinOperatorKind::LessOrEqual => Some(tacky::BinaryOperator::LessOrEqual),
            ast::BuiltinOperatorKind::GreaterThan => Some(tacky::BinaryOperator::GreaterThan),
            ast::BuiltinOperatorKind::GreaterOrEqual => Some(tacky::BinaryOperator::GreaterOrEqual),
            _ => None,
        }
    }

    fn lower_block(
        &mut self,
        block: &ast::Block,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        let ast::Block {
            statements,
            final_expression,
            span: _,
        } = block;

        for statement in statements {
            self.lower_expression(statement, instructions);
        }

        self.lower_expression(final_expression, instructions)
    }

    fn tmp_name(&mut self, hint: impl ToString) -> tacky::Identifier {
        let ident = tacky::Identifier::Temporary {
            hint: hint.to_string(),
            id: self.fresh,
        };
        self.fresh += 1;
        ident
    }

    fn tmp_variable(&mut self) -> tacky::Value {
        let name = self.tmp_name("var");
        tacky::Value::Variable(name)
    }
}
