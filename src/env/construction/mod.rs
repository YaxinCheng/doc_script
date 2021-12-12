mod module_operations;
mod scope_operations;
#[cfg(test)]
mod tests;

use super::scope::*;
use super::Environment;
use crate::ast::AbstractSyntaxTree;
#[cfg(not(test))]
use module_operations::ModuleAdder;
#[cfg(test)]
pub(in crate::env) use module_operations::ModuleAdder;
use scope_operations::ScopeGenerator;

impl<'ast, 'a> Environment<'ast, 'a> {
    pub fn construct(
        syntax_trees: &mut [AbstractSyntaxTree<'a>],
        module_paths: &[Vec<&'a str>],
    ) -> Environment<'ast, 'a> {
        let mut environment = Environment::new();
        ModuleAdder(&mut environment).add_modules(module_paths);
        ScopeGenerator(&mut environment).generate(syntax_trees, module_paths);
        environment
    }
}
