use super::super::scope::*;
use super::Environment;
use crate::ast::{
    AbstractSyntaxTree, ConstantDeclaration, Declaration, Expression, Parameter, Statement,
    StructDeclaration,
};

pub(in crate::env) struct ScopeGenerator<'ast, 'a, 'env>(pub &'env mut Environment<'ast, 'a>);

impl<'ast, 'a, 'env> ScopeGenerator<'ast, 'a, 'env> {
    pub fn generate(
        mut self,
        syntax_trees: &mut [AbstractSyntaxTree<'a>],
        module_paths: &[Vec<&'a str>],
    ) {
        for (syntax_tree, module_path) in syntax_trees.iter_mut().zip(module_paths.iter()) {
            let module_scope = self
                .0
                .find_module(module_path)
                .unwrap_or_else(|| panic!("Failed to find module `{}`", module_path.join(".")));
            self.generate_scopes(syntax_tree, module_scope)
        }
    }

    fn generate_scopes(
        &mut self,
        syntax_tree: &mut AbstractSyntaxTree<'a>,
        module_scope_id: ScopeId,
    ) {
        for declaration in syntax_tree.compilation_unit.declarations.iter_mut() {
            self.generate_for_declaration(declaration, module_scope_id)
        }
    }

    fn generate_for_declaration(&mut self, declaration: &mut Declaration<'a>, scope_id: ScopeId) {
        match declaration {
            Declaration::Constant(constant) => self.generate_for_constant(constant, scope_id),
            Declaration::Struct(r#struct) => {
                self.generate_for_struct_declaration(r#struct, scope_id)
            }
            _ => (), // import does not need a scope
        }
    }

    fn generate_for_constant(&mut self, constant: &mut ConstantDeclaration<'a>, scope_id: ScopeId) {
        self.generate_for_expression(&mut constant.value, scope_id)
    }

    fn generate_for_expression(&mut self, expression: &mut Expression<'a>, scope_id: ScopeId) {
        match expression {
            Expression::ConstUse(constant_name) => {
                constant_name.set_scope(scope_id);
            }
            Expression::Literal { .. } => (),
            Expression::StructInit {
                name,
                parameters,
                init_content,
            } => {
                name.set_scope(scope_id);
                parameters
                    .iter_mut()
                    .for_each(|parameter| self.generate_for_parameter(parameter, scope_id));
                if let Some(init_content) = init_content {
                    let body_scope_id = self.0.add_child_scope(scope_id).id;
                    init_content.set_scope(body_scope_id);
                    init_content.expressions.iter_mut().for_each(|expression| {
                        self.generate_for_expression(expression, body_scope_id)
                    });
                }
            }
            Expression::ChainingMethodInvocation {
                receiver,
                accessors,
            } => {
                self.generate_for_expression(receiver, scope_id);
                for accessor_value in accessors
                    .iter_mut()
                    .filter_map(|accessor| accessor.value.as_mut())
                {
                    self.generate_for_expression(accessor_value, scope_id);
                }
            }
            Expression::Block(block) => {
                if block.statements.is_empty() {
                    return;
                }
                let body_scope_id = self.0.add_child_scope(scope_id).id;
                block.set_scope(body_scope_id);
                block
                    .statements
                    .iter_mut()
                    .for_each(|statement| self.generate_for_statement(statement, body_scope_id))
            }
        }
    }

    fn generate_for_parameter(&mut self, parameter: &mut Parameter<'a>, scope_id: ScopeId) {
        match parameter {
            Parameter::Plain(value)
            | Parameter::Labelled {
                label: _,
                content: value,
            } => self.generate_for_expression(value, scope_id),
        }
    }

    fn generate_for_statement(&mut self, statement: &mut Statement<'a>, scope_id: ScopeId) {
        match statement {
            Statement::Expression(expression) => self.generate_for_expression(expression, scope_id),
            Statement::ConstantDeclaration(constant) => {
                self.generate_for_constant(constant, scope_id)
            }
        };
    }

    fn generate_for_struct_declaration(
        &mut self,
        r#struct: &mut StructDeclaration<'a>,
        scope_id: ScopeId,
    ) {
        let empty_body = r#struct.body.attributes.is_empty() && r#struct.fields.is_empty();
        if empty_body {
            return;
        }
        let body_scope = self.0.add_child_scope(scope_id).id;
        r#struct.body.set_scope(body_scope);
        for field in r#struct.fields.iter_mut() {
            field.field_type.0.set_scope(body_scope);
            if let Some(default_value) = field.default_value.as_mut() {
                // field default value is not in the body scope
                self.generate_for_expression(default_value, scope_id);
            }
        }
        for declaration in r#struct.body.attributes.iter_mut() {
            self.generate_for_constant(declaration, body_scope)
        }
    }
}
