mod struct_hierarchy;
#[cfg(test)]
mod tests;
mod type_checking;

use super::address_hash::hash;
use type_checking::types::Types;

use crate::ast::AbstractSyntaxTree;
use crate::env::Environment;
use struct_hierarchy::StructHierarchyChecker;
use type_checking::TypeChecker;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Struct cycle dependency detected at {0}")]
    StructCycleDependency(String),
}

pub fn check<'ast, 'a>(
    environment: &Environment<'ast, 'a>,
    syntax_trees: &[AbstractSyntaxTree<'a>],
) {
    let mut type_checker = TypeChecker::with_environment(environment);
    #[cfg(test)]
    if !crate::tests::FormulaSuppress::entry_check_suppressed() {
        type_checker.entry_check();
    }
    #[cfg(not(test))]
    type_checker.entry_check();
    type_checker.check(syntax_trees);
    StructHierarchyChecker::with_environment(environment).check(syntax_trees);
}
