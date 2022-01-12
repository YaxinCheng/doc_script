use crate::ast::abstract_tree;
use crate::env::construction;
use crate::env::declaration_resolution;
use crate::env::scope::GLOBAL_SCOPE;
use crate::env::Environment;
use crate::parser::parse;
use crate::tokenizer::tokenize;

macro_rules! import {
    ($env: ident, $code: expr) => {
        let module_paths = [vec![], vec!["test"], vec!["test", "nested"]];
        let syntax_trees = [
            abstract_tree(parse(tokenize(concat!($code, "\n")))),
            abstract_tree(parse(tokenize("const target = 42\nstruct Empty\n"))),
            abstract_tree(parse(tokenize(
                "const deeper_target = 42\nstruct DeeperEmpty\n",
            ))),
        ];

        construction::add_modules(&mut $env, &module_paths);
        declaration_resolution::resolve(&mut $env, &syntax_trees, &module_paths);
    };
}

#[test]
fn test_single_import_constant() {
    let mut env = Environment::default();
    import!(env, "use test.target");
    assert_constant(&env, "target", &["test"]);
}

#[test]
fn test_single_import_deeper_constant() {
    let mut env = Environment::default();
    import!(env, "use test.nested.deeper_target");
    assert_constant(&env, "deeper_target", &["test", "nested"]);
}

fn assert_constant(env: &Environment, name: &str, scope_name: &[&str]) {
    let source_scope = env.find_module(scope_name).expect("Cannot find scope name");
    let actual = *env
        .get_scope(GLOBAL_SCOPE)
        .name_spaces
        .declared
        .get(name)
        .unwrap()
        .as_constant()
        .unwrap();
    let expected = *env
        .get_scope(source_scope)
        .name_spaces
        .declared
        .get(name)
        .unwrap()
        .as_constant()
        .unwrap();
    assert!(std::ptr::eq(actual, expected))
}

#[test]
fn test_single_import_struct() {
    let mut env = Environment::default();
    import!(env, "use test.Empty");
    assert_struct(&env, "Empty", &["test"]);
}

#[test]
fn test_single_import_deeper_struct() {
    let mut env = Environment::default();
    import!(env, "use test.nested.DeeperEmpty");
    assert_struct(&env, "DeeperEmpty", &["test", "nested"]);
}

fn assert_struct(env: &Environment, name: &str, scope_name: &[&str]) {
    let source_scope = env.find_module(scope_name).expect("Cannot find scope name");
    let actual = *env
        .get_scope(GLOBAL_SCOPE)
        .name_spaces
        .declared
        .get(name)
        .unwrap();
    let expected = *env
        .get_scope(source_scope)
        .name_spaces
        .declared
        .get(name)
        .unwrap();
    assert_eq!(actual, expected)
}

#[test]
fn test_multiple_imports() {
    let mut env = Environment::default();
    import!(env, "use test.{ Empty, target }");
    assert_struct(&env, "Empty", &["test"]);
    assert_constant(&env, "target", &["test"]);
}

#[test]
fn test_multiple_nested_imports() {
    let mut env = Environment::default();
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
    let mut env = Environment::default();
    import!(env, "use test.*");
    assert!(env
        .get_scope(GLOBAL_SCOPE)
        .name_spaces
        .wildcard_imports
        .contains(&1))
}

#[test]
fn test_single_import_module() {
    let mut env = Environment::default();
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
