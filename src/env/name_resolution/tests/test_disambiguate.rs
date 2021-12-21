use super::try_block;
use crate::ast::{abstract_tree, Field, Name};
use crate::env::scope::{Scoped, GLOBAL_SCOPE};
use crate::env::Environment;
use crate::parser::parse;
use crate::tokenizer::tokenize;

macro_rules! test_disambiguate {
    ($syntax_trees: expr, $module_paths: expr, $name: expr) => {{
        test_disambiguate!($syntax_trees, $module_paths, $name, GLOBAL_SCOPE)
    }};

    ($syntax_trees: expr, $module_paths: expr, $name: expr, $scope: expr) => {{
        let mut env = Environment::builder()
            .add_modules(&$module_paths)
            .generate_scopes(&mut $syntax_trees)
            .resolve_names(&$syntax_trees)
            .build();
        env.resolved_names.remove(&$name)
    }};
}

#[test]
fn test_constant_field() {
    let mut syntax_trees = vec![
        abstract_tree(parse(tokenize(
            r#"
        use person.Person
        use system.Id
        
        const person = Person(Id(3))
        const val = person.id.digit
        "#,
        ))),
        abstract_tree(parse(tokenize("struct Person(id: system.Id)\n"))),
        abstract_tree(parse(tokenize("struct Id(digit: Int)\n"))),
    ];
    let module_paths = vec![vec![], vec!["person"], vec!["system"]];
    let mut target_name = Name::qualified(vec!["person", "id", "digit"]);
    target_name.set_scope(GLOBAL_SCOPE);
    let actual = *test_disambiguate!(syntax_trees, module_paths, target_name)
        .unwrap()
        .into_instance_access()
        .unwrap()
        .last()
        .unwrap()
        .as_field()
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
        abstract_tree(parse(tokenize("const value = test.test.test\n"))),
        abstract_tree(parse(tokenize(
            r#"
        struct Test(test: Int)
        const test = Test(3)
        "#,
        ))),
        abstract_tree(parse(tokenize("const test = 5\n"))),
    ];
    let module_paths = vec![vec![], vec!["test"], vec!["test", "test"]];
    let mut target_name = Name::qualified(vec!["test", "test", "test"]);
    target_name.set_scope(GLOBAL_SCOPE);
    let actual = *test_disambiguate!(syntax_trees, module_paths, target_name)
        .unwrap()
        .into_instance_access()
        .unwrap()
        .last()
        .unwrap()
        .as_field()
        .unwrap();
    let expected = try_block!(
        &Field,
        syntax_trees[1]
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
fn test_field_access_internal() {
    let program = r#"
        struct Id(number: Int)
        struct Person(id: Id) {
            const identifier = self.id.number
        }
        "#;
    test_field_access_self_id_number(program)
}

#[test]
fn test_attribute_access_internal() {
    let program = r#"
        struct Id(number: Int) 
        struct person {
            const id = Id(3)
            const identifier = self.id.number
        }
    "#;
    test_field_access_self_id_number(program)
}

fn test_field_access_self_id_number(program: &str) {
    let mut syntax_trees = vec![abstract_tree(parse(tokenize(program)))];
    let module_paths = vec![vec![]];
    let mut target_name = Name::qualified(vec!["self", "id", "number"]);
    target_name.set_scope(2); // second struct body scope
    let actual = *test_disambiguate!(syntax_trees, module_paths, target_name)
        .unwrap()
        .into_instance_access()
        .unwrap()
        .last()
        .unwrap()
        .as_field()
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
