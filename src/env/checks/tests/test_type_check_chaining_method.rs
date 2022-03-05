use super::try_block;
use crate::ast::abstract_tree;
use crate::env::checks::type_checking::types::Types;
use crate::env::checks::type_checking::TypeChecker;
use crate::env::Environment;
use crate::parser::parse;
use crate::tests::FormulaSuppress;
use crate::tokenizer::tokenize;

#[test]
fn test_chaining_method_invoke_normal() {
    test_chaining_method_invoke(
        r#"
        struct People(name: String, age: Int = 0) 
        
        const people = People("test").age(25)
        "#,
    )
}

#[test]
#[should_panic]
fn test_chaining_method_field_not_exist() {
    test_chaining_method_invoke(
        r#"
        struct People(name: String, age: Int = 0) 
        
        const people = People("test").birth_day(25)
        "#,
    )
}

#[test]
fn test_chaining_method_missing_parameter_with_default_value() {
    test_chaining_method_invoke(
        r#"
        struct People(name: String, registered: Bool = false) 
        
        const people = People("test").registered()
        "#,
    )
}

#[test]
#[should_panic]
fn test_chaining_method_missing_parameter_without_default_value() {
    test_chaining_method_invoke(
        r#"
        struct People(name: String, age: Int = 0) 
        
        const people = People("test").name()
        "#,
    )
}

#[test]
fn test_chaining_method_override_constructor() {
    test_chaining_method_invoke(
        r#"
        struct People(name: String, age: Int = 0) 
        
        const people = People("test").name("not test")
        "#,
    )
}

#[test]
fn test_chaining_method_from_constant() {
    test_chaining_method_invoke(
        r#"
        struct People(name: String, age: Int = 0) 
        
        const people = People("test");
        // needs bracket, because otherwise it will be recognized as type
        const debug_people = people.name("debug")
        "#,
    )
}

fn test_chaining_method_invoke(program: &str) {
    let formula = FormulaSuppress::all();
    formula.suppress();

    use crate::ast::{Expression, StructDeclaration};
    let mut syntax_trees = vec![abstract_tree(parse(tokenize(program)))];
    let module_paths = vec![vec![]];
    let env = Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .build();

    let target_expression = try_block!(
        &Expression,
        syntax_trees
            .first()?
            .compilation_unit
            .declarations
            .last()?
            .as_constant()
            .map(|constant| &constant.value)
    );
    let actual = TypeChecker::with_environment(&env).test_resolve_expression(target_expression);
    let expected_type = Types::Struct(try_block!(
        &StructDeclaration,
        syntax_trees
            .first()?
            .compilation_unit
            .declarations
            .first()?
            .as_struct()
    ));
    assert_eq!(actual, expected_type)
}
