mod resolution;
mod resolve_helper;
mod resolved;
#[cfg(test)]
mod tests;
mod type_linker;
mod typed_element;

use super::declaration_resolution::UnresolvedNames;
use super::Environment;
use resolution::NameResolver;
pub(in crate::env) use resolve_helper::ResolveHelper;
pub use resolved::Resolved;
use type_linker::TypeLinker;
pub use typed_element::TypedElement;

pub(in crate::env) fn resolve<'ast, 'a>(
    environment: &mut Environment<'ast, 'a>,
    unresolved_names: UnresolvedNames<'ast, 'a>,
) {
    let UnresolvedNames {
        type_names,
        expression_names,
    } = unresolved_names;
    TypeLinker(environment).link_types(type_names);
    NameResolver(environment).resolve_names(expression_names);
}
