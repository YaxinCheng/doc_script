use super::types::Types;
use crate::env::module_verifier::ModuleVerifier;
use crate::env::Environment;

pub struct RenderImplChecker<'ast, 'a, 'env>(pub &'env Environment<'ast, 'a>);

impl<'ast, 'a, 'env> RenderImplChecker<'ast, 'a, 'env> {
    pub fn check(self, source: &Types<'ast, 'a>, target: &Types<'ast, 'a>) -> bool {
        self.is_in_essential(target) && self.is_in_essential_render(source)
    }

    /// Checks if the give type is pointing to an essential trait
    fn is_in_essential(&self, source_type: &Types<'ast, 'a>) -> bool {
        let source_type = match source_type {
            Types::Trait(trait_type) => *trait_type,
            _ => return false,
        };
        ModuleVerifier::with_environment(self.0).in_essential(source_type)
    }

    fn is_in_essential_render(&self, source_type: &Types<'ast, 'a>) -> bool {
        let verifier = ModuleVerifier::with_environment(self.0);
        match source_type {
            Types::Trait(trait_type) => verifier.in_render(*trait_type),
            Types::Struct(struct_type) => verifier.in_render(*struct_type),
            _ => false,
        }
    }
}

#[cfg(test)]
mod essential_trait_checker_tests {
    use super::super::types::Types;
    use super::RenderImplChecker;
    use crate::env::Environment;
    use crate::stdlib;

    #[test]
    fn test_is_not_essential_trait() {
        let env = Environment::builder().build();
        assert!(!RenderImplChecker(&env).is_in_essential(&Types::Int));
    }

    #[test]
    fn test_is_essential_trait() {
        let mut syntax_trees = stdlib::compiled_content();
        let module_paths = stdlib::module_paths();
        let env = Environment::builder()
            .add_modules(&module_paths)
            .generate_scopes(&mut syntax_trees)
            .resolve_names(&syntax_trees)
            .build();
        let std_essential = env
            .find_module(&["std", "essential"])
            .map(|scope_id| env.get_scope(scope_id))
            .expect("Cannot find std.essential");
        let render_trait = std_essential
            .name_spaces
            .declared
            .get("Render")
            .expect("Cannot find std.essential.Render")
            .into_trait()
            .expect("Render is not trait");
        assert!(RenderImplChecker(&env).is_in_essential(&Types::Trait(render_trait)))
    }

    #[test]
    fn test_impl_essential_trait() {
        let mut syntax_trees = stdlib::compiled_content();
        let module_paths = stdlib::module_paths();
        let env = Environment::builder()
            .add_modules(&module_paths)
            .generate_scopes(&mut syntax_trees)
            .resolve_names(&syntax_trees)
            .build();
        let std_essential_render = env
            .find_module(&["std", "essential", "Render"])
            .map(|scope_id| env.get_scope(scope_id))
            .expect("Cannot find std.essential.Render");
        let doc_type = std_essential_render
            .name_spaces
            .declared
            .get("Doc")
            .expect("Cannot find Doc type")
            .into_struct()
            .expect("Doc is not struct");
        assert!(RenderImplChecker(&env).is_in_essential_render(&Types::Struct(doc_type)))
    }
}
