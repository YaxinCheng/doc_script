use crate::ast::{abstract_tree, AbstractSyntaxTree};
use crate::env::Environment;
use crate::parser::parse;
use crate::stdlib;
use crate::tokenizer::tokenize;

#[test]
#[should_panic]
fn test_entry_is_not_render() {
    let (mut syntax_trees, module_paths) = build_syntax_trees("const Main = \"hello word\"\n");
    let _ = Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .validate(&syntax_trees)
        .build();
}

#[test]
#[should_panic]
fn test_entry_not_declared() {
    let (mut syntax_trees, module_paths) = build_syntax_trees("const NotMain = \"hello word\"\n");
    let _ = Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .validate(&syntax_trees);
}

#[test]
fn test_entry_follows_render() {
    let (mut syntax_trees, module_paths) = build_syntax_trees("const Main = Doc()\n");
    let _ = Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .validate(&syntax_trees);
}

fn build_syntax_trees(source: &str) -> (Vec<AbstractSyntaxTree>, Vec<Vec<&str>>) {
    let syntax_trees = stdlib::compiled_content()
        .into_iter()
        .chain(std::iter::once(abstract_tree(parse(tokenize(source)))))
        .collect::<Vec<_>>();
    let module_paths = stdlib::module_paths()
        .into_iter()
        .chain(std::iter::once(vec![]))
        .collect::<Vec<_>>();
    (syntax_trees, module_paths)
}
