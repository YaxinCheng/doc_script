use super::super::scope::Scoped;
use super::resolve_helper::ResolveHelper;
use super::{Environment, Resolved};
use crate::ast::Name;

pub(in crate::env) struct NameResolver<'ast, 'a, 'env>(pub &'env mut Environment<'ast, 'a>);

/// Name resolving
impl<'ast, 'a, 'env> NameResolver<'ast, 'a, 'env> {
    pub(in crate::env) fn resolve_names<I: IntoIterator<Item = &'ast Name<'a>>>(
        mut self,
        names: I,
    ) {
        for name in names {
            let resolved = self
                .resolve_added_name(name)
                .unwrap_or_else(|| self.resolve_module_lead_name(name));
            self.0.resolved_names.insert(name.clone(), resolved);
        }
    }

    fn resolve_added_name(&mut self, name: &'ast Name<'a>) -> Option<Resolved<'ast, 'a>> {
        ResolveHelper(self.0).resolve(name.scope(), &name.moniker)
    }

    fn resolve_module_lead_name(&mut self, name: &'ast Name<'a>) -> Resolved<'ast, 'a> {
        let (resolved, not_resolved) =
            ResolveHelper(self.0).resolve_module_lead_name(name.scope(), &name.moniker);
        if not_resolved.is_empty() {
            resolved
        } else {
            let fields = not_resolved;
            let receiver = match resolved {
                Resolved::Constant(constant) => constant,
                _ => unreachable!(
                    "InstanceAccess only happens on constant or self (which is also a constant)"
                ),
            };
            Resolved::InstanceAccess(receiver, fields)
        }
    }
}
