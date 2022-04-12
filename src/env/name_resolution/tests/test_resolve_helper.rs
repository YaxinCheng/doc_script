use super::super::resolve_helper::ResolveHelper;
use crate::ast::abstract_tree;
use crate::env::declaration_resolution;
use crate::env::Environment;
use crate::formula_suppress::FormulaSuppress;
use crate::parser::parse;
use crate::tokenizer::tokenize;

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
    let formula = FormulaSuppress::all();
    formula.suppress();

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
    declaration_resolution::resolve(&mut env, &syntax_trees, &module_path);
    let helper = ResolveHelper(&env);
    let source_scope_id = env.find_module(&module_path[1]).unwrap();
    let resolved = helper
        .resolve(source_scope_id, "name")
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
    let formula = FormulaSuppress::all();
    formula.suppress();

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
    declaration_resolution::resolve(&mut env, &syntax_trees, &module_path);
    let helper = ResolveHelper(&env);
    let source_scope_id = env.find_module(&["test", "target"]).unwrap();
    let unresolved = helper.resolve(source_scope_id, "title");
    assert!(unresolved.is_none())
}

#[test]
fn test_shaded_name() {
    let formula = FormulaSuppress::all();
    formula.suppress();

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
    declaration_resolution::resolve(&mut env, &syntax_trees, &module_path);
    let helper = ResolveHelper(&env);
    let source_scope_id = env.find_module(&["test", "target"]).unwrap();
    let resolved = helper
        .resolve(source_scope_id, "name")
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

#[test]
#[should_panic]
fn test_ambiguous_wildcard_imports() {
    let module_path = [vec!["first"], vec!["second"], vec!["target"]];
    let mut syntax_trees = [
        abstract_tree(parse(tokenize("const name = \"Test Name\"\n"))),
        abstract_tree(parse(tokenize("const name = 3\n"))),
        abstract_tree(parse(tokenize(
            r#"
        use first.*
        use second.*
        const target = name
        "#,
        ))),
    ];
    let _ = Environment::builder()
        .add_modules(&module_path)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .build();
}

#[test]
#[should_panic]
fn test_cycling_wildcard_imports() {
    let module_paths = [vec!["first"], vec!["second"]];
    let mut syntax_trees = [
        abstract_tree(parse(tokenize("const name = second_name\n"))),
        abstract_tree(parse(tokenize("const not_second_name = 42\n"))),
    ];
    let _ = Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .build();
}
