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

impl<'ast, 'a> Environment<'ast, 'a> {
    pub(in crate::env) fn resolve_declarations(
        &mut self,
        syntax_trees: &'ast [AbstractSyntaxTree<'a>],
        module_paths: &[Vec<&'a str>],
    ) -> HashSet<&'ast Name<'a>> {
        let unresolved_names = DeclarationAdder(self).add_from(syntax_trees, module_paths);
        Importer(self).import_from(syntax_trees, module_paths);
        unresolved_names
    }
}
