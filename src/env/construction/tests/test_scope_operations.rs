use super::super::scope_operations::ScopeGenerator;
use crate::ast::{abstract_tree, AbstractSyntaxTree, ConstantDeclaration, StructDeclaration};
use crate::env::scope::Scoped;
use crate::env::Environment;
use crate::parser::parse;
use crate::tokenizer::tokenize;

fn prepare_module_paths() -> Vec<Vec<&'static str>> {
    vec![vec![]]
}

#[test]
fn test_scope_for_block() {
    let mut env = Environment::default();
    let mut ast = vec![abstract_tree(parse(tokenize("const a = { 3 }\n")))];
    let module_paths = prepare_module_paths();
    ScopeGenerator(&mut env).generate(&mut ast, &module_paths);
    let constant = constants(ast.pop().unwrap()).pop().unwrap();
    let block = constant.value.into_block().unwrap();
    assert_eq!(block.scope(), 1);
}

#[test]
fn test_scope_for_struct_init_content() {
    let mut env = Environment::default();
    let mut ast = vec![abstract_tree(parse(tokenize(
        "const a = View { Text(\"\") }\n",
    )))];
    let module_paths = prepare_module_paths();
    ScopeGenerator(&mut env).generate(&mut ast, &module_paths);
    let constant = constants(ast.pop().unwrap()).pop().unwrap();
    let mut init_content = constant.value.into_struct_init().unwrap().2.unwrap();
    let text = init_content.0.pop().unwrap().into_struct_init().unwrap().0;
    assert_eq!(text.scope(), 1);
}

#[test]
fn test_scope_for_struct_definition() {
    let mut env = Environment::default();
    let mut ast = vec![abstract_tree(parse(tokenize(
        "struct Test { const a = 3\n }\n",
    )))];
    let module_paths = prepare_module_paths();
    ScopeGenerator(&mut env).generate(&mut ast, &module_paths);
    let struct_definition = struct_definitions(ast.pop().unwrap()).pop().unwrap();
    let body = struct_definition.body.expect("Body is empty");
    assert_eq!(body.scope(), 1)
}

fn constants(syntax_tree: AbstractSyntaxTree) -> Vec<ConstantDeclaration> {
    syntax_tree
        .compilation_unit
        .declarations
        .into_iter()
        .filter_map(|declaration| declaration.into_constant().ok())
        .collect()
}

fn struct_definitions(syntax_tree: AbstractSyntaxTree) -> Vec<StructDeclaration> {
    syntax_tree
        .compilation_unit
        .declarations
        .into_iter()
        .filter_map(|declaration| declaration.into_struct().ok())
        .collect()
}
