pub use super::name_resolution::Resolved;
use super::scope::{Scope, ScopeId, GLOBAL_SCOPE};
use crate::ast::Name;
use crate::env::EnvironmentBuilder;
use std::collections::HashMap;

pub struct Environment<'ast, 'a> {
    scopes: Vec<Scope<'ast, 'a>>,
    pub resolved_names: HashMap<Name<'a>, Resolved<'ast, 'a>>,
}

impl<'ast, 'a> Default for Environment<'ast, 'a> {
    fn default() -> Self {
        Environment {
            scopes: vec![Scope::global()],
            resolved_names: HashMap::new(),
        }
    }
}

impl<'ast, 'a> Environment<'ast, 'a> {
    pub fn builder() -> EnvironmentBuilder<'ast, 'a> {
        EnvironmentBuilder::new()
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
