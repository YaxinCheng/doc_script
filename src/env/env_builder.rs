use super::checks::StructHierarchyChecker;
use super::construction;
use super::{declaration_resolution, name_resolution, Environment};
use crate::ast::AbstractSyntaxTree;
use crate::env::checks::type_checking::TypeChecker;

const CONSTRUCTED: usize = 0;
const MODULE_ADDED: usize = 1;
const SCOPE_GENERATED: usize = 2;
const NAME_RESOLVED: usize = 3;
const VALIDATED: usize = 4;

pub struct EnvironmentBuilder<'ast, 'a, const STATE: usize> {
    environment: Environment<'ast, 'a>,
    module_paths: Vec<Vec<&'a str>>,
}

impl<'ast, 'a> EnvironmentBuilder<'ast, 'a, CONSTRUCTED> {
    pub(in crate::env) fn new() -> Self {
        EnvironmentBuilder {
            environment: Environment::default(),
            module_paths: vec![],
        }
    }

    /// Add module information to environment based on file paths
    ///
    /// # Parameters
    /// * `paths` - a slice of paths to source files
    ///
    /// # Note
    /// file path starts from src folder of the project,
    /// then each folder will become a module.
    ///
    /// For example, path `src/models/tree/tree.ds` generate
    /// modules `models` and `models.tree`
    pub fn add_modules_from_paths(
        mut self,
        paths: &'a [String],
    ) -> EnvironmentBuilder<'ast, 'a, MODULE_ADDED> {
        let module_paths = paths
            .iter()
            .map(String::as_str)
            .map(Self::convert_to_module);
        self.module_paths.extend(module_paths);
        construction::add_modules(&mut self.environment, &self.module_paths);
        EnvironmentBuilder {
            environment: self.environment,
            module_paths: self.module_paths,
        }
    }

    #[cfg(test)]
    pub fn add_modules(
        mut self,
        module_paths: &[Vec<&'a str>],
    ) -> EnvironmentBuilder<'ast, 'a, MODULE_ADDED> {
        self.module_paths.extend_from_slice(module_paths);
        construction::add_modules(&mut self.environment, &self.module_paths);
        EnvironmentBuilder {
            environment: self.environment,
            module_paths: self.module_paths,
        }
    }

    fn convert_to_module(file_name: &str) -> Vec<&str> {
        file_name
            .rsplit_once(std::path::MAIN_SEPARATOR)
            .unwrap_or(("", ""))
            .0
            .split(std::path::MAIN_SEPARATOR)
            .collect()
    }
}

impl<'ast, 'a> EnvironmentBuilder<'ast, 'a, MODULE_ADDED> {
    /// Insert scope information into environment and mark them on the syntax tree.
    ///
    /// # Parameters
    /// `syntax_trees` - a mutable slice of syntax trees
    ///
    /// # Note
    /// Starting from the global scope, a new scope will be added
    /// whenever a block/struct body is met
    ///
    /// The `syntax_trees` are taken as mutable because the scope id should be stored
    /// with some scoped elements, like `Expression::SelfRef` or `Name`
    pub fn generate_scopes(
        mut self,
        syntax_trees: &mut [AbstractSyntaxTree<'a>],
    ) -> EnvironmentBuilder<'ast, 'a, SCOPE_GENERATED> {
        construction::generate_scope(&mut self.environment, syntax_trees, &self.module_paths);
        EnvironmentBuilder {
            environment: self.environment,
            module_paths: self.module_paths,
        }
    }
}

impl<'ast, 'a> EnvironmentBuilder<'ast, 'a, SCOPE_GENERATED> {
    ///
    pub fn resolve_names(
        mut self,
        syntax_trees: &'ast [AbstractSyntaxTree<'a>],
    ) -> EnvironmentBuilder<'ast, 'a, NAME_RESOLVED> {
        let unresolved_names = declaration_resolution::resolve(
            &mut self.environment,
            syntax_trees,
            &self.module_paths,
        );
        name_resolution::resolve(&mut self.environment, unresolved_names);
        EnvironmentBuilder {
            environment: self.environment,
            module_paths: self.module_paths,
        }
    }
}

impl<'ast, 'a> EnvironmentBuilder<'ast, 'a, NAME_RESOLVED> {
    pub fn validate(
        self,
        syntax_trees: &'ast [AbstractSyntaxTree<'a>],
    ) -> EnvironmentBuilder<'ast, 'a, VALIDATED> {
        TypeChecker::with_environment(&self.environment).check(syntax_trees);
        StructHierarchyChecker::with_environment(&self.environment).check(syntax_trees);
        EnvironmentBuilder {
            environment: self.environment,
            module_paths: self.module_paths,
        }
    }
}

impl<'ast, 'a, const STATE: usize> EnvironmentBuilder<'ast, 'a, STATE> {
    pub fn build(self) -> Environment<'ast, 'a> {
        self.environment
    }
}
