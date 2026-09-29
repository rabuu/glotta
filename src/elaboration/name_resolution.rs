use std::collections::HashMap;

use crate::ast;

use super::{ElaborationError, Result};

type Id = usize;
type Scope = HashMap<String, Id>;

pub struct NameResolver {
    scopes: Vec<Scope>,
    fresh: Id,
}

impl NameResolver {
    pub fn new() -> Self {
        NameResolver {
            scopes: Vec::default(),
            fresh: 1,
        }
    }

    pub fn resolve(&mut self, program: &mut ast::Program) -> Result<()> {
        let ast::Program { functions, span: _ } = program;

        let global_scope = self.resolve_top_level_items(functions)?;
        self.scopes.push(global_scope);

        for function in functions {
            self.resolve_function_definition(function)?;
        }

        Ok(())
    }

    fn resolve_top_level_items(
        &mut self,
        functions: &mut [ast::FunctionDefinition],
    ) -> Result<Scope> {
        let mut global_scope = HashMap::with_capacity(functions.len());

        for function in functions {
            let ast::FunctionDefinition {
                name,
                parameters: _,
                body: _,
                span: _,
            } = function;

            let ast::Name {
                id,
                identifier: name,
                span: _,
            } = name;

            *id = self.fresh_id();

            if global_scope.insert(name.clone(), *id).is_some() {
                return Err(ElaborationError::DuplicateFunctionName {
                    name: name.clone(),
                    span: function.name.span,
                });
            }
        }

        Ok(global_scope)
    }

    fn resolve_function_definition(
        &mut self,
        function: &mut ast::FunctionDefinition,
    ) -> Result<()> {
        let ast::FunctionDefinition {
            name,
            parameters,
            body,
            span: _,
        } = function;

        debug_assert!(name.id != 0);

        let function_scope = self.resolve_parameters(parameters)?;
        self.scopes.push(function_scope);

        self.resolve_expression(body)?;

        self.scopes.pop().expect("function scope");

        Ok(())
    }

    fn resolve_parameters(&mut self, parameters: &mut ast::ParameterList) -> Result<Scope> {
        let ast::ParameterList {
            parameters,
            span: _,
        } = parameters;

        let mut function_scope = HashMap::new();

        for parameter in parameters {
            let ast::Parameter { name, span: _ } = parameter;
            let ast::Name {
                id,
                identifier: name,
                span,
            } = name;

            *id = self.fresh_id();

            if function_scope.insert(name.clone(), *id).is_some() {
                return Err(ElaborationError::DuplicateParameterName {
                    name: name.clone(),
                    span: *span,
                });
            }
        }

        Ok(function_scope)
    }

    fn resolve_expression(&mut self, expression: &mut ast::Expression) -> Result<()> {
        match expression {
            ast::Expression::Constant(_) => Ok(()),
            ast::Expression::Variable(variable) => self.resolve_variable(variable),
            ast::Expression::Declaration(declaration) => self.resolve_declaration(declaration),
            ast::Expression::Assignment(assignment) => self.resolve_assignment(assignment),
            ast::Expression::BuiltinCall(call) => self.resolve_builtin_call(call),
            ast::Expression::FunctionCall(call) => self.resolve_function_call(call),
            ast::Expression::Conditional(conditional) => self.resolve_conditional(conditional),
            ast::Expression::Block(block) => self.resolve_block(block),
            ast::Expression::Loop(loop_expr) => self.resolve_loop(loop_expr),
            ast::Expression::Break(ast::Break { id: _, span: _ }) => Ok(()),
            ast::Expression::Continue(_) => Ok(()),
        }
    }

    fn resolve_variable(&mut self, variable: &mut ast::Variable) -> Result<()> {
        let ast::Variable { name } = variable;
        self.resolve_name(name)
    }

    fn resolve_declaration(&mut self, declaration: &mut ast::Declaration) -> Result<()> {
        let ast::Declaration {
            variable,
            initializer,
            span: _,
        } = declaration;
        let ast::Variable { name: variable } = variable;
        let ast::Name {
            identifier: variable,
            id,
            span: _,
        } = variable;

        self.resolve_expression(initializer)?;

        *id = self.fresh_id();

        self.scopes
            .last_mut()
            .expect("there must always be at least one scope")
            .insert(variable.to_string(), *id);

        Ok(())
    }

    fn resolve_assignment(&mut self, assignment: &mut ast::Assignment) -> Result<()> {
        let ast::Assignment { lhs, rhs, span: _ } = assignment;

        self.resolve_expression(&mut *lhs)?;
        self.resolve_expression(&mut *rhs)?;

        Ok(())
    }

    fn resolve_builtin_call(&mut self, builtin_call: &mut ast::BuiltinCall) -> Result<()> {
        let ast::BuiltinCall {
            operator: _,
            arguments,
            span: _,
        } = builtin_call;

        self.resolve_arguments(arguments)?;

        Ok(())
    }

    fn resolve_function_call(&mut self, function_call: &mut ast::FunctionCall) -> Result<()> {
        let ast::FunctionCall {
            function_name,
            arguments,
            span: _,
        } = function_call;

        self.resolve_name(function_name)?;
        self.resolve_arguments(arguments)?;

        Ok(())
    }

    fn resolve_arguments(&mut self, arguments: &mut ast::ArgumentList) -> Result<()> {
        let ast::ArgumentList { arguments, span: _ } = arguments;

        for argument in arguments {
            self.resolve_expression(argument)?;
        }

        Ok(())
    }

    fn resolve_conditional(&mut self, conditional: &mut ast::Conditional) -> Result<()> {
        let ast::Conditional {
            condition,
            then_branch,
            else_branch,
            span: _,
        } = conditional;

        self.resolve_expression(condition)?;
        self.resolve_expression(then_branch)?;
        self.resolve_expression(else_branch)?;

        Ok(())
    }

    fn resolve_block(&mut self, block: &mut ast::Block) -> Result<()> {
        let ast::Block {
            statements,
            final_expression,
            span: _,
        } = block;

        self.scopes.push(HashMap::new());

        for statement in statements {
            self.resolve_expression(statement)?;
        }
        self.resolve_expression(final_expression)?;

        self.scopes.pop();

        Ok(())
    }

    fn resolve_loop(&mut self, loop_expr: &mut ast::Loop) -> Result<()> {
        let ast::Loop {
            id: _,
            body,
            span: _,
        } = loop_expr;

        self.resolve_expression(body)
    }

    fn resolve_name(&self, name: &mut ast::Name) -> Result<()> {
        let ast::Name {
            id,
            identifier,
            span,
        } = name;

        match self
            .scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(identifier).copied())
        {
            Some(lookup) => {
                *id = lookup;
                Ok(())
            }
            None => Err(ElaborationError::NameNotBound {
                name: identifier.clone(),
                span: *span,
            }),
        }
    }

    fn fresh_id(&mut self) -> Id {
        let id = self.fresh;
        self.fresh += 1;
        id
    }
}
