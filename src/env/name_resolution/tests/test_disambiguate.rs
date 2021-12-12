use super::try_block;
use crate::ast::{abstract_tree, ConstantDeclaration, Field, Moniker, StructDeclaration};
use crate::env::name_resolution::NameResolver;
use crate::env::scope::GLOBAL_SCOPE;
use crate::env::Environment;
use crate::parser::parse;
use crate::tokenizer::tokenize;

macro_rules! test_disambiguate {
    ($syntax_trees: expr, $module_paths: expr, $name: expr) => {{
        test_disambiguate!($syntax_trees, $module_paths, $name, GLOBAL_SCOPE)
    }};

    ($syntax_trees: expr, $module_paths: expr, $name: expr, $scope: expr) => {{
        let mut env = Environment::construct(&mut $syntax_trees, &$module_paths);
        env.resolve_declarations(&$syntax_trees, &$module_paths);
        NameResolver::environment(&mut env).test_disambiguate($scope, &$name)
    }};
}

#[test]
fn resolve_module_constant() {
    let mut syntax_trees = vec![
        abstract_tree(parse(tokenize("const a = test.target\n"))),
        abstract_tree(parse(tokenize("const target = 3\n"))),
    ];
    let module_paths = vec![vec![], vec!["test"]];
    let name = Moniker::Qualified(vec!["test", "target"].into_boxed_slice());
    let actual = test_disambiguate!(syntax_trees, module_paths, name)
        .unwrap()
        .into_constant()
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
    let target_name = Moniker::Qualified(vec!["empty", "Empty"].into_boxed_slice());
    let actual = test_disambiguate!(syntax_trees, module_paths, target_name)
        .unwrap()
        .into_struct()
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

#[test]
fn test_constant_field() {
    let mut syntax_trees = vec![
        abstract_tree(parse(tokenize(
            r#"
        use person.Person
        use system.Id
        
        const person = Person(Id(3))
        "#,
        ))),
        abstract_tree(parse(tokenize("struct Person(id: system.Id)\n"))),
        abstract_tree(parse(tokenize("struct Id(digit: Int)\n"))),
    ];
    let module_paths = vec![vec![], vec!["person"], vec!["system"]];
    let target_name = Moniker::Qualified(vec!["person", "id", "digit"].into_boxed_slice());
    let actual = test_disambiguate!(syntax_trees, module_paths, target_name)
        .unwrap()
        .into_field()
        .unwrap();
    let expected = try_block!(
        &Field,
        syntax_trees
            .last()?
            .compilation_unit
            .declarations
            .last()?
            .as_struct()?
            .fields
            .last()
    )
    .unwrap();
    assert!(std::ptr::eq(actual, expected))
}

#[test]
fn test_field_over_package() {
    let mut syntax_trees = vec![
        abstract_tree(parse(tokenize(
            r#"
        struct Test(test: Int)
        const test = Test(3)
        "#,
        ))),
        abstract_tree(parse(tokenize("const test = 5\n"))),
    ];
    let module_paths = vec![vec!["test"], vec!["test", "test"]];
    let target_name = Moniker::Qualified(vec!["test", "test", "test"].into_boxed_slice());
    let actual = test_disambiguate!(syntax_trees, module_paths, target_name)
        .unwrap()
        .into_field()
        .unwrap();
    let expected = try_block!(
        &Field,
        syntax_trees
            .first()?
            .compilation_unit
            .declarations
            .first()?
            .as_struct()?
            .fields
            .first()
    )
    .unwrap();
    assert!(std::ptr::eq(actual, expected))
}

#[test]
fn test_resolve_field_internal() {
    let mut syntax_trees = vec![abstract_tree(parse(tokenize(
        r#"
        struct Test(field: Int)
        "#,
    )))];
    let module_path = vec![vec![]];
    let target_name = Moniker::Qualified(vec!["self", "field"].into_boxed_slice());
    // env not constructed, but we know the struct scope is 1 as only two scopes exist
    let actual = test_disambiguate!(syntax_trees, module_path, target_name, 1)
        .unwrap()
        .into_field()
        .unwrap();
    let expected = try_block!(
        &Field,
        syntax_trees
            .last()?
            .compilation_unit
            .declarations
            .first()?
            .as_struct()?
            .fields
            .first()
    )
    .unwrap();
    assert!(std::ptr::eq(actual, expected))
}

#[test]
fn test_resolve_attribute_internal() {
    let mut syntax_trees = vec![abstract_tree(parse(tokenize(
        r#"
        struct Test {
            const field = 3
        } 
        "#,
    )))];
    let module_path = vec![vec![]];
    let target_name = Moniker::Qualified(vec!["self", "field"].into_boxed_slice());
    // env not constructed, but we know the struct scope is 1 as only two scopes exist
    let actual = test_disambiguate!(syntax_trees, module_path, target_name, 1)
        .unwrap()
        .into_constant()
        .unwrap();
    let expected = try_block!(
        &ConstantDeclaration,
        syntax_trees
            .last()?
            .compilation_unit
            .declarations
            .first()?
            .as_struct()?
            .body
            .attributes
            .first()
    )
    .unwrap();
    assert!(std::ptr::eq(actual, expected))
}
