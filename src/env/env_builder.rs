use super::checks::StructHierarchyChecker;
use super::construction::{ModuleAdder, ScopeGenerator};
use super::{declaration_resolution, name_resolution, Environment};
use crate::ast::{AbstractSyntaxTree, Declaration};
use std::collections::HashSet;

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

    pub fn add_modules_from_files(
        mut self,
        file_names: &'a [String],
    ) -> EnvironmentBuilder<'ast, 'a, MODULE_ADDED> {
        let module_paths = file_names
            .iter()
            .map(String::as_str)
            .map(Self::convert_to_module);
        self.module_paths.extend(module_paths);
        ModuleAdder(&mut self.environment).add_modules(&self.module_paths);
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
        ModuleAdder(&mut self.environment).add_modules(&self.module_paths);
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
    pub fn generate_scopes(
        mut self,
        syntax_trees: &mut [AbstractSyntaxTree<'a>],
    ) -> EnvironmentBuilder<'ast, 'a, SCOPE_GENERATED> {
        ScopeGenerator(&mut self.environment).generate(syntax_trees, &self.module_paths);
        EnvironmentBuilder {
            environment: self.environment,
            module_paths: self.module_paths,
        }
    }
}

impl<'ast, 'a> EnvironmentBuilder<'ast, 'a, SCOPE_GENERATED> {
    pub fn resolve_names(
        mut self,
        syntax_trees: &'ast [AbstractSyntaxTree<'a>],
    ) -> EnvironmentBuilder<'ast, 'a, NAME_RESOLVED> {
        let unresolved_names = declaration_resolution::resolve(
            &mut self.environment,
            syntax_trees,
            &self.module_paths,
        );
        name_resolution::resolve(&mut self.environment, unresolved_names, syntax_trees);
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
