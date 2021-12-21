use super::super::resolution::InstanceField;
use super::super::typed_element::TypedElement;
use super::super::types::Types;
use super::struct_init_checker::StructInitChecker;
use super::type_resolver;
use crate::ast::{
    AbstractSyntaxTree, Accessor, Block, Declaration, Expression, Field, Name, Parameter,
    Statement, StructDeclaration, StructInitContent,
};
use crate::env::address_hash::hash;
use crate::env::environment::Resolved;
use crate::env::Environment;
use std::collections::HashMap;

hash!(Field);
hash!(Expression);

#[derive(Default)]
pub(in crate::env) struct TypeChecker<'ast, 'a> {
    resolved_expressions: HashMap<&'ast Expression<'a>, Types<'ast, 'a>>,
    resolved_fields: HashMap<&'ast Field<'a>, Types<'ast, 'a>>,
    instance_fields: HashMap<&'ast Name<'a>, InstanceField<'ast, 'a>>,
}

impl<'ast, 'a> TypeChecker<'ast, 'a> {
    pub fn new(instance_fields: HashMap<&'ast Name<'a>, InstanceField<'ast, 'a>>) -> Self {
        Self {
            instance_fields,
            ..Default::default()
        }
    }
}

impl<'ast, 'a> TypeChecker<'ast, 'a> {
    pub fn resolve_and_check(
        mut self,
        environment: &mut Environment<'ast, 'a>,
        syntax_trees: &'ast [AbstractSyntaxTree<'a>],
    ) {
        for syntax_tree in syntax_trees {
            for declaration in &syntax_tree.compilation_unit.declarations {
                self.resolve_declaration(environment, declaration);
            }
        }
    }

    fn resolve_declaration(
        &mut self,
        environment: &mut Environment<'ast, 'a>,
        declaration: &'ast Declaration<'a>,
    ) {
        match declaration {
            Declaration::Constant(constant) => {
                self.resolve_expression(environment, &constant.value);
            }
            Declaration::Struct(r#struct) => self.resolve_struct(environment, r#struct),
            Declaration::Import(_) => (), // do nothing for import
        }
    }

    fn resolve_expression(
        &mut self,
        environment: &mut Environment<'ast, 'a>,
        expression: &'ast Expression<'a>,
    ) -> Types<'ast, 'a> {
        if let Some(resolved_type) = self.resolved_expressions.get(&expression) {
            return *resolved_type;
        }
        let resolve_type = match expression {
            Expression::ConstUse(name) => self.resolve_from_constant_use_name(environment, name),
            Expression::Literal { kind, .. } => type_resolver::resolve_literal(kind),
            Expression::Block(block) => self.resolve_block(environment, block),
            Expression::StructInit {
                name,
                parameters,
                init_content,
            } => self.resolve_struct_init(environment, name, parameters, init_content),
            Expression::ChainingMethodInvocation {
                receiver,
                accessors,
            } => self.resolve_chaining_method(environment, receiver, accessors),
        };
        let existing = self.resolved_expressions.insert(expression, resolve_type);
        debug_assert!(existing.is_none(), "Expression resolved twice");
        resolve_type
    }

    fn resolve_from_constant_use_name(
        &mut self,
        environment: &mut Environment<'ast, 'a>,
        name: &'ast Name<'a>,
    ) -> Types<'ast, 'a> {
        self.resolve_from_resolved_names(environment, name)
            .or_else(|| self.resolve_from_instance_fields(environment, name))
            .unwrap_or_else(|| {
                panic!(
                    "Unexpected name `{}`. This is likely a cycle reference",
                    name
                )
            })
    }

    fn resolve_from_resolved_names(
        &mut self,
        environment: &mut Environment<'ast, 'a>,
        name: &'ast Name<'a>,
    ) -> Option<Types<'ast, 'a>> {
        let (name, resolved) = environment.resolved_names.remove_entry(name)?;
        let resolved_type = match &resolved {
            Resolved::Struct(struct_type) => Types::Struct(struct_type),
            Resolved::Constant(constant) => self.resolve_expression(environment, &constant.value),
            Resolved::Field(field) => self.resolve_field(environment, field),
            Resolved::Module(_) => panic!("Cannot assign module to constant"),
            Resolved::InstanceAccess(access) => match access.last() {
                Some(TypedElement::Field(field)) => self.resolve_field(environment, field),
                Some(TypedElement::Constant(constant)) => {
                    self.resolve_expression(environment, &constant.value)
                }
                _ => unreachable!("Resolved field access is has more than one component"),
            },
        };
        environment.resolved_names.insert(name, resolved);
        Some(resolved_type)
    }

    fn resolve_from_instance_fields(
        &mut self,
        environment: &mut Environment<'ast, 'a>,
        name: &'ast Name<'a>,
    ) -> Option<Types<'ast, 'a>> {
        let InstanceField { instance, fields } = self.instance_fields.remove(name)?;
        let mut current_type = match instance {
            TypedElement::Constant(constant) => {
                self.resolve_expression(environment, &constant.value)
            }
            TypedElement::Field(field) => self.resolve_field(environment, field),
        };
        let mut field_access = vec![instance];
        for field in fields {
            let access = current_type
                .access(field)
                .unwrap_or_else(|| panic!("Failed to find {}", field));
            field_access.push(access);
            current_type = match access {
                TypedElement::Field(field) => self.resolve_field(environment, field),
                TypedElement::Constant(constant) => {
                    self.resolve_expression(environment, &constant.value)
                }
            };
        }
        environment
            .resolved_names
            .insert(name.clone(), Resolved::InstanceAccess(field_access));
        Some(current_type)
    }

    fn resolve_block(
        &mut self,
        environment: &mut Environment<'ast, 'a>,
        block: &'ast Block<'a>,
    ) -> Types<'ast, 'a> {
        block
            .statements
            .iter()
            .map(|statement| self.resolve_statement(environment, statement))
            .last()
            .unwrap_or(Types::Void)
    }

    fn resolve_struct_init(
        &mut self,
        environment: &mut Environment<'ast, 'a>,
        name: &'ast Name<'a>,
        parameters: &'ast [Parameter<'a>],
        init_content: &'ast Option<StructInitContent<'a>>,
    ) -> Types<'ast, 'a> {
        let struct_type = type_resolver::resolve_type_name(environment, name)
            .unwrap_or_else(|| panic!("type name `{}` not linked", name));
        let fields = struct_type.fields();
        let field_types = fields
            .iter()
            .map(|field| self.resolve_field(environment, field))
            .collect::<Vec<_>>();
        let parameter_types = parameters
            .iter()
            .map(|parameter| self.resolve_expression(environment, parameter.expression()))
            .collect::<Vec<_>>();
        StructInitChecker::with_fields(fields, field_types)
            .check_parameters(parameters, parameter_types)
            .expect("Failed struct field type check");
        init_content
            .iter()
            .for_each(|init_content| self.resolve_init_content(environment, init_content));
        struct_type
    }

    fn resolve_init_content(
        &mut self,
        environment: &mut Environment<'ast, 'a>,
        init_content: &'ast StructInitContent<'a>,
    ) {
        for expression in &init_content.expressions {
            self.resolve_expression(environment, expression);
            // TODO: check if type is compatible
        }
    }

    fn resolve_chaining_method(
        &mut self,
        environment: &mut Environment<'ast, 'a>,
        receiver: &'ast Expression<'a>,
        accessors: &'ast [Accessor<'a>],
    ) -> Types<'ast, 'a> {
        let receiver_type = self.resolve_expression(environment, receiver);
        for accessor in accessors {
            let field = receiver_type.field(accessor.identifier).unwrap_or_else(|| {
                panic!(
                    "Field `{}` could not be found in type `{:?}`",
                    accessor.identifier, receiver_type
                )
            });
            let field_type = self.resolve_field(environment, field);
            if let Some(value) = &accessor.value {
                let argument_type = self.resolve_expression(environment, value);
                assert_eq!(
                    argument_type, field_type,
                    "Expect type: `{:?}`\nFound type: `{:?}`, on access .{}",
                    field_type, argument_type, accessor.identifier
                );
            } else {
                assert!(
                    field.default_value.is_some(),
                    "Field `{}` has no default value",
                    field.name
                );
            }
        }
        receiver_type
    }

    fn resolve_statement(
        &mut self,
        environment: &mut Environment<'ast, 'a>,
        statement: &'ast Statement<'a>,
    ) -> Types<'ast, 'a> {
        match statement {
            Statement::Expression(expression) => self.resolve_expression(environment, expression),
            _ => Types::Void,
        }
    }

    fn resolve_struct(
        &mut self,
        environment: &mut Environment<'ast, 'a>,
        r#struct: &'ast StructDeclaration<'a>,
    ) {
        for field in &r#struct.fields {
            self.resolve_field(environment, field);
        }
        for attribute in &r#struct.body.attributes {
            self.resolve_expression(environment, &attribute.value);
        }
    }

    fn resolve_field(
        &mut self,
        environment: &mut Environment<'ast, 'a>,
        field: &'ast Field<'a>,
    ) -> Types<'ast, 'a> {
        if let Some(resolved_type) = self.resolved_fields.get(&field) {
            return *resolved_type;
        }
        let expected_type = type_resolver::resolve_type_name(environment, &field.field_type.0)
            .unwrap_or_else(|| panic!("Field type `{}` is invalid", field.field_type.0));
        if let Some(default_value) = &field.default_value {
            let value_type = self.resolve_expression(environment, default_value);
            assert_eq!(
                expected_type, value_type,
                "Field default value has a different type.\nExpected: {:?}\nFound: {:?}\n",
                expected_type, value_type
            )
        }
        let existing = self.resolved_fields.insert(field, expected_type);
        debug_assert!(
            existing.is_none(),
            "Duplicated field resolution: {}",
            field.name
        );
        expected_type
    }
}

#[cfg(test)]
impl<'ast, 'a> TypeChecker<'ast, 'a> {
    pub fn test_resolve_expression(
        &mut self,
        environment: &mut Environment<'ast, 'a>,
        expression: &'ast Expression<'a>,
    ) -> Types<'ast, 'a> {
        self.resolve_expression(environment, expression)
    }
}
