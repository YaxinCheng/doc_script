use super::super::hash;
use super::assignable_checker::AssignableChecker;
use super::error::*;
use super::essential_trait;
use super::struct_init_checker::StructInitChecker;
use super::type_resolver;
use crate::ast::{
    AbstractSyntaxTree, Accessor, Block, ConstantDeclaration, Declaration, Expression, Field, Name,
    Parameter, Statement, StructDeclaration, StructInitContent, TraitDeclaration,
};
use crate::env::checks::type_checking::types::Types;
use crate::env::environment::Resolved;
use crate::env::name_resolution::ResolveHelper;
use crate::env::scope::ScopeId;
use crate::env::Environment;
use crate::env::TypedElement;
use std::collections::{HashMap, HashSet};

hash!(Field);
hash!(Expression);

pub(in crate::env) struct TypeChecker<'ast, 'a, 'env> {
    pub(in crate::env) environment: &'env Environment<'ast, 'a>,
    resolved_expressions: HashMap<&'ast Expression<'a>, Types<'ast, 'a>>,
    resolved_fields: HashMap<&'ast Field<'a>, Types<'ast, 'a>>,
    resolved_instance_fields: HashMap<Name<'a>, Types<'ast, 'a>>,
    checking_expression: HashSet<Name<'a>>,
}

impl<'ast, 'a, 'env> TypeChecker<'ast, 'a, 'env> {
    pub fn with_environment(environment: &'env Environment<'ast, 'a>) -> Self {
        Self {
            environment,
            resolved_expressions: HashMap::new(),
            resolved_fields: HashMap::new(),
            resolved_instance_fields: HashMap::new(),
            checking_expression: HashSet::new(),
        }
    }

    pub fn check(mut self, syntax_trees: &'ast [AbstractSyntaxTree<'a>]) {
        for syntax_tree in syntax_trees {
            for declaration in &syntax_tree.compilation_unit.declarations {
                self.resolve_declaration(declaration);
            }
        }
    }

    pub fn entry_check(&mut self) {
        let entry = self.environment.entry().expect("Main cannot be found");
        let entry_type = self.resolve_expression(&entry.value);
        let render_trait = essential_trait::render(self.environment);
        assert!(
            AssignableChecker(self).check(&entry_type, &render_trait),
            "Entry needs to conform trait Render. `{entry_type}` does not conform trait `Render`"
        );
    }

    fn resolve_declaration(&mut self, declaration: &'ast Declaration<'a>) {
        match declaration {
            Declaration::Constant(constant) => {
                self.resolve_expression(&constant.value);
            }
            Declaration::Struct(r#struct) => self.resolve_struct(r#struct),
            Declaration::Trait(r#trait) => self.resolve_trait(r#trait),
            Declaration::Import(_) => (), // do nothing for import
        }
    }

    pub(in crate::env) fn resolve_expression(
        &mut self,
        expression: &'ast Expression<'a>,
    ) -> Types<'ast, 'a> {
        if let Some(resolved_type) = self.resolved_expressions.get(&expression) {
            return *resolved_type;
        }
        let resolve_type = match expression {
            Expression::Void => Types::Void,
            Expression::ConstUse(name) => self.resolve_from_constant_use_name(name),
            Expression::SelfRef(scope_id) => {
                self.resolve_self(scope_id.expect("self scope not set"))
            }
            Expression::Literal { kind, .. } => type_resolver::resolve_literal(kind),
            Expression::Block(block) => self.resolve_block(block),
            Expression::StructInit {
                name,
                parameters,
                init_content,
            } => self.resolve_struct_init(name, parameters, init_content),
            Expression::ChainingMethodInvocation {
                receiver,
                accessors,
            } => self.resolve_chaining_method(receiver, accessors),
            Expression::FieldAccess {
                receiver,
                field_names,
            } => self.resolve_field_access(receiver, field_names),
        };
        let existing = self.resolved_expressions.insert(expression, resolve_type);
        debug_assert!(existing.is_none(), "Expression resolved twice");
        resolve_type
    }

    fn resolve_from_constant_use_name(&mut self, name: &Name<'a>) -> Types<'ast, 'a> {
        self.resolve_from_resolved_names(name)
            .unwrap_or_else(|| panic!("Unresolvable name `{}`", name))
    }

    fn resolve_from_resolved_names(&mut self, name: &Name<'a>) -> Option<Types<'ast, 'a>> {
        if !self.checking_expression.insert(name.clone()) {
            panic!("Cycle reference detected for {}", name)
        }
        let resolved = self.environment.resolved_names.get(name)?;
        let resolved_type = match &resolved {
            Resolved::Constant(constant) => self.resolve_expression(&constant.value),
            Resolved::InstanceAccess(instance, fields) => {
                if let Some(cached) = self.resolved_instance_fields.get(name) {
                    *cached
                } else {
                    let resolved_type = self.resolve_from_instance_fields(instance, fields);
                    self.resolved_instance_fields
                        .insert(name.clone(), resolved_type);
                    resolved_type
                }
            }
            Resolved::Module(_) => panic!("Cannot assign module `{}` to constant", name),
            Resolved::Struct(struct_type) => {
                panic!("Cannot assign struct `{}` to constant", struct_type.name)
            }
            Resolved::Trait(trait_type) => {
                panic!("Cannot assign trait `{}` to constant", trait_type.name)
            }
        };
        self.checking_expression.remove(name);
        Some(resolved_type)
    }

    fn resolve_self(&self, scope: ScopeId) -> Types<'ast, 'a> {
        match ResolveHelper(self.environment).resolve(scope, "self") {
            Some(Resolved::Struct(struct_declaration)) => Types::Struct(struct_declaration),
            _ => panic!("self can only be used in structs"),
        }
    }

    fn resolve_from_instance_fields(
        &mut self,
        instance: &'ast ConstantDeclaration<'a>,
        fields: &[&'a str],
    ) -> Types<'ast, 'a> {
        let mut current_type = self.resolve_expression(&instance.value);
        for field in fields {
            let access = current_type
                .access(field)
                .unwrap_or_else(|| panic!("Failed to find {}", field));
            current_type = match access {
                TypedElement::Field(field) => self.resolve_field(field),
                TypedElement::Constant(constant) => self.resolve_expression(&constant.value),
            };
        }
        current_type
    }

    fn resolve_block(&mut self, block: &'ast Block<'a>) -> Types<'ast, 'a> {
        block
            .statements
            .iter()
            .map(|statement| self.resolve_statement(statement))
            .last()
            .unwrap_or(Types::Void)
    }

    fn resolve_struct_init(
        &mut self,
        name: &'ast Name<'a>,
        parameters: &'ast [Parameter<'a>],
        init_content: &'ast Option<StructInitContent<'a>>,
    ) -> Types<'ast, 'a> {
        let struct_type = type_resolver::resolve_type_name(self.environment, name)
            .unwrap_or_else(|| panic!("type name `{}` not linked", name));
        let fields = struct_type.fields();
        let field_types = fields
            .iter()
            .map(|field| self.resolve_field(field))
            .collect::<Vec<_>>();
        let parameter_types = parameters
            .iter()
            .map(|parameter| self.resolve_expression(parameter.expression()))
            .collect::<Vec<_>>();
        let (fields, field_types) = if let Some(init_content) = init_content {
            self.check_can_have_init_content(field_types.last())
                .unwrap_or_else(|error| {
                    panic!("struct `{struct_type}` cannot have init content. Error: {error}")
                });
            self.resolve_init_content(init_content)
                .expect("init content has non render-able content");
            (
                &fields[..fields.len() - 1],
                &field_types[..field_types.len() - 1],
            )
        } else {
            (fields, field_types.as_slice())
        };
        StructInitChecker::new(AssignableChecker(self))
            .check_parameters(parameters, parameter_types, fields, field_types)
            .expect("Failed struct field type check");
        struct_type
    }

    fn check_can_have_init_content(&mut self, field_type: Option<&Types>) -> Result<()> {
        match field_type {
            Some(Types::Children) => Ok(()),
            None => Err(Error::NotExpectingChildren),
            _ => Err(Error::LastFieldIsNotChildren),
        }
    }

    fn resolve_init_content(&mut self, init_content: &'ast StructInitContent<'a>) -> Result<()> {
        let render_trait = essential_trait::render(self.environment);
        for expression in &init_content.0 {
            let expr_type = self.resolve_expression(expression);
            if !AssignableChecker(self).check(&expr_type, &render_trait) {
                return Err(Error::InitContentNotRender(expr_type.to_string()));
            }
        }
        Ok(())
    }

    fn resolve_chaining_method(
        &mut self,
        receiver: &'ast Expression<'a>,
        accessors: &'ast [Accessor<'a>],
    ) -> Types<'ast, 'a> {
        let receiver_type = self.resolve_expression(receiver);
        for accessor in accessors {
            let field = receiver_type.field(accessor.identifier).unwrap_or_else(|| {
                panic!(
                    "Field `{}` could not be found in type `{:?}`",
                    accessor.identifier, receiver_type
                )
            });
            let field_type = self.resolve_field(field);
            if let Some(value) = &accessor.value {
                let argument_type = self.resolve_expression(value);
                if !AssignableChecker(self).check(&argument_type, &field_type) {
                    panic!(
                        "Expect type: `{}`\nFound type: `{}`, on access .{}",
                        field_type, argument_type, accessor.identifier
                    );
                }
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

    fn resolve_statement(&mut self, statement: &'ast Statement<'a>) -> Types<'ast, 'a> {
        match statement {
            Statement::Expression(expression) => self.resolve_expression(expression),
            _ => Types::Void,
        }
    }

    fn resolve_struct(&mut self, r#struct: &'ast StructDeclaration<'a>) {
        for field in &r#struct.fields {
            self.resolve_field(field);
        }
        if let Some(body) = &r#struct.body {
            for attribute in &body.attributes {
                self.resolve_expression(&attribute.value);
            }
        }
    }

    fn resolve_trait(&mut self, r#trait: &'ast TraitDeclaration<'a>) {
        for field in &r#trait.required {
            self.resolve_field(field);
        }
    }

    fn resolve_field(&mut self, field: &'ast Field<'a>) -> Types<'ast, 'a> {
        if let Some(resolved_type) = self.resolved_fields.get(&field) {
            return *resolved_type;
        }
        let expected_type = type_resolver::resolve_type_name(self.environment, &field.field_type.0)
            .unwrap_or_else(|| panic!("Field type `{}` is invalid", field.field_type.0));
        if let Some(default_value) = &field.default_value {
            let value_type = self.resolve_expression(default_value);
            if !AssignableChecker(self).check(&value_type, &expected_type) {
                panic!(
                    "Default value for field `{}` has a different type.\nExpected: {}\nFound: {}\n",
                    field.name, expected_type, value_type
                )
            }
        }
        let existing = self.resolved_fields.insert(field, expected_type);
        debug_assert!(
            existing.is_none(),
            "Duplicated field resolution: {}",
            field.name
        );
        expected_type
    }

    fn resolve_field_access(
        &mut self,
        receiver: &'ast Expression<'a>,
        name: &[&'a str],
    ) -> Types<'ast, 'a> {
        let receiver_type = self.resolve_expression(receiver);
        let mut last_type = receiver_type;
        for name in name {
            last_type = match last_type.access(name) {
                Some(TypedElement::Field(field)) => self.resolve_field(field),
                Some(TypedElement::Constant(constant)) => self.resolve_expression(&constant.value),
                None => panic!("{:?} has no field or attribute named {}", last_type, name),
            };
        }
        last_type
    }
}

#[cfg(test)]
impl<'ast, 'a, 'env> TypeChecker<'ast, 'a, 'env> {
    pub fn test_resolve_expression(&mut self, expression: &'ast Expression<'a>) -> Types<'ast, 'a> {
        self.resolve_expression(expression)
    }
}
