use super::Environment;
use crate::env::scope::DeclaredElement;

pub struct ModuleVerifier<'env, 'ast, 'a>(&'env Environment<'ast, 'a>);

const STD_ESSENTIAL: &[&str] = &["std", "essential"];
const STD_ESSENTIAL_RENDER: &[&str] = &["std", "essential", "Render"];

impl<'env, 'ast, 'a> ModuleVerifier<'env, 'ast, 'a> {
    pub fn with_environment(environment: &'env Environment<'ast, 'a>) -> Self {
        Self(environment)
    }

    pub fn in_essential<D: Into<DeclaredElement<'ast, 'a>>>(&self, declared: D) -> bool {
        self.is_in_module(declared.into(), STD_ESSENTIAL)
    }

    pub fn in_render<D: Into<DeclaredElement<'ast, 'a>>>(&self, declared: D) -> bool {
        self.is_in_module(declared.into(), STD_ESSENTIAL_RENDER)
    }

    fn is_in_module(&self, declared: DeclaredElement<'ast, 'a>, module_name: &[&str]) -> bool {
        let name = declared.name();
        self.0
            .find_module(module_name)
            .map(|scope_id| self.0.get_scope(scope_id))
            .and_then(|scope| scope.name_spaces.declared.get(name))
            .map(|found| found == &declared)
            .unwrap_or_default()
    }
}
