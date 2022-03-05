use crate::ast::{abstract_tree, AbstractSyntaxTree, StructDeclaration};
use crate::env::checks::{Error, StructHierarchyChecker};
use crate::parser::parse;
use crate::tests::FormulaSuppress;
use crate::tokenizer::tokenize;
use std::collections::HashSet;

#[test]
fn test_self_reference_cycle_dependency() {
    let res = test_hierarchy_check(
        r#"
        struct A(field: A)
        "#,
    );
    assert!(res.is_err())
}

#[test]
fn test_cycle_reference_between_structs() {
    let res = test_hierarchy_check(
        r#"
    struct A(field: B)
    struct B(field: A)
    "#,
    );
    assert!(res.is_err())
}

#[test]
fn test_reference_to_cycle() {
    let res = test_hierarchy_check(
        r#"
    struct A(field: B)
    struct B(field: A)
    struct C(field: A)
    "#,
    );
    assert!(res.is_err())
}

#[test]
fn test_attribute_reference_to_self() {
    let res = test_hierarchy_check(
        r#"
    struct A(field: Int) {
        const attr = A(3)
    }
    "#,
    );
    assert!(res.is_ok())
}

fn test_hierarchy_check(program: &str) -> Result<(), Error> {
    let formula = FormulaSuppress::all();
    formula.suppress();

    let mut syntax_trees = vec![abstract_tree(parse(tokenize(program)))];
    let module_paths = vec![vec![]];
    let env = crate::env::Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .build();
    let target_declaration =
        get_declaration(syntax_trees.first()).expect("Failed to find declaration");
    let mut white_list = HashSet::new();
    let res = StructHierarchyChecker::with_environment(&env)
        .test_recursively_check(target_declaration, &mut white_list);
    if res.is_ok() {
        assert!(!white_list.is_empty());
    }
    res
}

fn get_declaration<'ast, 'a>(
    syntax_tree: Option<&'ast AbstractSyntaxTree<'a>>,
) -> Option<&'ast StructDeclaration<'a>> {
    syntax_tree?
        .compilation_unit
        .declarations
        .last()?
        .as_struct()
}

#[test]
fn test_with_whitelist() {
    let formula = FormulaSuppress::all();
    formula.suppress();

    let mut syntax_trees = vec![abstract_tree(parse(tokenize(
        r#"
    struct A(field: B)
    struct B(field: A)
    struct C(field: A)
    "#,
    )))];
    let module_paths = vec![vec![]];
    let env = crate::env::Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .build();
    let target_declaration =
        get_declaration(syntax_trees.first()).expect("Failed to find declaration");
    let declaration_of_a = syntax_trees
        .first()
        .unwrap()
        .compilation_unit
        .declarations
        .first()
        .unwrap()
        .as_struct()
        .unwrap();
    let mut white_list = HashSet::new();
    white_list.insert(declaration_of_a);
    let res = StructHierarchyChecker::with_environment(&env)
        .test_recursively_check(target_declaration, &mut white_list);
    assert!(res.is_ok())
}
