use crate::ast::{abstract_tree, Name};
use crate::env::scope::{Scoped, GLOBAL_SCOPE};
use crate::env::Environment;
use crate::parser::parse;
use crate::tokenizer::tokenize;

macro_rules! test_disambiguate {
    ($syntax_trees: expr, $module_paths: expr, $name: expr) => {{
        let formula = crate::tests::FormulaSuppress::all();
        formula.suppress();

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
        .1
        .last()
        .unwrap();
    let expected = "digit";
    assert_eq!(actual, expected)
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
        .1
        .last()
        .unwrap();
    let expected = "test";
    assert_eq!(actual, expected)
}
