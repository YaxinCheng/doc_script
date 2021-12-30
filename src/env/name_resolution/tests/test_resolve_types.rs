use super::{construct_env, try_block};
use crate::ast::{abstract_tree, ConstantDeclaration, Expression, StructDeclaration};
use crate::env::name_resolution::types::Types;
use crate::env::name_resolution::TypeChecker;
use crate::env::Environment;
use crate::parser::parse;
use crate::tokenizer::{tokenize, LiteralKind};

#[test]
fn test_int() {
    test_literals(LiteralKind::Integer, Types::Int)
}

#[test]
fn test_binary() {
    test_literals(LiteralKind::Binary, Types::Int)
}

#[test]
fn test_float() {
    test_literals(LiteralKind::Floating, Types::Float)
}

#[test]
fn test_string() {
    test_literals(LiteralKind::String, Types::String)
}

#[test]
fn test_bool() {
    test_literals(LiteralKind::Boolean, Types::Bool)
}

fn test_literals(kind: LiteralKind, expected: Types) {
    let expression = Expression::Literal { kind, lexeme: "" };
    let env = construct_env();
    let actual = TypeChecker::with_environment(&env).test_resolve_expression(&expression);
    assert_eq!(actual, expected)
}

#[test]
fn test_resolve_struct() {
    let mut syntax_trees = vec![abstract_tree(parse(tokenize(
        r#"
        struct Empty
        const a = Empty()
        "#,
    )))];
    let module_paths = vec![vec![]];
    let env = Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .build();

    let target_expression = try_block!(
        &Expression,
        Some(
            &syntax_trees
                .first()?
                .compilation_unit
                .declarations
                .last()?
                .as_constant()?
                .value
        )
    )
    .unwrap();
    let target_struct = try_block!(
        &StructDeclaration,
        syntax_trees
            .first()?
            .compilation_unit
            .declarations
            .first()?
            .as_struct()
    )
    .unwrap();
    let actual = TypeChecker::with_environment(&env).test_resolve_expression(target_expression);
    let expected = Types::Struct(target_struct);
    assert_eq!(actual, expected)
}

#[test]
fn test_resolve_from_block() {
    let mut syntax_trees = vec![abstract_tree(parse(tokenize(
        r#"
        const a = {
            const b = 3
            b
        }
        "#,
    )))];
    let module_paths = vec![vec![]];
    let env = Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .build();

    let target_block = try_block!(
        &Expression,
        Some(
            &syntax_trees
                .first()?
                .compilation_unit
                .declarations
                .first()?
                .as_constant()?
                .value
        )
    )
    .unwrap();
    let actual = TypeChecker::with_environment(&env).test_resolve_expression(target_block);
    let expected = Types::Int;
    assert_eq!(actual, expected)
}

#[test]
fn test_resolve_field_access_from_block() {
    let mut syntax_trees = vec![abstract_tree(parse(tokenize(
        r#"
        struct A(field: String)
        const a = {
            A("test")
        }.field
        "#,
    )))];
    let module_paths = vec![vec![]];
    let env = Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .build();

    let target_block = try_block!(
        &Expression,
        Some(
            &syntax_trees
                .first()?
                .compilation_unit
                .declarations
                .last()?
                .as_constant()?
                .value
        )
    )
    .unwrap();
    let actual = TypeChecker::with_environment(&env).test_resolve_expression(target_block);
    let expected = Types::String;
    assert_eq!(actual, expected)
}

#[test]
fn test_type_field_access_internal() {
    test_type_access_internal(
        r#"
        struct A(field: String) {
            const a = self.field
        }
        "#,
    );
}

#[test]
fn test_type_attribute_access_internal() {
    test_type_access_internal(
        r#"
        struct A(field: String) {
            const b = self.field
            const a = b
        }
        "#,
    );
}

#[test]
#[should_panic]
fn test_type_field_access_internal_without_self() {
    test_type_access_internal(
        r#"
        struct A(field: String) {
            const a = field
        }
        "#,
    );
}

#[test]
fn test_type_attribute_access_internal_with_self() {
    test_type_access_internal(
        r#"
        struct A(field: String) {
            const b = self.field
            const a = self.b
        }
        "#,
    );
}

#[test]
#[should_panic]
fn test_type_attribute_access_internal_directly_from_type() {
    test_type_access_internal(
        r#"
        struct A(field: String) {
            const b = self.field
            const a = A.b
        }
        "#,
    );
}

#[test]
#[should_panic]
fn test_type_field_access_internal_directly_from_type() {
    test_type_access_internal(
        r#"
        struct A(field: String) {
            const a = A.field
        }
        "#,
    );
}

#[test]
#[should_panic]
fn test_type_constant_points_to_struct() {
    test_type_access_internal(
        r#"
        struct A(field: String) 
        const a = A
        "#,
    );
}

fn test_type_access_internal(program: &str) {
    let mut syntax_trees = vec![abstract_tree(parse(tokenize(program)))];
    let module_paths = vec![vec![]];
    let env = Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .build();

    let target_constant = try_block!(
        &ConstantDeclaration,
        syntax_trees
            .first()?
            .compilation_unit
            .declarations
            .first()?
            .as_struct()?
            .body
            .as_ref()?
            .attributes
            .first()
    )
    .unwrap();

    let actual =
        TypeChecker::with_environment(&env).test_resolve_expression(&target_constant.value);
    let expected = Types::String;
    assert_eq!(actual, expected)
}

#[test]
fn test_self_type() {
    let program = r#"
    struct TestType {
        const a = self
    }
    "#;
    let module_paths = vec![vec![]];
    let mut syntax_trees = [abstract_tree(parse(tokenize(program)))];
    let env = Environment::builder()
        .add_modules(&module_paths)
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .build();

    let target_struct = try_block!(
        &StructDeclaration,
        syntax_trees
            .first()?
            .compilation_unit
            .declarations
            .first()?
            .as_struct()
    )
    .unwrap();
    let target_constant = try_block!(
        &ConstantDeclaration,
        target_struct.body.as_ref()?.attributes.first()
    )
    .unwrap();
    let actual =
        TypeChecker::with_environment(&env).test_resolve_expression(&target_constant.value);
    let expected = Types::Struct(target_struct);
    assert_eq!(actual, expected)
}
