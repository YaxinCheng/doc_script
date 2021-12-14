mod construction;
mod declaration_resolution;
mod environment;
mod name_resolution;
pub mod scope;

use crate::ast::AbstractSyntaxTree;

pub use environment::Environment;

pub fn construct<'ast, 'a>(
    syntax_trees: &'ast mut [AbstractSyntaxTree<'a>],
    file_names: &'a [String],
) -> Environment<'ast, 'a> {
    let module_paths = file_names
        .iter()
        .map(String::as_str)
        .map(convert_to_module)
        .collect::<Vec<_>>();
    let mut environment = Environment::construct(syntax_trees, &module_paths);
    let unresolved_names =
        declaration_resolution::resolve(&mut environment, syntax_trees, &module_paths);
    name_resolution::resolve(&mut environment, unresolved_names, syntax_trees);
    environment
}

fn convert_to_module(file_name: &str) -> Vec<&str> {
    file_name
        .rsplit_once(std::path::MAIN_SEPARATOR)
        .unwrap_or(("", ""))
        .0
        .split(std::path::MAIN_SEPARATOR)
        .collect()
}

#[cfg(test)]
pub(in crate::env) fn construct_env<'ast, 'a>() -> Environment<'ast, 'a> {
    Environment::new()
}
