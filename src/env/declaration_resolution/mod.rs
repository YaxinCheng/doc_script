use crate::ast::{AbstractSyntaxTree, Name};
use crate::env::Environment;
use std::collections::HashSet;

mod declaration_operations;
mod import_operations;
#[cfg(test)]
mod tests;

#[cfg(not(test))]
use declaration_operations::DeclarationAdder;
#[cfg(not(test))]
use import_operations::Importer;

#[cfg(test)]
pub(in crate::env) use declaration_operations::DeclarationAdder;
#[cfg(test)]
pub(in crate::env) use import_operations::Importer;

#[derive(Default)]
pub(in crate::env) struct UnresolvedNames<'ast, 'a> {
    pub type_names: HashSet<&'ast Name<'a>>,
    pub expression_names: HashSet<&'ast Name<'a>>,
}

pub(in crate::env) fn resolve<'ast, 'a>(
    environment: &mut Environment<'ast, 'a>,
    syntax_trees: &'ast [AbstractSyntaxTree<'a>],
    module_paths: &[Vec<&'a str>],
) -> UnresolvedNames<'ast, 'a> {
    let unresolved = DeclarationAdder(environment).add_from(syntax_trees, module_paths);
    Importer(environment).import_from(syntax_trees, module_paths);
    unresolved
}
