use crate::{ast, tacky};

pub fn lower(program: &ast::Program) -> tacky::Program {
    Lowerer { fresh: 0 }.lower_program(program)
}

struct Lowerer {
    fresh: usize,
}

impl Lowerer {
    fn lower_program(&mut self, program: &ast::Program) -> tacky::Program {
        let ast::Program { functions, span: _ } = program;

        let functions = functions
            .iter()
            .map(|function| self.lower_function_definition(function))
            .collect();

        tacky::Program { functions }
    }

    fn lower_function_definition(
        &mut self,
        function: &ast::FunctionDefinition,
    ) -> tacky::FunctionDefinition {
        let ast::FunctionDefinition {
            name,
            parameters,
            body,
            span: _,
        } = function;

        let name = tacky::Identifier::Function(name.identifier.clone());
        let parameters = self.lower_parameters(parameters);

        let mut instructions = Vec::new();
        let body = self.lower_expression(body, &mut instructions);
        instructions.push(tacky::Instruction::Return(body));

        tacky::FunctionDefinition {
            name,
            parameters,
            body: instructions,
        }
    }

    fn lower_parameters(&self, parameters: &ast::ParameterList) -> Vec<tacky::Identifier> {
        let ast::ParameterList {
            parameters,
            span: _,
        } = parameters;

        let mut tacky_parameters = Vec::with_capacity(parameters.len());
        for parameter in parameters {
            let ast::Parameter { variable, span: _ } = parameter;
            let parameter = self.lower_variable_name(variable);
            tacky_parameters.push(parameter);
        }

        tacky_parameters
    }

    fn lower_expression(
        &mut self,
        expression: &ast::Expression,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        match expression {
            ast::Expression::Constant(constant) => self.lower_constant(constant),
            ast::Expression::Variable(variable) => self.lower_variable(variable),
            ast::Expression::Declaration(declaration) => {
                self.lower_declaration(declaration, instructions)
            }
            ast::Expression::Assignment(assignment) => {
                self.lower_assignment(assignment, instructions)
            }
            ast::Expression::BuiltinCall(call) => self.lower_builtin_call(call, instructions),
            ast::Expression::FunctionCall(call) => self.lower_function_call(call, instructions),
            ast::Expression::Conditional(conditional) => {
                self.lower_conditional(conditional, instructions)
            }
            ast::Expression::Block(block) => self.lower_block(block, instructions),
            ast::Expression::Loop(loop_expr) => self.lower_loop(loop_expr, instructions),
            ast::Expression::Break(break_expr) => self.lower_break(break_expr, instructions),
            ast::Expression::Continue(continue_expr) => {
                self.lower_continue(continue_expr, instructions)
            }
        }
    }

    fn lower_constant(&self, constant: &ast::IntegerConstant) -> tacky::Value {
        let ast::IntegerConstant { value, span: _ } = constant;
        tacky::Value::Constant(*value)
    }

    fn lower_variable(&self, variable: &ast::Variable) -> tacky::Value {
        let name = self.lower_variable_name(variable);
        tacky::Value::Variable(name)
    }

    fn lower_variable_name(&self, variable: &ast::Variable) -> tacky::Identifier {
        let ast::Variable { name } = variable;
        let ast::Name {
            identifier: name,
            id,
            span: _,
        } = name;

        tacky::Identifier::Variable {
            name: name.clone(),
            id: *id,
        }
    }

    fn lower_declaration(
        &mut self,
        declaration: &ast::Declaration,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        let ast::Declaration {
            variable,
            initializer,
            span: _,
        } = declaration;

        let variable = self.lower_variable(variable);
        let initializer = self.lower_expression(initializer, instructions);
        instructions.push(tacky::Instruction::Copy {
            src: initializer,
            dst: variable.clone(),
        });

        variable
    }

    fn lower_assignment(
        &mut self,
        assignment: &ast::Assignment,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        let ast::Assignment { lhs, rhs, span: _ } = assignment;

        let ast::Expression::Variable(variable) = &**lhs else {
            todo!("complex lvalue handling");
        };
        let variable = self.lower_variable(variable);

        let rhs = self.lower_expression(rhs, instructions);
        instructions.push(tacky::Instruction::Copy {
            src: rhs,
            dst: variable.clone(),
        });

        variable
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
                let arg = &arguments.arguments[0];

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
                let lhs = &arguments.arguments[0];
                let rhs = &arguments.arguments[1];

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
                let lhs = &arguments.arguments[0];
                let rhs = &arguments.arguments[1];

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
                let lhs = &arguments.arguments[0];
                let rhs = &arguments.arguments[1];

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

    fn lower_function_call(
        &mut self,
        function_call: &ast::FunctionCall,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        let ast::FunctionCall {
            function_name,
            arguments,
            span: _,
        } = function_call;

        let result = self.tmp_variable();

        let function_name = tacky::Identifier::Function(function_name.identifier.clone());
        let arguments = self.lower_arguments(arguments, instructions);

        instructions.push(tacky::Instruction::FunctionCall(tacky::FunctionCall {
            function_name,
            arguments,
            dst: result.clone(),
        }));

        result
    }

    fn lower_arguments(
        &mut self,
        arguments: &ast::ArgumentList,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> Vec<tacky::Value> {
        let ast::ArgumentList { arguments, span: _ } = arguments;

        let mut tacky_arguments = Vec::with_capacity(arguments.len());

        for argument in arguments {
            let argument = self.lower_expression(argument, instructions);
            tacky_arguments.push(argument);
        }

        tacky_arguments
    }

    fn lower_conditional(
        &mut self,
        conditional: &ast::Conditional,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        let ast::Conditional {
            condition,
            then_branch,
            else_branch,
            span: _,
        } = conditional;

        let else_label = self.tmp_name("if.else");
        let end_label = self.tmp_name("if.end");
        let result = self.tmp_variable();

        let condition = self.lower_expression(condition, instructions);

        instructions.push(tacky::Instruction::JumpIfZero {
            condition,
            target: else_label.clone(),
        });

        let then_branch = self.lower_expression(then_branch, instructions);

        instructions.extend([
            tacky::Instruction::Copy {
                src: then_branch,
                dst: result.clone(),
            },
            tacky::Instruction::Jump(end_label.clone()),
            tacky::Instruction::Label(else_label),
        ]);

        let else_branch = self.lower_expression(else_branch, instructions);

        instructions.extend([
            tacky::Instruction::Copy {
                src: else_branch,
                dst: result.clone(),
            },
            tacky::Instruction::Label(end_label),
        ]);

        result
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

    fn lower_loop(
        &mut self,
        loop_expr: &ast::Loop,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        let ast::Loop { id, body, span: _ } = loop_expr;

        let continue_label = tacky::Identifier::Loop {
            id: *id,
            kind: "continue".to_string(),
        };

        let break_label = tacky::Identifier::Loop {
            id: *id,
            kind: "break".to_string(),
        };

        instructions.push(tacky::Instruction::Label(continue_label.clone()));

        self.lower_expression(body, instructions);

        instructions.extend([
            tacky::Instruction::Jump(continue_label),
            tacky::Instruction::Label(break_label),
        ]);

        tacky::Value::Constant(0)
    }

    fn lower_break(
        &mut self,
        break_expr: &ast::Break,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        let ast::Break { id, span: _ } = break_expr;

        let break_label = tacky::Identifier::Loop {
            id: *id,
            kind: "break".to_string(),
        };
        instructions.push(tacky::Instruction::Jump(break_label));

        tacky::Value::Constant(0)
    }

    fn lower_continue(
        &mut self,
        continue_expr: &ast::Continue,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        let ast::Continue { id, span: _ } = continue_expr;

        let continue_label = tacky::Identifier::Loop {
            id: *id,
            kind: "continue".to_string(),
        };
        instructions.push(tacky::Instruction::Jump(continue_label));

        tacky::Value::Constant(0)
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
