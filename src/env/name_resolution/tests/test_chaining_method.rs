use crate::ast::abstract_tree;
use crate::env::name_resolution::resolution::NameResolver;
use crate::env::name_resolution::tests::try_block;
use crate::env::name_resolution::type_checking::TypeChecker;
use crate::env::name_resolution::type_linker::TypeLinker;
use crate::env::name_resolution::types::Types;
use crate::env::{declaration_resolution, Environment};
use crate::parser::parse;
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
        const debug_people = (people).name("debug")
        "#,
    )
}

fn test_chaining_method_invoke(program: &str) {
    use crate::ast::{Expression, StructDeclaration};
    let mut syntax_trees = vec![abstract_tree(parse(tokenize(program)))];
    let module_paths = vec![vec![]];
    let mut env = Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .build();
    let names = declaration_resolution::resolve(&mut env, &syntax_trees, &module_paths);
    TypeLinker(&mut env).link_types(names.type_names);
    let instance_fields = NameResolver(&mut env).resolve_names(names.expression_names);
    let target_expression = try_block!(
        &Expression,
        syntax_trees
            .first()?
            .compilation_unit
            .declarations
            .last()?
            .as_constant()
            .map(|constant| &constant.value)
    )
    .unwrap();
    let actual =
        TypeChecker::new(instance_fields).test_resolve_expression(&mut env, target_expression);
    let expected_type = try_block!(
        &StructDeclaration,
        syntax_trees
            .first()?
            .compilation_unit
            .declarations
            .first()?
            .as_struct()
    )
    .map(Types::Struct)
    .unwrap();
    assert_eq!(actual, expected_type)
}
