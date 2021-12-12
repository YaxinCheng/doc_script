use super::super::DeclarationAdder;
use super::construct_env;
use crate::ast::{abstract_tree, AbstractSyntaxTree, Moniker};
use crate::env::scope::GLOBAL_SCOPE;
use crate::env::Environment;
use crate::parser::parse;
use crate::tokenizer::tokenize;

fn prepare_module_path() -> Vec<Vec<&'static str>> {
    vec![vec![]]
}

#[test]
fn test_add_constant() {
    let mut env = construct_env();
    let syntax_tree = abstract_tree(parse(tokenize("const a = b\n")));
    let unresolved_names = kick_off(&mut env, &syntax_tree);
    assert_eq!(unresolved_names, vec![&Moniker::Simple("b")]);
    let expected = syntax_tree
        .compilation_unit
        .declarations
        .last()
        .unwrap()
        .as_constant()
        .unwrap();
    let actual = env
        .get_scope(GLOBAL_SCOPE)
        .name_spaces
        .expressions
        .get(&vec!["a"])
        .expect("Failed to get from namespace")
        .into_constant()
        .unwrap();
    assert!(std::ptr::eq(expected, actual))
}

#[test]
fn test_add_struct_declaration() {
    let mut syntax_trees = vec![abstract_tree(parse(tokenize(
        "struct Text {\n const b = x\n }\n",
    )))];
    let module_path = prepare_module_path();
    let mut env = Environment::construct(&mut syntax_trees, &module_path);
    let unresolved_names = kick_off(&mut env, syntax_trees.last().unwrap());
    assert_eq!(unresolved_names, vec![&Moniker::Simple("x")]);
    let expected = syntax_trees
        .last()
        .unwrap()
        .compilation_unit
        .declarations
        .last()
        .unwrap()
        .as_struct()
        .unwrap();
    let actual = *env
        .get_scope(GLOBAL_SCOPE)
        .name_spaces
        .structs
        .get(&vec!["Text"])
        .expect("Failed to find from namespace");
    assert!(std::ptr::eq(expected, actual))
}

fn kick_off<'ast, 'a>(
    env: &mut Environment<'ast, 'a>,
    syntax_tree: &'ast AbstractSyntaxTree<'a>,
) -> Vec<&'ast Moniker<'a>> {
    let declaration_adder = DeclarationAdder(env);
    let module_path = prepare_module_path();
    let mut unresolved_names = declaration_adder
        .add_from(std::slice::from_ref(syntax_tree), &module_path)
        .into_iter()
        .map(|name| &name.moniker)
        .collect::<Vec<_>>();
    unresolved_names.sort();
    unresolved_names
}
