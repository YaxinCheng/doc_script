use super::super::scope::*;
use super::{Environment, Resolved};
use crate::search::Traversal;

pub(in crate::env::name_resolution) struct ResolveHelper<'ast, 'a, 'env>(
    pub &'env Environment<'ast, 'a>,
);

impl<'ast, 'a, 'env> ResolveHelper<'ast, 'a, 'env> {
    /// This function kicks off a search from given scope all the way to the global scope
    /// trying to find an element that matches the given moniker.
    ///
    /// If successfully found, this function returns the resolved element
    /// and the scope where the moniker was resolved
    ///
    /// # Warning
    /// When the moniker is resolved as an element which was imported through wildcard import,
    /// the returned scope is not the scope where the element was imported from,
    /// instead the scope is where the element was imported in
    ///
    /// # Arguments
    /// * `scope` - The scope where search starts
    /// * `moniker` - A description for name, such as `["com", "module", "type"]`
    pub fn resolve<N: AsRef<[&'a str]>>(
        self,
        scope: ScopeId,
        moniker: &N,
    ) -> Option<(Resolved<'ast, 'a>, ScopeId)> {
        let start_scope = self.0.get_scope(scope);
        let mut traverse_to_global = Traversal::traverse(start_scope, |scope| match scope.id {
            GLOBAL_SCOPE => None,
            _ => Some(self.0.get_scope(scope.parent)),
        });
        traverse_to_global
            .find_map(|scope| (self.try_resolve_name(scope, moniker).zip(Some(scope.id))))
    }

    fn try_resolve_name<N: AsRef<[&'a str]>>(
        &self,
        scope: &Scope<'ast, 'a>,
        name: &N,
    ) -> Option<Resolved<'ast, 'a>> {
        let name_slice = name.as_ref();
        Self::resolve_declared(scope, name_slice)
            .or_else(|| self.resolve_from_wildcard_imports(scope, name))
            .or_else(|| Self::resolve_mod(scope, name_slice))
    }

    pub(in crate::env::name_resolution) fn resolve_in_module(
        &self,
        module_scope: ScopeId,
        name: &'a str,
    ) -> Option<Resolved<'ast, 'a>> {
        let name = std::slice::from_ref(&name);
        let module_scope = self.0.get_scope(module_scope);
        Self::resolve_declared(module_scope, name).or_else(|| Self::resolve_mod(module_scope, name))
    }

    pub(in crate::env::name_resolution) fn resolve_declared(
        scope: &Scope<'ast, 'a>,
        name_slice: &[&str],
    ) -> Option<Resolved<'ast, 'a>> {
        scope
            .name_spaces
            .declared
            .get(name_slice)
            .copied()
            .map(Resolved::from)
    }

    fn resolve_from_wildcard_imports<N: AsRef<[&'a str]>>(
        &self,
        scope: &Scope<'ast, 'a>,
        name: &N,
    ) -> Option<Resolved<'ast, 'a>> {
        scope
            .name_spaces
            .wildcard_imports
            .iter()
            .copied()
            .map(|scope_id| self.0.get_scope(scope_id))
            .find_map(|scope| self.try_resolve_name(scope, name))
    }

    pub(in crate::env::name_resolution) fn resolve_mod(
        scope: &Scope<'ast, 'a>,
        name_slice: &[&str],
    ) -> Option<Resolved<'ast, 'a>> {
        match name_slice {
            [name] => scope
                .name_spaces
                .modules
                .get(name)
                .copied()
                .map(Resolved::Module),
            _ => None,
        }
    }

    pub(in crate::env::name_resolution) fn resolve_module_lead_name<N: AsRef<[&'a str]>>(
        &self,
        scope: ScopeId,
        name: N,
    ) -> (Resolved<'ast, 'a>, Vec<&'a str>) {
        let (first_component, rest) = super::split_first_component(name.as_ref());
        let (mut last_resolved, _) = ResolveHelper(self.0)
            .resolve(scope, &first_component)
            .unwrap_or_else(|| panic!("Name `{}` is unresolvable", first_component.join(".")));
        let mut access_iter = rest.iter();
        while let Some(component) = access_iter.next() {
            last_resolved = match last_resolved {
                Resolved::Module(module_scope) => self
                    .resolve_in_module(module_scope, component)
                    .unwrap_or_else(|| panic!("`{}` cannot be found in module", component)),
                resolved @ (Resolved::Constant(_) | Resolved::Field(_)) => {
                    return (
                        resolved,
                        std::iter::once(component)
                            .chain(access_iter)
                            .copied()
                            .collect(),
                    )
                }
                Resolved::Struct(_) => panic!("Cannot access field from struct type definition"),
                Resolved::InstanceAccess { .. } => {
                    unreachable!("Field cannot be found at this stage")
                }
            };
        }
        (last_resolved, vec![])
    }
}
