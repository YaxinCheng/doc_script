use crate::ast::{AbstractSyntaxTree, ConstantDeclaration};

mod expression_evaluator_tests;
mod instance_access_evaluator_tests;
mod instance_resolver_tests;
mod literal_resolver_tests;
mod string_resolver_tests;
mod struct_resolver_tests;

fn get_constant<'ast, 'a>(
    syntax_tree: &'ast AbstractSyntaxTree<'a>,
) -> Option<&'ast ConstantDeclaration<'a>> {
    syntax_tree
        .compilation_unit
        .declarations
        .last()?
        .as_constant()
}
