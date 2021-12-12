use super::super::scope::Scoped;
use super::resolve_helper::ResolveHelper;
use super::types::Types;
use super::{Environment, Resolved};
use crate::ast::{Block, ConstantDeclaration, Expression, Field, Name, Statement};
use crate::env::scope::ScopeId;
use crate::tokenizer::LiteralKind;
use std::collections::HashSet;

pub(in crate::env) struct NameResolver<'ast, 'a, 'env> {
    environment: &'env mut Environment<'ast, 'a>,
    resolving: HashSet<&'ast Name<'a>>,
}

/// Constructor
impl<'ast, 'a, 'env> NameResolver<'ast, 'a, 'env> {
    pub fn environment(environment: &'env mut Environment<'ast, 'a>) -> Self {
        Self {
            environment,
            resolving: HashSet::new(),
        }
    }
}

/// Name resolving
impl<'ast, 'a, 'env> NameResolver<'ast, 'a, 'env> {
    pub fn resolve_names<I: IntoIterator<Item = &'ast Name<'a>>>(mut self, names: I) {
        names.into_iter().for_each(|name| {
            self.resolve_name(name);
        });
    }

    fn resolve_name(&mut self, name: &'ast Name<'a>) -> Resolved<'ast, 'a> {
        if let Some(resolved) = self.environment.resolved_names.get(name) {
            return *resolved;
        }
        if !self.resolving.insert(name) {
            panic!("Cycle dependency found for name `{}`", name)
        }
        let resolved = ResolveHelper(self.environment)
            .resolve(name.scope(), &name.moniker)
            .map(|(resolved, _)| resolved)
            .or_else(|| self.disambiguate(name.scope(), &name.moniker))
            .unwrap_or_else(|| panic!("Failed to resolve name: {}", name));
        self.environment
            .resolved_names
            .insert(name.clone(), resolved);
        self.resolving.remove(name);
        resolved
    }

    fn disambiguate<N: AsRef<[&'a str]>>(
        &mut self,
        scope: ScopeId,
        moniker: &N,
    ) -> Option<Resolved<'ast, 'a>> {
        let names = moniker.as_ref();
        if names.len() <= 1 {
            // Simple name is not ambiguous
            return None;
        }
        let (first, rest) = Self::split_first_component(names)?;
        let (mut last_resolved, scope_id) =
            ResolveHelper(self.environment).resolve(scope, &first)?;
        for name in rest {
            last_resolved = match last_resolved {
                Resolved::Module(scope_id) => self.resolve_in_module(scope_id, name),
                Resolved::Struct(_) => panic!("Nested types are not supported for now"),
                Resolved::Constant(constant) => {
                    self.resolve_from_constant(constant, name, scope_id)
                }
                Resolved::Field(field) => self.resolve_from_field(field, name),
            }?;
        }
        Some(last_resolved)
    }

    fn split_first_component<'b, 's>(
        names: &'b [&'s str],
    ) -> Option<(&'b [&'s str], &'b [&'s str])> {
        if names.first() == Some(&"self") {
            Some(names.split_at(2))
        } else {
            let (first, remaining) = names.split_first()?;
            Some((std::slice::from_ref(first), remaining))
        }
    }

    fn resolve_in_module(
        &self,
        module_scope: ScopeId,
        name: &'a str,
    ) -> Option<Resolved<'ast, 'a>> {
        let name = std::slice::from_ref(&name);
        let scope = self.environment.get_scope(module_scope);
        ResolveHelper::resolve_declared(scope, name)
            .or_else(|| ResolveHelper::resolve_mod(scope, name))
    }

    fn resolve_from_constant(
        &mut self,
        constant: &'ast ConstantDeclaration<'a>,
        name: &'a str,
        scope: ScopeId,
    ) -> Option<Resolved<'ast, 'a>> {
        let resolved_type = self.resolve_type(scope, &constant.value);
        resolved_type
            .field(name)
            .map(Resolved::Field)
            .or_else(|| resolved_type.attribute(name).map(Resolved::Constant))
    }

    fn resolve_from_field(
        &mut self,
        field: &'ast Field<'a>,
        name: &'a str,
    ) -> Option<Resolved<'ast, 'a>> {
        let resolved_type = self.resolve_type_from_name(&field.field_type.0);
        resolved_type
            .field(name)
            .map(Resolved::Field)
            .or_else(|| resolved_type.attribute(name).map(Resolved::Constant))
    }
}

/// Type checking
impl<'ast, 'a, 'env> NameResolver<'ast, 'a, 'env> {
    fn resolve_type(
        &mut self,
        scope: ScopeId,
        expression: &'ast Expression<'a>,
    ) -> Types<'ast, 'a> {
        match expression {
            Expression::Block(block) => self.resolve_block(block),
            Expression::Literal { kind, .. } => Self::resolve_literal(kind),
            Expression::ConstUse(name) | Expression::StructInit { name, .. } => {
                self.resolve_type_from_name(name)
            }
            Expression::ChainingMethodInvocation { receiver, .. } => {
                self.resolve_type(scope, receiver)
            }
        }
    }

    fn resolve_block(&mut self, block: &'ast Block<'a>) -> Types<'ast, 'a> {
        let statements = &block.statements;
        let scope = block.scope();
        statements
            .last()
            .map(|statement| self.resolve_statement(scope, statement))
            .unwrap_or(Types::Void)
    }

    fn resolve_statement(
        &mut self,
        scope: ScopeId,
        statement: &'ast Statement<'a>,
    ) -> Types<'ast, 'a> {
        match statement {
            Statement::Expression(expression) => self.resolve_type(scope, expression),
            _ => Types::Void,
        }
    }

    fn resolve_type_from_name(&mut self, name: &'ast Name<'a>) -> Types<'ast, 'a> {
        match self.resolve_name(name) {
            Resolved::Struct(structure) => Types::Struct(structure),
            Resolved::Constant(constant) => self.resolve_type(name.scope(), &constant.value),
            Resolved::Field(field) => self.resolve_type_from_name(&field.field_type.0),
            Resolved::Module(_) => panic!("Unexpected module appeared: {}", name),
        }
    }

    fn resolve_literal(literal_kind: &LiteralKind) -> Types<'ast, 'a> {
        match literal_kind {
            LiteralKind::Binary | LiteralKind::Hex | LiteralKind::Integer => Types::Int,
            LiteralKind::Boolean => Types::Bool,
            LiteralKind::Floating => Types::Float,
            LiteralKind::String => Types::String,
        }
    }
}

#[cfg(test)]
impl<'ast, 'a, 'env> NameResolver<'ast, 'a, 'env> {
    pub fn test_disambiguate<N: AsRef<[&'a str]>>(
        &mut self,
        scope: ScopeId,
        moniker: &N,
    ) -> Option<Resolved<'ast, 'a>> {
        self.disambiguate(scope, moniker)
    }
    
    pub fn test_resolve_type(
        &mut self,
        scope: ScopeId,
        expression: &'ast Expression<'a>,
    ) -> Types<'ast, 'a> {
        self.resolve_type(scope, expression)        
    }
}
