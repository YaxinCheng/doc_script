pub use super::name_resolution::Resolved;
use super::scope::{Scope, ScopeId, GLOBAL_SCOPE};
use crate::ast::Name;
use std::collections::HashMap;

#[derive(Default)]
pub struct Environment<'ast, 'a> {
    scopes: Vec<Scope<'ast, 'a>>,
    pub(in crate::env) resolved_names: HashMap<Name<'a>, Resolved<'ast, 'a>>,
}

impl<'ast, 'a> Environment<'ast, 'a> {
    pub(in crate::env) fn new() -> Self {
        Environment {
            scopes: vec![Scope::global()],
            ..Default::default()
        }
    }

    pub fn find_module<'b>(&self, names: &[&'b str]) -> Option<ScopeId> {
        let mut scope_id = GLOBAL_SCOPE;
        for module_name in names.iter() {
            let scope = self.get_scope(scope_id);
            scope_id = *scope.name_spaces.modules.get(module_name)?;
        }
        Some(scope_id)
    }

    pub fn get_scope_mut(&mut self, scope_id: ScopeId) -> &mut Scope<'ast, 'a> {
        self.scopes.get_mut(scope_id).expect("Invalid scope id")
    }

    pub fn get_scope(&self, scope_id: ScopeId) -> &Scope<'ast, 'a> {
        self.scopes.get(scope_id).expect("Invalid scope id")
    }

    pub(in crate::env) fn add_child_scope(&mut self, parent_id: ScopeId) -> &mut Scope<'ast, 'a> {
        let child_id = self.scopes.len();
        let child = Scope {
            parent: parent_id,
            id: child_id,
            ..Default::default()
        };
        self.scopes.push(child);
        self.scopes.last_mut().expect("Child scope expected")
    }
}
