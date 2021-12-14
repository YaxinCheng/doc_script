use super::super::scope::Scoped;
use super::resolve_helper::ResolveHelper;
use super::typed_element::TypedElement;
use super::{Environment, Resolved};
use crate::ast::Name;
use std::collections::HashMap;

pub(in crate::env) struct InstanceField<'ast, 'a> {
    pub instance: TypedElement<'ast, 'a>,
    pub fields: Vec<&'a str>,
}

pub(in crate::env) struct NameResolver<'ast, 'a, 'env>(pub &'env mut Environment<'ast, 'a>);

/// Name resolving
impl<'ast, 'a, 'env> NameResolver<'ast, 'a, 'env> {
    pub(in crate::env) fn resolve_names<I: IntoIterator<Item = &'ast Name<'a>>>(
        mut self,
        names: I,
    ) -> HashMap<&'ast Name<'a>, InstanceField<'ast, 'a>> {
        let mut instance_field_accesses = HashMap::new();
        for name in names {
            match self.resolve_added_name(name) {
                Some(resolved) => {
                    self.0.resolved_names.insert(name.clone(), resolved);
                }
                None => match self.resolve_module_lead_name(name) {
                    Ok(resolved) => {
                        self.0.resolved_names.insert(name.clone(), resolved);
                    }
                    Err(instance_field) => {
                        instance_field_accesses.insert(name, instance_field);
                    }
                },
            };
        }
        instance_field_accesses
    }

    fn resolve_added_name(&mut self, name: &'ast Name<'a>) -> Option<Resolved<'ast, 'a>> {
        ResolveHelper(self.0)
            .resolve(name.scope(), &name.moniker)
            .map(|(resolved, _)| resolved)
    }

    fn resolve_module_lead_name(
        &mut self,
        name: &'ast Name<'a>,
    ) -> Result<Resolved<'ast, 'a>, InstanceField<'ast, 'a>> {
        let (resolved, not_resolved) =
            ResolveHelper(self.0).resolve_module_lead_name(name.scope(), &name.moniker);
        if not_resolved.is_empty() {
            Ok(resolved)
        } else {
            let instance = match resolved {
                Resolved::Constant(constant) => constant.into(),
                Resolved::Field(field) => field.into(),
                _ => unreachable!("Resolved can only be constant or field at this moment"),
            };
            let fields = not_resolved;
            Err(InstanceField { instance, fields })
        }
    }
}
