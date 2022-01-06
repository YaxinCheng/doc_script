use super::super::resolve_helper::ResolveHelper;
use crate::ast::{abstract_tree, Moniker};
use crate::env::declaration_resolution::{DeclarationAdder, Importer};
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
    let mut env = Environment::builder()
        .add_modules(&module_path)
        .generate_scopes(&mut syntax_trees)
        .build();
    prepare_env!(env, &syntax_trees, &module_path);
    let helper = ResolveHelper(&env);
    let source_scope_id = env.find_module(&module_path[1]).unwrap();
    let resolved = helper
        .resolve(source_scope_id, &Moniker::Simple("name"))
        .expect("Failed to resolve");
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
    let mut env = Environment::builder()
        .add_modules(&module_path)
        .generate_scopes(&mut syntax_trees)
        .build();
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
    let mut env = Environment::builder()
        .add_modules(&module_path)
        .generate_scopes(&mut syntax_trees)
        .build();
    prepare_env!(env, &syntax_trees, &module_path);
    let helper = ResolveHelper(&env);
    let source_scope_id = env.find_module(&["test", "target"]).unwrap();
    let resolved = helper
        .resolve(source_scope_id, &Moniker::Simple("name"))
        .expect("Failed to resolve");
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
