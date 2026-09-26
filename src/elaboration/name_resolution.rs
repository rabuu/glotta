use std::collections::HashMap;

use crate::ast;

use super::{ElaborationError, Result};

pub struct NameResolver {
    function_names: HashMap<String, usize>,
    scopes: Vec<HashMap<String, usize>>,
    fresh: usize,
}

impl NameResolver {
    pub fn new() -> Self {
        NameResolver {
            function_names: HashMap::default(),
            scopes: vec![HashMap::default()],
            fresh: 1,
        }
    }

    pub fn resolve(&mut self, program: &mut ast::Program) -> Result<()> {
        let ast::Program { function } = program;
        self.resolve_function_definition(function)
    }

    fn resolve_function_definition(
        &mut self,
        function: &mut ast::FunctionDefinition,
    ) -> Result<()> {
        let ast::FunctionDefinition {
            name,
            body,
            span: _,
        } = function;

        // TODO: with multiple functions, we need to resolve all function definitions first
        self.resolve_function_name(name)?;
        self.resolve_expression(body)?;

        Ok(())
    }

    fn resolve_function_name(&mut self, name: &mut ast::Identifier) -> Result<()> {
        let ast::Identifier {
            identifier,
            id,
            span,
        } = name;

        if self.function_names.contains_key(identifier) {
            return Err(ElaborationError::DuplicateFunctionName {
                name: identifier.to_string(),
                span: *span,
            });
        }

        *id = self.fresh_id();
        self.function_names.insert(identifier.to_owned(), *id);

        Ok(())
    }

    fn resolve_expression(&mut self, expression: &mut ast::Expression) -> Result<()> {
        match expression {
            ast::Expression::Constant(_) => Ok(()),
            ast::Expression::Variable(variable) => self.resolve_variable(variable),
            ast::Expression::Declaration(declaration) => self.resolve_declaration(declaration),
            ast::Expression::Assignment(assignment) => self.resolve_assignment(assignment),
            ast::Expression::Call(call) => self.resolve_call(call),
            ast::Expression::Block(block) => self.resolve_block(block),
        }
    }

    fn resolve_variable(&mut self, variable: &mut ast::Variable) -> Result<()> {
        let ast::Variable { name } = variable;
        let ast::Identifier {
            identifier: variable,
            id,
            span,
        } = name;

        let Some(lookup) = self
            .scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(variable).copied())
        else {
            return Err(ElaborationError::VariableNotBound {
                variable: variable.to_string(),
                span: *span,
            });
        };

        *id = lookup;

        Ok(())
    }

    fn resolve_declaration(&mut self, declaration: &mut ast::Declaration) -> Result<()> {
        let ast::Declaration {
            variable,
            initializer,
            span: _,
        } = declaration;
        let ast::Variable { name: variable } = variable;
        let ast::Identifier {
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

    fn resolve_call(&mut self, call: &mut ast::Call) -> Result<()> {
        match call {
            ast::Call::Builtin(builtin_call) => self.resolve_builtin_call(builtin_call),
            ast::Call::Function(function_call) => self.resolve_function_call(function_call),
        }
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

        let ast::Identifier {
            identifier: function_name,
            id,
            span,
        } = function_name;

        let Some(lookup) = self.function_names.get(function_name).copied() else {
            return Err(ElaborationError::FunctionNotBound {
                function: function_name.to_string(),
                span: *span,
            });
        };

        *id = lookup;

        self.resolve_arguments(arguments)?;

        Ok(())
    }

    fn resolve_arguments(&mut self, arguments: &mut ast::ArgumentList) -> Result<()> {
        let ast::ArgumentList {
            inner: arguments,
            span: _,
        } = arguments;

        for argument in arguments {
            self.resolve_expression(argument)?;
        }

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

    fn fresh_id(&mut self) -> usize {
        let id = self.fresh;
        self.fresh += 1;
        id
    }
}
