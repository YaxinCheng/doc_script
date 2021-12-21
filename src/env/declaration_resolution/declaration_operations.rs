use crate::ast::{
    AbstractSyntaxTree, ConstantDeclaration, Declaration, Expression, Parameter, Statement,
    StructDeclaration,
};
use crate::env::declaration_resolution::UnresolvedNames;
use crate::env::scope::*;
use crate::env::Environment;

pub(in crate::env) struct DeclarationAdder<'ast, 'a, 'env>(pub &'env mut Environment<'ast, 'a>);

impl<'ast, 'a, 'env> DeclarationAdder<'ast, 'a, 'env> {
    /// Add all declarations to the environment and return unresolved names
    pub fn add_from(
        mut self,
        syntax_trees: &'ast [AbstractSyntaxTree<'a>],
        module_paths: &[Vec<&'a str>],
    ) -> UnresolvedNames<'ast, 'a> {
        let mut names = UnresolvedNames::default();
        for (syntax_tree, module_path) in syntax_trees.iter().zip(module_paths.iter()) {
            let module_scope = self
                .0
                .find_module(module_path)
                .unwrap_or_else(|| panic!("Failed to find module `{}`", module_path.join(".")));
            for declaration in &syntax_tree.compilation_unit.declarations {
                self.add_declaration(declaration, module_scope, &mut names);
            }
        }
        names
    }

    fn add_declaration(
        &mut self,
        declaration: &'ast Declaration<'a>,
        scope_id: ScopeId,
        seen_names: &mut UnresolvedNames<'ast, 'a>,
    ) {
        match declaration {
            Declaration::Constant(constant) => {
                self.add_constant(false, constant, scope_id, seen_names)
            }
            Declaration::Struct(r#struct) => {
                self.add_struct_declaration(r#struct, scope_id, seen_names)
            }
            _ => (), // imports are ignored
        }
    }

    fn add_constant(
        &mut self,
        is_struct_attr: bool,
        constant: &'ast ConstantDeclaration<'a>,
        scope_id: ScopeId,
        seen_names: &mut UnresolvedNames<'ast, 'a>,
    ) {
        let scope = self.0.get_scope_mut(scope_id);
        let constant_name = match is_struct_attr {
            true => vec!["self", constant.name],
            false => vec![constant.name],
        };
        let duplicate_declaration = scope
            .name_spaces
            .declared
            .insert(constant_name, constant.into());
        assert!(
            duplicate_declaration.is_none(),
            "Cannot redefine constant in the same module with name: {}",
            constant.name
        );
        self.add_expression(&constant.value, scope_id, seen_names)
    }

    fn add_expression(
        &mut self,
        expression: &'ast Expression<'a>,
        scope_id: ScopeId,
        seen_names: &mut UnresolvedNames<'ast, 'a>,
    ) {
        match expression {
            Expression::ConstUse(constant_name) => {
                seen_names.expression_names.insert(constant_name);
            }
            Expression::Literal { .. } => (),
            Expression::StructInit {
                name,
                parameters,
                init_content,
            } => {
                seen_names.type_names.insert(name);
                parameters
                    .iter()
                    .for_each(|parameter| self.add_parameter(parameter, scope_id, seen_names));
                if let Some(init_content) = init_content {
                    let body_scope_id = init_content.scope();
                    init_content.expressions.iter().for_each(|expression| {
                        self.add_expression(expression, body_scope_id, seen_names)
                    });
                }
            }
            Expression::ChainingMethodInvocation {
                receiver,
                accessors,
            } => {
                self.add_expression(receiver, scope_id, seen_names);
                for accessor_value in accessors
                    .iter()
                    .filter_map(|accessor| accessor.value.as_ref())
                {
                    self.add_expression(accessor_value, scope_id, seen_names);
                }
            }
            Expression::Block(block) => {
                if block.statements.is_empty() {
                    return;
                }
                let body_scope_id = block.scope();
                block
                    .statements
                    .iter()
                    .for_each(|statement| self.add_statement(statement, body_scope_id, seen_names))
            }
        }
    }

    fn add_parameter(
        &mut self,
        parameter: &'ast Parameter<'a>,
        scope_id: ScopeId,
        seen_names: &mut UnresolvedNames<'ast, 'a>,
    ) {
        match parameter {
            Parameter::Plain(value)
            | Parameter::Labelled {
                label: _,
                content: value,
            } => self.add_expression(value, scope_id, seen_names),
        }
    }

    fn add_statement(
        &mut self,
        statement: &'ast Statement<'a>,
        scope_id: ScopeId,
        seen_names: &mut UnresolvedNames<'ast, 'a>,
    ) {
        match &statement {
            Statement::Expression(expression) => {
                self.add_expression(expression, scope_id, seen_names)
            }
            Statement::ConstantDeclaration(constant) => {
                self.add_constant(false, constant, scope_id, seen_names)
            }
        };
    }

    fn add_struct_declaration(
        &mut self,
        r#struct: &'ast StructDeclaration<'a>,
        scope_id: ScopeId,
        seen_names: &mut UnresolvedNames<'ast, 'a>,
    ) {
        let scope = self.0.get_scope_mut(scope_id);
        let duplicate_declaration = scope
            .name_spaces
            .declared
            .insert(vec![r#struct.name], r#struct.into());
        assert!(
            duplicate_declaration.is_none(),
            "Cannot redefine struct in the same module with name: {}",
            r#struct.name
        );
        let skip_body_scope = r#struct.body.attributes.is_empty() && r#struct.fields.is_empty();
        if skip_body_scope {
            return;
        }
        let body_scope_id = r#struct.body.scope();
        let body_scope = self.0.get_scope_mut(body_scope_id);
        for field in &r#struct.fields {
            seen_names.type_names.insert(&field.field_type.0);
            let insert_result = body_scope
                .name_spaces
                .declared
                .insert(vec!["self", field.name], field.into());
            assert!(
                insert_result.is_none(),
                "Conflicting field name: {}",
                field.name
            );
        }
        r#struct
            .fields
            .iter()
            .filter_map(|field| field.default_value.as_ref())
            .for_each(|default_value| self.add_expression(default_value, scope_id, seen_names));
        for declaration in &r#struct.body.attributes {
            self.add_constant(true, declaration, body_scope_id, seen_names)
        }
    }
}
