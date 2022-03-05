use super::super::scope::*;
use super::{Environment, Resolved};
use crate::search::Traversal;
use std::collections::HashSet;

pub(in crate::env) struct ResolveHelper<'ast, 'a, 'env>(pub &'env Environment<'ast, 'a>);

impl<'ast, 'a, 'env> ResolveHelper<'ast, 'a, 'env> {
    /// This function kicks off a search from given scope all the way to the global scope
    /// trying to find an element that matches the given name.
    ///
    /// If successfully found, this function returns the resolved element
    pub fn resolve(&self, scope: ScopeId, name: &str) -> Option<Resolved<'ast, 'a>> {
        let start_scope = self.0.get_scope(scope);
        let mut traverse_to_global = Traversal::traverse(start_scope, |scope| match scope.id {
            GLOBAL_SCOPE => None,
            _ => Some(self.0.get_scope(scope.parent)),
        });
        let mut searched_scopes = HashSet::new();
        traverse_to_global
            .find_map(|scope| self.try_resolve_name(scope, name, &mut searched_scopes))
    }

    fn try_resolve_name(
        &self,
        scope: &Scope<'ast, 'a>,
        name: &str,
        searched_scopes: &mut HashSet<ScopeId>,
    ) -> Option<Resolved<'ast, 'a>> {
        Self::resolve_declared(scope, name)
            .or_else(|| self.resolve_from_wildcard_imports(scope, name, searched_scopes))
            .or_else(|| Self::resolve_mod(scope, name))
    }

    pub(in crate::env::name_resolution) fn resolve_declared(
        scope: &Scope<'ast, 'a>,
        name_slice: &str,
    ) -> Option<Resolved<'ast, 'a>> {
        scope
            .name_spaces
            .declared
            .get(name_slice)
            .copied()
            .map(Resolved::from)
    }

    fn resolve_from_wildcard_imports(
        &self,
        scope: &Scope<'ast, 'a>,
        name: &str,
        searched_scopes: &mut HashSet<ScopeId>,
    ) -> Option<Resolved<'ast, 'a>> {
        let mut resolved = scope
            .name_spaces
            .wildcard_imports
            .iter()
            .copied()
            .filter_map(|scope_id| match searched_scopes.insert(scope_id) {
                true => {
                    let scope = self.0.get_scope(scope_id);
                    self.try_resolve_name(scope, name, searched_scopes)
                }
                false => None,
            })
            .take(2)
            .collect::<Vec<_>>();
        if resolved.len() > 1 {
            panic!(
                "Name `{name}` is ambiguous. There are more than one options in wildcard imports"
            )
        }
        resolved.pop()
    }

    pub(in crate::env::name_resolution) fn resolve_mod(
        scope: &Scope<'ast, 'a>,
        name: &str,
    ) -> Option<Resolved<'ast, 'a>> {
        scope
            .name_spaces
            .modules
            .get(name)
            .copied()
            .map(Resolved::Module)
    }

    /// This function resolves a qualified name.
    /// For example, names like: `self.field.attribute` or `constant.field` or `module1.module2.Struct`
    pub(in crate::env::name_resolution) fn disambiguate<N: AsRef<[&'a str]>>(
        &self,
        scope: ScopeId,
        moniker: N,
    ) -> Resolved<'ast, 'a> {
        let (first_component, rest) = moniker.as_ref().split_first().expect("name is empty");
        let mut last_resolved = self
            .resolve(scope, first_component)
            .unwrap_or_else(|| panic!("Name `{first_component}` is unresolvable"));
        let mut access_iter = rest.iter().peekable();
        while let Some(component) = access_iter.peek() {
            last_resolved = match last_resolved {
                Resolved::Module(module_scope) => self
                    .resolve_in_module(module_scope, component)
                    .unwrap_or_else(|| panic!("`{component}` cannot be found in module")),
                Resolved::Constant(constant) => {
                    return Resolved::InstanceAccess(constant, access_iter.copied().collect())
                }
                Resolved::Trait(_) => panic!("Cannot access field from trait type definition"),
                Resolved::Struct(_) => panic!("Cannot access field from struct type definition"),
                Resolved::InstanceAccess { .. } => {
                    unreachable!("Field cannot be found at this stage")
                }
            };
            access_iter.next();
        }
        last_resolved
    }

    pub(in crate::env::name_resolution) fn resolve_in_module(
        &self,
        module_scope: ScopeId,
        name: &'a str,
    ) -> Option<Resolved<'ast, 'a>> {
        let module_scope = self.0.get_scope(module_scope);
        Self::resolve_declared(module_scope, name).or_else(|| Self::resolve_mod(module_scope, name))
    }
}
