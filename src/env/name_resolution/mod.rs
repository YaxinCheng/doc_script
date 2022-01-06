mod resolution;
mod resolve_helper;
mod resolved;
#[cfg(test)]
mod tests;
mod type_checking;
mod type_linker;
mod typed_element;
pub mod types;
pub use type_checking::type_resolver;

use super::declaration_resolution::UnresolvedNames;
use super::Environment;
use crate::ast::AbstractSyntaxTree;
use resolution::NameResolver;
pub use resolved::Resolved;
use type_checking::TypeChecker;
use type_linker::TypeLinker;
pub use typed_element::TypedElement;

pub(in crate::env) fn resolve<'ast, 'a>(
    environment: &mut Environment<'ast, 'a>,
    unresolved_names: UnresolvedNames<'ast, 'a>,
    syntax_trees: &'ast [AbstractSyntaxTree<'a>],
) {
    let UnresolvedNames {
        type_names,
        expression_names,
    } = unresolved_names;
    TypeLinker(environment).link_types(type_names);
    NameResolver(environment).resolve_names(expression_names);
    TypeChecker::with_environment(environment).check(syntax_trees);
}
