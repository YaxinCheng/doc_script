use crate::ast::{abstract_tree, Name};
use crate::env::name_resolution::TypeLinker;
use crate::env::scope::{ScopeId, Scoped, GLOBAL_SCOPE};
use crate::env::{declaration_resolution, Environment};
use crate::formula_suppress::FormulaSuppress;
use crate::parser::parse;
use crate::tokenizer::tokenize;

#[test]
fn test_resolve_name_at_current_scope() {
    test_type_linking("struct Empty\n", GLOBAL_SCOPE)
}

#[test]
fn test_resolve_from_struct_body() {
    test_type_linking("struct Empty {\n const shared = Empty()\n }\n", 1)
}

#[test]
fn test_resolve_from_block() {
    test_type_linking("struct Empty {\n const shared = { Empty() }\n }\n", 2)
}

#[test]
fn test_resolve_from_struct_init() {
    test_type_linking("struct Empty {\n const shared = View { Empty() }\n }\n", 2)
}

#[test]
fn test_resolve_from_field() {
    test_type_linking("struct Empty(empty: Empty)\n", GLOBAL_SCOPE);
}

#[test]
#[should_panic]
fn test_unresolvable_type() {
    test_type_linking("struct A(empty: Empty)\n", 1);
}

#[test]
#[should_panic]
fn test_call_on_constant() {
    test_type_linking(
        r#"
    const Empty = 3
    
    const empty = Empty()
    "#,
        GLOBAL_SCOPE,
    );
}

fn test_type_linking(program: &str, source_scope: ScopeId) {
    let formula = FormulaSuppress::all();
    formula.suppress();

    let module_path: Vec<Vec<&str>> = vec![vec![]];
    let mut syntax_trees = vec![abstract_tree(parse(tokenize(program)))];
    let mut env = Environment::builder()
        .add_modules(&module_path)
        .generate_scopes(&mut syntax_trees)
        .build();
    declaration_resolution::resolve(&mut env, &syntax_trees, &module_path);
    let mut name = Name::simple("Empty");
    name.set_scope(source_scope);
    TypeLinker(&mut env).link_types([&name]);
    let actual = *env
        .resolved_names
        .get(&name)
        .unwrap()
        .as_struct()
        .expect("Not struct");
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
