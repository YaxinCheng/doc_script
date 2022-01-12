mod module_operations;
mod scope_operations;
mod subdivide_struct_init;
#[cfg(test)]
mod tests;

use super::scope::*;
use super::Environment;
use crate::ast::AbstractSyntaxTree;

pub(in crate::env) fn add_modules<'ast, 'a>(
    environment: &mut Environment<'ast, 'a>,
    module_paths: &[Vec<&'a str>],
) {
    module_operations::ModuleAdder(environment).add_modules(module_paths)
}

pub(in crate::env) fn generate_scope<'ast, 'a>(
    environment: &mut Environment<'ast, 'a>,
    syntax_trees: &mut [AbstractSyntaxTree<'a>],
    module_paths: &[Vec<&'a str>],
) {
    scope_operations::ScopeGenerator(environment).generate(syntax_trees, module_paths)
}
