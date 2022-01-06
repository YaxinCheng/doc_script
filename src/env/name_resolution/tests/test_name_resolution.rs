use crate::ast::{abstract_tree, ConstantDeclaration, Name, StructDeclaration};
use crate::env::name_resolution::tests::try_block;
use crate::env::name_resolution::{NameResolver, TypeLinker};
use crate::env::scope::{Scoped, GLOBAL_SCOPE};
use crate::env::{declaration_resolution, Environment};
use crate::parser::parse;
use crate::tokenizer::tokenize;

macro_rules! test_resolve {
    ($syntax_trees: ident, $module_paths: ident) => {{
        let mut env = Environment::builder()
            .add_modules(&$module_paths)
            .generate_scopes(&mut $syntax_trees)
            .build();
        let names = declaration_resolution::resolve(&mut env, &$syntax_trees, &$module_paths);
        TypeLinker(&mut env).link_types(names.type_names);
        NameResolver(&mut env).resolve_names(names.expression_names);
        env.resolved_names
    }};
}

#[test]
fn resolve_module_constant() {
    let mut syntax_trees = vec![
        abstract_tree(parse(tokenize("const a = test.target\n"))),
        abstract_tree(parse(tokenize("const target = 3\n"))),
    ];
    let module_paths = vec![vec![], vec!["test"]];
    let mut name = Name::qualified(vec!["test", "target"]);
    name.set_scope(GLOBAL_SCOPE);
    let resolved = test_resolve!(syntax_trees, module_paths);
    let actual = try_block!(
        &ConstantDeclaration,
        resolved.get(&name)?.as_constant().copied()
    )
    .unwrap();
    let expected = try_block!(
        &ConstantDeclaration,
        syntax_trees
            .last()?
            .compilation_unit
            .declarations
            .last()?
            .as_constant()
    )
    .unwrap();
    assert!(std::ptr::eq(actual, expected))
}

#[test]
fn resolve_module_struct() {
    let mut syntax_trees = vec![
        abstract_tree(parse(tokenize("const a = empty.Empty()\n"))),
        abstract_tree(parse(tokenize("struct Empty\n"))),
    ];
    let module_paths = vec![vec![], vec!["empty"]];
    let mut target_name = Name::qualified(vec!["empty", "Empty"]);
    target_name.set_scope(GLOBAL_SCOPE);
    let resolved_names = test_resolve!(syntax_trees, module_paths);
    let actual = try_block!(
        &StructDeclaration,
        resolved_names.get(&target_name)?.as_struct().copied()
    )
    .unwrap();
    let expected = try_block!(
        &StructDeclaration,
        syntax_trees
            .last()?
            .compilation_unit
            .declarations
            .last()?
            .as_struct()
    )
    .unwrap();
    assert!(std::ptr::eq(actual, expected));
}
