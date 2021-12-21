use super::checks::StructHierarchyChecker;
use super::construction::{ModuleAdder, ScopeGenerator};
use super::{declaration_resolution, name_resolution, Environment};
use crate::ast::{AbstractSyntaxTree, Declaration};
use std::collections::HashSet;

#[derive(Default)]
pub struct EnvironmentBuilder<'ast, 'a> {
    environment: Environment<'ast, 'a>,
    module_paths: Vec<Vec<&'a str>>,
}

impl<'ast, 'a> EnvironmentBuilder<'ast, 'a> {
    pub fn add_modules_from_files(mut self, file_names: &'a [String]) -> Self {
        self.module_paths = file_names
            .iter()
            .map(String::as_str)
            .map(Self::convert_to_module)
            .collect();
        ModuleAdder(&mut self.environment).add_modules(&self.module_paths);
        self
    }

    fn convert_to_module(file_name: &str) -> Vec<&str> {
        file_name
            .rsplit_once(std::path::MAIN_SEPARATOR)
            .unwrap_or(("", ""))
            .0
            .split(std::path::MAIN_SEPARATOR)
            .collect()
    }

    pub fn generate_scopes(mut self, syntax_trees: &mut [AbstractSyntaxTree<'a>]) -> Self {
        ScopeGenerator(&mut self.environment).generate(syntax_trees, &self.module_paths);
        self
    }

    pub fn resolve_names(mut self, syntax_trees: &'ast [AbstractSyntaxTree<'a>]) -> Self {
        let unresolved_names = declaration_resolution::resolve(
            &mut self.environment,
            syntax_trees,
            &self.module_paths,
        );
        name_resolution::resolve(&mut self.environment, unresolved_names, syntax_trees);
        self
    }

    pub fn validate(self, syntax_trees: &'ast [AbstractSyntaxTree<'a>]) -> Self {
        let mut white_list = HashSet::new();
        syntax_trees
            .iter()
            .flat_map(|syntax_tree| &syntax_tree.compilation_unit.declarations)
            .filter_map(|declaration| match declaration {
                Declaration::Struct(struct_declaration) => Some(struct_declaration),
                _ => None,
            })
            .for_each(|struct_declaration| {
                StructHierarchyChecker::with_environment(&self.environment)
                    .recursively_check(struct_declaration, &mut white_list)
                    .expect("Cycle reference found in struct declaration")
            });
        self
    }

    pub fn build(self) -> Environment<'ast, 'a> {
        self.environment
    }

    #[cfg(test)]
    pub fn add_modules(mut self, module_paths: &[Vec<&'a str>]) -> Self {
        self.module_paths = module_paths.into();
        ModuleAdder(&mut self.environment).add_modules(&self.module_paths);
        self
    }
}
