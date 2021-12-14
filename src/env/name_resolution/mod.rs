// mod disambiguate;
mod address_hash;
mod resolution;
mod resolve_helper;
mod resolved;
#[cfg(test)]
mod tests;
mod type_checking;
mod type_linker;
mod typed_element;
pub mod types;

use super::declaration_resolution::UnresolvedNames;
use super::Environment;
use crate::ast::AbstractSyntaxTree;
use resolution::NameResolver;
pub use resolved::Resolved;
use type_checking::TypeChecker;
use type_linker::TypeLinker;

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
    let instance_fields = NameResolver(environment).resolve_names(expression_names);
    TypeChecker::new(instance_fields).resolve_and_check(environment, syntax_trees);
}

pub(in crate::env::name_resolution) fn split_first_component<'b, 's>(
    name_components: &'b [&'s str],
) -> (&'b [&'s str], &'b [&'s str]) {
    if name_components.first() == Some(&"self") {
        name_components.split_at(2)
    } else {
        let (first, remaining) = name_components.split_first().expect("Failed to split");
        (std::slice::from_ref(first), remaining)
    }
}
