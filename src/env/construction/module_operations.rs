use super::{Environment, ScopeId, GLOBAL_SCOPE};
use std::collections::{HashMap, HashSet};

/// Insert module information into the environment
pub(in crate::env::construction) struct ModuleAdder<'ast, 'a, 'env>(
    pub &'env mut Environment<'ast, 'a>,
);

impl<'ast, 'a, 'env> ModuleAdder<'ast, 'a, 'env> {
    pub fn add_modules(mut self, module_paths: &[Vec<&'a str>]) {
        let mut name_to_scope_id = HashMap::<Vec<&'a str>, ScopeId>::new();
        for module_path in module_paths.iter().collect::<HashSet<_>>() {
            self.insert_modules(module_path, &mut name_to_scope_id)
        }
    }

    fn insert_modules(
        &mut self,
        module_path: &[&'a str],
        existing_modules: &mut HashMap<Vec<&'a str>, ScopeId>,
    ) {
        let mut parent_scope_id = GLOBAL_SCOPE;
        let mut scope_name = vec![];
        for &module_name in module_path {
            scope_name.push(module_name);
            if let Some(scope_id) = existing_modules.get(&scope_name) {
                parent_scope_id = *scope_id;
            } else {
                // child scope resides in parent scope, but it cannot access data in its parent scope.
                // So their parent pointer will be pointed to the GLOBAL_SCOPE directly
                let child_scope_id = self.0.add_child_scope(GLOBAL_SCOPE).id;
                let shaded_module = self
                    .0
                    .get_scope_mut(parent_scope_id)
                    .name_spaces
                    .modules
                    .insert(module_name, child_scope_id);
                assert!(
                    shaded_module.is_none(),
                    "Conflicting import for name: {}",
                    module_name
                );
                existing_modules.insert(scope_name.clone(), child_scope_id);
                parent_scope_id = child_scope_id;
            }
        }
    }
}
