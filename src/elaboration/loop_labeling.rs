use crate::ast;

use super::{ElaborationError, Result};

pub struct LoopLabeler {
    fresh: usize,
    current_loop_id: Option<usize>,
}

impl LoopLabeler {
    pub fn new() -> Self {
        Self {
            fresh: 1,
            current_loop_id: None,
        }
    }

    pub fn label(&mut self, program: &mut ast::Program) -> Result<()> {
        let ast::Program { function } = program;
        self.label_function_definition(function)
    }

    fn label_function_definition(&mut self, function: &mut ast::FunctionDefinition) -> Result<()> {
        let ast::FunctionDefinition {
            name: _,
            body,
            span: _,
        } = function;

        self.label_expression(body)
    }

    fn label_expression(&mut self, expression: &mut ast::Expression) -> Result<()> {
        match expression {
            ast::Expression::Constant(_) => Ok(()),
            ast::Expression::Variable(_) => Ok(()),
            ast::Expression::Declaration(declaration) => self.label_declaration(declaration),
            ast::Expression::Assignment(assignment) => self.label_assignment(assignment),
            ast::Expression::BuiltinCall(builtin_call) => self.label_builtin_call(builtin_call),
            ast::Expression::FunctionCall(function_call) => self.label_function_call(function_call),
            ast::Expression::Conditional(conditional) => self.label_conditional(conditional),
            ast::Expression::Block(block) => self.label_block(block),
            ast::Expression::Loop(loop_expr) => self.label_loop(loop_expr),
            ast::Expression::Break(break_expr) => self.label_break(break_expr),
            ast::Expression::Continue(continue_expr) => self.label_continue(continue_expr),
        }
    }

    fn label_loop(&mut self, loop_expr: &mut ast::Loop) -> Result<()> {
        let ast::Loop { id, body, span: _ } = loop_expr;

        *id = self.fresh_id();

        let previous_id = self.current_loop_id;
        self.current_loop_id = Some(*id);
        self.label_expression(body)?;
        self.current_loop_id = previous_id;

        Ok(())
    }

    fn label_break(&mut self, break_expr: &mut ast::Break) -> Result<()> {
        let ast::Break { id, span } = break_expr;

        let Some(current_loop_id) = self.current_loop_id else {
            return Err(ElaborationError::OrphanedBreak { span: *span });
        };

        *id = current_loop_id;

        Ok(())
    }

    fn label_continue(&mut self, continue_expr: &mut ast::Continue) -> Result<()> {
        let ast::Continue { id, span } = continue_expr;

        let Some(current_loop_id) = self.current_loop_id else {
            return Err(ElaborationError::OrphanedContinue { span: *span });
        };

        *id = current_loop_id;

        Ok(())
    }

    fn label_declaration(&mut self, declaration: &mut ast::Declaration) -> Result<()> {
        let ast::Declaration {
            variable: _,
            initializer,
            span: _,
        } = declaration;

        self.label_expression(initializer)
    }

    fn label_assignment(&mut self, assignment: &mut ast::Assignment) -> Result<()> {
        let ast::Assignment { lhs, rhs, span: _ } = assignment;
        self.label_expression(lhs)?;
        self.label_expression(rhs)?;
        Ok(())
    }

    fn label_builtin_call(&mut self, builtin_call: &mut ast::BuiltinCall) -> Result<()> {
        let ast::BuiltinCall {
            operator: _,
            arguments,
            span: _,
        } = builtin_call;

        self.label_arguments(arguments)
    }

    fn label_function_call(&mut self, function_call: &mut ast::FunctionCall) -> Result<()> {
        let ast::FunctionCall {
            function_name: _,
            arguments,
            span: _,
        } = function_call;

        self.label_arguments(arguments)
    }

    fn label_arguments(&mut self, arguments: &mut ast::ArgumentList) -> Result<()> {
        let ast::ArgumentList { arguments, span: _ } = arguments;

        for argument in arguments {
            self.label_expression(argument)?;
        }

        Ok(())
    }

    fn label_conditional(&mut self, conditional: &mut ast::Conditional) -> Result<()> {
        let ast::Conditional {
            condition,
            then_branch,
            else_branch,
            span: _,
        } = conditional;

        self.label_expression(condition)?;
        self.label_expression(then_branch)?;
        self.label_expression(else_branch)?;

        Ok(())
    }

    fn label_block(&mut self, block: &mut ast::Block) -> Result<()> {
        let ast::Block {
            statements,
            final_expression,
            span: _,
        } = block;

        for statement in statements {
            self.label_expression(statement)?;
        }
        self.label_expression(final_expression)?;

        Ok(())
    }

    fn fresh_id(&mut self) -> usize {
        let id = self.fresh;
        self.fresh += 1;
        id
    }
}
