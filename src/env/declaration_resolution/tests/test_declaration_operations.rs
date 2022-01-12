use super::super::DeclarationAdder;
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
    let mut env = Environment::default();
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
        .declared
        .get("a")
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
    let mut env = Environment::builder()
        .add_modules(&module_path)
        .generate_scopes(&mut syntax_trees)
        .build();
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
        .declared
        .get("Text")
        .expect("Failed to find from namespace")
        .as_struct()
        .unwrap();
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
        .expression_names
        .into_iter()
        .map(|name| &name.moniker)
        .collect::<Vec<_>>();
    unresolved_names.sort();
    unresolved_names
}
