use super::{construct_env, try_block};
use crate::ast::{abstract_tree, Expression, StructDeclaration};
use crate::env::name_resolution::types::Types;
use crate::env::name_resolution::{NameResolver, TypeChecker, TypeLinker};
use crate::env::{declaration_resolution, Environment};
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
    let mut env = construct_env();
    let actual = TypeChecker::default().test_resolve_expression(&mut env, &expression);
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
    let mut env = Environment::construct(&mut syntax_trees, &module_paths);
    let names = declaration_resolution::resolve(&mut env, &syntax_trees, &module_paths);
    TypeLinker(&mut env).link_types(names.type_names);
    let instance_fields = NameResolver(&mut env).resolve_names(names.expression_names);

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
    let actual =
        TypeChecker::new(instance_fields).test_resolve_expression(&mut env, target_expression);
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
    let mut env = Environment::construct(&mut syntax_trees, &module_paths);
    let names = declaration_resolution::resolve(&mut env, &syntax_trees, &module_paths);
    TypeLinker(&mut env).link_types(names.type_names);
    let instance_fields = NameResolver(&mut env).resolve_names(names.expression_names);

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
    let actual = TypeChecker::new(instance_fields).test_resolve_expression(&mut env, target_block);
    let expected = Types::Int;
    assert_eq!(actual, expected)
}
