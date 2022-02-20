use super::super::type_checking::assignable_checker::AssignableChecker;
use super::super::type_checking::essential_trait;
use crate::ast::{abstract_tree, AbstractSyntaxTree};
use crate::env::checks::tests::try_block;
use crate::env::checks::type_checking::types::Types;
use crate::env::checks::type_checking::TypeChecker;
use crate::env::Environment;
use crate::parser::parse;
use crate::stdlib;
use crate::tokenizer::tokenize;

#[test]
fn test_find_render_trait() {
    let mut syntax_trees = syntax_trees([]);
    let module_paths = module_paths([]);
    let environment = Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .build();
    assert!(essential_trait::render(&environment).is_some())
}

#[test]
fn test_impl_render() {
    let mut syntax_trees = syntax_trees([r#"
struct MyPage {
    const rendered = Page()    
}
"#]);
    let module_paths = module_paths([vec![]]);
    let environment = Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .prelude_std()
        .resolve_names(&syntax_trees)
        .build();

    let source = try_block!(
        Types,
        syntax_trees
            .last()?
            .compilation_unit
            .declarations
            .first()?
            .as_struct()
            .map(Types::Struct)
    );
    let target = essential_trait::render(&environment)
        .map(Types::Trait)
        .unwrap();
    let mut type_checker = TypeChecker::with_environment(&environment);
    let mut checker = AssignableChecker(&mut type_checker);
    assert!(checker.check(&source, &target))
}

fn syntax_trees<const N: usize>(source: [&'static str; N]) -> Vec<AbstractSyntaxTree<'static>> {
    stdlib::compiled_content()
        .into_iter()
        .chain(source.map(tokenize).map(parse).map(abstract_tree))
        .collect()
}

fn module_paths<const N: usize>(module_paths: [Vec<&'static str>; N]) -> Vec<Vec<&'static str>> {
    stdlib::module_paths()
        .into_iter()
        .chain(module_paths)
        .collect()
}
