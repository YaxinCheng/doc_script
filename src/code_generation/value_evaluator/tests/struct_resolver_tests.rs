use super::super::expression_evaluator::ExpressionEvaluator;
use crate::ast::{abstract_tree, StructDeclaration};
use crate::code_generation::value_evaluator::struct_evaluator::{Struct, StructEvaluator};
use crate::code_generation::value_evaluator::value::Value;
use crate::env::Environment;
use crate::parser::parse;
use crate::tokenizer::tokenize;
use std::borrow::Cow;
use std::collections::HashMap;

#[test]
fn test_resolve_empty_struct() {
    let struct_declaration = get_struct(
        r#"
        struct Empty
        "#,
    )
    .expect("StructDeclaration not found");
    let structure = generate_struct(&struct_declaration);
    assert!(structure.default_fields.is_empty());
    assert!(structure.attributes.is_empty());
}

#[test]
fn test_resolve_struct_with_fields_no_default() {
    let struct_declaration = get_struct(
        r#"
        struct Empty(field: String)
        "#,
    )
    .expect("StructDeclaration not found");
    let structure = generate_struct(&struct_declaration);
    assert!(structure.default_fields.is_empty());
    assert!(structure.attributes.is_empty());
}

#[test]
fn test_resolve_struct_with_default_fields() {
    let struct_declaration = get_struct(
        r#"
        struct Empty(field: String = "string")
        "#,
    )
    .expect("StructDeclaration not found");
    let structure = generate_struct(&struct_declaration);
    let expected = [("field", Value::String(Cow::Borrowed("string")))]
        .into_iter()
        .collect::<HashMap<_, _>>();
    assert_eq!(structure.default_fields, expected);
    assert!(structure.attributes.is_empty());
}

#[test]
fn test_resolve_struct_with_default_fields_and_attributes() {
    let struct_declaration = get_struct(
        r#"
        struct Empty(field: String = "string") {
            const attr = 3
        }
        "#,
    )
    .expect("StructDeclaration not found");
    let structure = generate_struct(&struct_declaration);
    let expected_fields = [("field", Value::String(Cow::Borrowed("string")))]
        .into_iter()
        .collect::<HashMap<_, _>>();
    assert_eq!(structure.default_fields, expected_fields);
    let expected_attribute_keys = vec!["attr"];
    assert_eq!(
        structure.attributes.keys().copied().collect::<Vec<_>>(),
        expected_attribute_keys
    );
}

fn get_struct(program: &str) -> Option<StructDeclaration> {
    let mut syntax_tree = abstract_tree(parse(tokenize(program)));
    syntax_tree
        .compilation_unit
        .declarations
        .remove(0)
        .into_struct()
        .ok()
}

fn generate_struct<'ast, 'a>(struct_declaration: &'ast StructDeclaration<'a>) -> Struct<'ast, 'a> {
    let environment = Environment::default();
    let mut expression_resolver = ExpressionEvaluator::with_environment(&environment);
    let struct_resolver = StructEvaluator(&mut expression_resolver);
    struct_resolver.evaluate(struct_declaration)
}
