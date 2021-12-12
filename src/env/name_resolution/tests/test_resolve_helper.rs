use super::super::resolve_helper::ResolveHelper;
use crate::ast::{abstract_tree, Moniker};
use crate::env::declaration_resolution::{DeclarationAdder, Importer};
use crate::env::scope::{ScopeId, GLOBAL_SCOPE};
use crate::env::Environment;
use crate::parser::parse;
use crate::tokenizer::tokenize;

macro_rules! prepare_env {
    ($env: expr, $syntax_trees: expr, $module_paths: expr) => {
        DeclarationAdder(&mut $env).add_from($syntax_trees, $module_paths);
        Importer(&mut $env).import_from($syntax_trees, $module_paths);
    };
}

#[test]
fn test_resolve_name_at_current_scope() {
    test_resolve_struct_empty("struct Empty\n", GLOBAL_SCOPE)
}

#[test]
fn test_resolve_from_struct_body() {
    test_resolve_struct_empty("struct Empty {\n const shared = Empty()\n }\n", 1)
}

#[test]
fn test_resolve_from_block() {
    test_resolve_struct_empty("struct Empty {\n const shared = { Empty() }\n }\n", 2)
}

#[test]
fn test_resolve_from_struct_init() {
    test_resolve_struct_empty("struct Empty {\n const shared = View { Empty() }\n }\n", 2)
}

fn test_resolve_struct_empty(program: &str, source_scope: ScopeId) {
    let module_path: Vec<Vec<&str>> = vec![vec![]];
    let mut syntax_trees = vec![abstract_tree(parse(tokenize(program)))];
    let mut env = Environment::construct(&mut syntax_trees, &module_path);
    prepare_env!(env, &syntax_trees, &module_path);
    let helper = ResolveHelper(&env);
    let (resolved, scope) = helper
        .resolve(source_scope, &Moniker::Simple("Empty"))
        .expect("Failed to resolve");
    assert_eq!(scope, GLOBAL_SCOPE);
    let actual = resolved.into_struct().expect("Not struct");
    let expected = syntax_trees
        .last()
        .unwrap()
        .compilation_unit
        .declarations
        .last()
        .unwrap()
        .as_struct()
        .unwrap();
    assert!(std::ptr::eq(actual, expected))
}

#[test]
fn test_resolve_constant_from_block() {
    test_resolve_from_wildcard_import([
        "const name = \"Test Name\"\n",
        r#"use test.source.*
           const test = { name }
        "#,
    ])
}

#[test]
fn test_resolve_constant_from_struct_init() {
    test_resolve_from_wildcard_import([
        "const name = \"Test Name\"\n",
        r#"use test.source.*
           const test = View { Text(name) }
        "#,
    ])
}

#[test]
fn test_resolve_constant_from_struct_body() {
    test_resolve_from_wildcard_import([
        "const name = \"Test Name\"\n",
        r#"use test.source.*
           struct View {
                const title = name
           }
        "#,
    ])
}

fn test_resolve_from_wildcard_import(programs: [&'static str; 2]) {
    let module_path: Vec<Vec<&str>> = vec![vec!["test", "source"], vec!["test", "target"]];
    let mut syntax_trees = programs
        .into_iter()
        .map(tokenize)
        .map(parse)
        .map(abstract_tree)
        .collect::<Vec<_>>();
    let mut env = Environment::construct(&mut syntax_trees, &module_path);
    prepare_env!(env, &syntax_trees, &module_path);
    let helper = ResolveHelper(&env);
    let source_scope_id = env.find_module(&module_path[1]).unwrap();
    let (resolved, scope) = helper
        .resolve(source_scope_id, &Moniker::Simple("name"))
        .expect("Failed to resolve");
    assert_eq!(scope, source_scope_id);
    let actual = resolved.into_constant().expect("Not constant");
    let expected = syntax_trees
        .first()
        .unwrap()
        .compilation_unit
        .declarations
        .first()
        .unwrap()
        .as_constant()
        .unwrap();
    assert!(std::ptr::eq(actual, expected))
}

#[test]
fn test_unresolvable_name() {
    let module_path: Vec<Vec<&str>> = vec![vec!["test", "source"], vec!["test", "target"]];
    let mut syntax_trees = vec![
        abstract_tree(parse(tokenize("const name = \"Test Name\"\n"))),
        abstract_tree(parse(tokenize(
            r#"
        use test.source.*
        const name = title
        "#,
        ))),
    ];
    let mut env = Environment::construct(&mut syntax_trees, &module_path);
    prepare_env!(env, &syntax_trees, &module_path);
    let helper = ResolveHelper(&env);
    let source_scope_id = env.find_module(&["test", "target"]).unwrap();
    let unresolved = helper.resolve(source_scope_id, &Moniker::Simple("title"));
    assert!(unresolved.is_none())
}

#[test]
fn test_shaded_name() {
    let module_path: Vec<Vec<&str>> = vec![vec!["test", "source"], vec!["test", "target"]];
    let mut syntax_trees = vec![
        abstract_tree(parse(tokenize("const name = \"Test Name\"\n"))),
        abstract_tree(parse(tokenize(
            r#"
        use test.source.*
        const name = "Debug Name"
        "#,
        ))),
    ];
    let mut env = Environment::construct(&mut syntax_trees, &module_path);
    prepare_env!(env, &syntax_trees, &module_path);
    let helper = ResolveHelper(&env);
    let source_scope_id = env.find_module(&["test", "target"]).unwrap();
    let (resolved, scope) = helper
        .resolve(source_scope_id, &Moniker::Simple("name"))
        .expect("Failed to resolve");
    assert_eq!(scope, source_scope_id);
    let actual = resolved.into_constant().expect("not constant");
    let expected = syntax_trees
        .last()
        .unwrap()
        .compilation_unit
        .declarations
        .last()
        .unwrap()
        .as_constant()
        .unwrap();
    assert!(std::ptr::eq(actual, expected))
}
