use crate::ast;
use crate::typ::{FunctionType, Type};

use super::{ElaborationError, Result, SymbolTable};

pub fn typecheck(program: &ast::Program) -> Result<SymbolTable> {
    let mut typechecker = Typechecker::default();
    typechecker.check_program(program)?;
    Ok(typechecker.symbol_table)
}

#[derive(Default)]
struct Typechecker {
    symbol_table: SymbolTable,
}

impl Typechecker {
    fn check_program(&mut self, program: &ast::Program) -> Result<()> {
        let ast::Program { functions, span: _ } = program;

        self.check_top_level_items(functions);

        for function in functions {
            self.check_function_definition(function)?;
        }

        Ok(())
    }

    fn check_top_level_items(&mut self, functions: &[ast::FunctionDefinition]) {
        for function in functions {
            let ast::FunctionDefinition {
                name,
                parameters,
                body: _,
                span: _,
            } = function;

            let parameters = parameters.arity();
            let typ = Type::Function(FunctionType { parameters });

            self.symbol_table.insert(name.id, typ);
        }
    }

    fn check_function_definition(&mut self, function: &ast::FunctionDefinition) -> Result<()> {
        let ast::FunctionDefinition {
            name,
            parameters,
            body,
            span: _,
        } = function;

        debug_assert!(self.symbol_table.contains_key(&name.id));

        self.check_parameters(parameters);
        self.check_expression(body)?;

        Ok(())
    }

    fn check_parameters(&mut self, parameters: &ast::ParameterList) {
        let ast::ParameterList {
            parameters,
            span: _,
        } = parameters;

        for parameter in parameters {
            let ast::Parameter { variable, span: _ } = parameter;
            self.symbol_table.insert(variable.name.id, Type::Int);
        }
    }

    fn check_expression(&mut self, expression: &ast::Expression) -> Result<()> {
        match expression {
            ast::Expression::Constant(_) => Ok(()),
            ast::Expression::Variable(variable) => self.check_variable(variable),
            ast::Expression::Declaration(declaration) => self.check_declaration(declaration),
            ast::Expression::Assignment(assignment) => self.check_assignment(assignment),
            ast::Expression::BuiltinCall(call) => self.check_builtin_call(call),
            ast::Expression::FunctionCall(call) => self.check_function_call(call),
            ast::Expression::Conditional(conditional) => self.check_conditional(conditional),
            ast::Expression::Block(block) => self.check_block(block),
            ast::Expression::Loop(loop_expr) => self.check_loop(loop_expr),
            ast::Expression::Break(ast::Break { id: _, span: _ }) => Ok(()),
            ast::Expression::Continue(_) => Ok(()),
        }
    }

    fn check_variable(&mut self, variable: &ast::Variable) -> Result<()> {
        let ast::Variable { name } = variable;

        let typ = self
            .symbol_table
            .get(&name.id)
            .expect("binder already checked");

        if *typ != Type::Int {
            return Err(ElaborationError::NotAVariable {
                name: name.identifier.clone(),
                span: name.span,
            });
        }

        Ok(())
    }

    fn check_declaration(&mut self, declaration: &ast::Declaration) -> Result<()> {
        let ast::Declaration {
            variable,
            initializer,
            span: _,
        } = declaration;

        let ast::Variable { name: variable } = variable;

        self.symbol_table.insert(variable.id, Type::Int);
        self.check_expression(initializer)?;

        Ok(())
    }

    fn check_assignment(&mut self, assignment: &ast::Assignment) -> Result<()> {
        let ast::Assignment { lhs, rhs, span: _ } = assignment;

        self.check_expression(&*lhs)?;
        self.check_expression(&*rhs)?;

        Ok(())
    }

    fn check_builtin_call(&mut self, builtin_call: &ast::BuiltinCall) -> Result<()> {
        let ast::BuiltinCall {
            operator,
            arguments,
            span: _,
        } = builtin_call;

        let expected_arity = builtin_arity(&operator.kind);
        let got_arity = arguments.arity();
        if got_arity != expected_arity {
            return Err(ElaborationError::BuiltinArityMismatch {
                expected: expected_arity,
                got: got_arity,
                span: operator.span,
            });
        }

        self.check_arguments(arguments)?;

        Ok(())
    }

    fn check_function_call(&mut self, function_call: &ast::FunctionCall) -> Result<()> {
        let ast::FunctionCall {
            function_name,
            arguments,
            span: _,
        } = function_call;

        let typ = self
            .symbol_table
            .get(&function_name.id)
            .expect("checked as top level item");

        let Type::Function(FunctionType { parameters }) = typ else {
            return Err(ElaborationError::NotAFunction {
                name: function_name.identifier.clone(),
                span: function_name.span,
            });
        };

        let expected_arity = *parameters;
        let got_arity = arguments.arity();

        if got_arity != expected_arity {
            return Err(ElaborationError::FunctionArityMismatch {
                expected: expected_arity,
                got: got_arity,
                span: function_name.span,
            });
        }

        self.check_arguments(arguments)?;

        Ok(())
    }

    fn check_arguments(&mut self, arguments: &ast::ArgumentList) -> Result<()> {
        let ast::ArgumentList { arguments, span: _ } = arguments;

        for argument in arguments {
            self.check_expression(argument)?;
        }

        Ok(())
    }

    fn check_conditional(&mut self, conditional: &ast::Conditional) -> Result<()> {
        let ast::Conditional {
            condition,
            then_branch,
            else_branch,
            span: _,
        } = conditional;

        self.check_expression(condition)?;
        self.check_expression(then_branch)?;
        self.check_expression(else_branch)?;

        Ok(())
    }

    fn check_block(&mut self, block: &ast::Block) -> Result<()> {
        let ast::Block {
            statements,
            final_expression,
            span: _,
        } = block;

        for statement in statements {
            self.check_expression(statement)?;
        }
        self.check_expression(final_expression)?;

        Ok(())
    }

    fn check_loop(&mut self, loop_expr: &ast::Loop) -> Result<()> {
        let ast::Loop {
            id: _,
            body,
            span: _,
        } = loop_expr;

        self.check_expression(body)
    }
}

fn builtin_arity(builtin: &ast::BuiltinOperatorKind) -> usize {
    match builtin {
        ast::BuiltinOperatorKind::BitwiseNot
        | ast::BuiltinOperatorKind::Negation
        | ast::BuiltinOperatorKind::Not => 1,
        ast::BuiltinOperatorKind::Addition
        | ast::BuiltinOperatorKind::Multiplication
        | ast::BuiltinOperatorKind::Subtraction
        | ast::BuiltinOperatorKind::Division
        | ast::BuiltinOperatorKind::Remainder
        | ast::BuiltinOperatorKind::And
        | ast::BuiltinOperatorKind::Or
        | ast::BuiltinOperatorKind::Equal
        | ast::BuiltinOperatorKind::NotEqual
        | ast::BuiltinOperatorKind::LessThan
        | ast::BuiltinOperatorKind::LessOrEqual
        | ast::BuiltinOperatorKind::GreaterThan
        | ast::BuiltinOperatorKind::GreaterOrEqual => 2,
    }
}
