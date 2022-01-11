use super::super::scope::Scoped;
use super::resolve_helper::ResolveHelper;
use super::{Environment, Resolved};
use crate::ast::{Moniker, Name};

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
                .unwrap_or_else(|| self.disambiguate_name(name));
            self.0.resolved_names.insert(name.clone(), resolved);
        }
    }

    fn resolve_added_name(&mut self, name: &'ast Name<'a>) -> Option<Resolved<'ast, 'a>> {
        match &name.moniker {
            Moniker::Simple(simple_name) => {
                ResolveHelper(self.0).resolve(name.scope(), simple_name)
            }
            _ => None,
        }
    }

    fn disambiguate_name(&mut self, name: &'ast Name<'a>) -> Resolved<'ast, 'a> {
        ResolveHelper(self.0).disambiguate(name)
    }
}
