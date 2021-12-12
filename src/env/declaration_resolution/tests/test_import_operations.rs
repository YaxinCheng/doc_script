use super::super::DeclarationAdder;
use super::super::Importer;
use super::construct_env;
use crate::ast::abstract_tree;
use crate::env::construction::ModuleAdder;
use crate::env::scope::GLOBAL_SCOPE;
use crate::env::Environment;
use crate::parser::parse;
use crate::tokenizer::tokenize;

fn prepare_module_path() -> Vec<Vec<&'static str>> {
    vec![vec![], vec!["test"], vec!["test", "nested"]]
}

macro_rules! import {
    ($env: expr, $code: expr) => {
        let module_paths = prepare_module_path();
        ModuleAdder(&mut $env).add_modules(&module_paths);
        let syntax_trees = vec![
            abstract_tree(parse(tokenize(concat!($code, "\n")))),
            abstract_tree(parse(tokenize("const target = 42\nstruct Empty\n"))),
            abstract_tree(parse(tokenize(
                "const deeper_target = 42\nstruct DeeperEmpty\n",
            ))),
        ];
        DeclarationAdder(&mut $env).add_from(&syntax_trees, &module_paths);
        Importer(&mut $env).import_from(&syntax_trees, &module_paths);
    };
}

#[test]
fn test_single_import_constant() {
    let mut env = construct_env();
    import!(env, "use test.target");
    assert_constant(&env, "target", &["test"]);
}

#[test]
fn test_single_import_deeper_constant() {
    let mut env = construct_env();
    import!(env, "use test.nested.deeper_target");
    assert_constant(&env, "deeper_target", &["test", "nested"]);
}

fn assert_constant(env: &Environment, name: &str, scope_name: &[&str]) {
    let source_scope = env.find_module(scope_name).expect("Cannot find scope name");
    let actual = *env
        .get_scope(GLOBAL_SCOPE)
        .name_spaces
        .expressions
        .get(&vec![name])
        .unwrap()
        .as_constant()
        .unwrap();
    let expected = *env
        .get_scope(source_scope)
        .name_spaces
        .expressions
        .get(&vec![name])
        .unwrap()
        .as_constant()
        .unwrap();
    assert!(std::ptr::eq(actual, expected))
}

#[test]
fn test_single_import_struct() {
    let mut env = construct_env();
    import!(env, "use test.Empty");
    assert_struct(&env, "Empty", &["test"]);
}

#[test]
fn test_single_import_deeper_struct() {
    let mut env = construct_env();
    import!(env, "use test.nested.DeeperEmpty");
    assert_struct(&env, "DeeperEmpty", &["test", "nested"]);
}

fn assert_struct(env: &Environment, name: &str, scope_name: &[&str]) {
    let source_scope = env.find_module(scope_name).expect("Cannot find scope name");
    let actual = *env
        .get_scope(GLOBAL_SCOPE)
        .name_spaces
        .structs
        .get(&vec![name])
        .unwrap();
    let expected = *env
        .get_scope(source_scope)
        .name_spaces
        .structs
        .get(&vec![name])
        .unwrap();
    assert!(std::ptr::eq(actual, expected))
}

#[test]
fn test_multiple_imports() {
    let mut env = construct_env();
    import!(env, "use test.{ Empty, target }");
    assert_struct(&env, "Empty", &["test"]);
    assert_constant(&env, "target", &["test"]);
}

#[test]
fn test_multiple_nested_imports() {
    let mut env = construct_env();
    import!(
        env,
        "use test.{ Empty, target, nested.deeper_target, nested.DeeperEmpty }"
    );
    assert_struct(&env, "Empty", &["test"]);
    assert_constant(&env, "target", &["test"]);
    assert_struct(&env, "DeeperEmpty", &["test", "nested"]);
    assert_constant(&env, "deeper_target", &["test", "nested"]);
}

#[test]
fn test_wildcard_import() {
    let mut env = construct_env();
    import!(env, "use test.*");
    assert!(env
        .get_scope(GLOBAL_SCOPE)
        .name_spaces
        .wildcard_imports
        .contains(&1))
}

#[test]
fn test_single_import_module() {
    let mut env = construct_env();
    import!(env, "use test.nested");
    let actual = env
        .get_scope(GLOBAL_SCOPE)
        .name_spaces
        .modules
        .get("nested")
        .copied();
    let expected = env.find_module(&["test", "nested"]);
    assert_eq!(actual, expected)
}
