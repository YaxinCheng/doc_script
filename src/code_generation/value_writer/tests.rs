use super::super::value::{Instance, PackageState, Struct, Value};
use super::super::value_evaluator::ExpressionEvaluator;
use super::write_to_string;
use super::RENDER_TAG;
use crate::env::Environment;
use std::collections::HashMap;
use std::rc::Rc;

fn evaluator<'ast, 'a, 'env>(
    env: &'env Environment<'ast, 'a>,
) -> ExpressionEvaluator<'ast, 'a, 'env> {
    ExpressionEvaluator::with_environment(env)
}

#[test]
fn test_primitive_values() {
    let primitive_values = [
        Value::Int(42),
        Value::Float(10.48),
        Value::Bool(true),
        Value::String("Hello World".into()),
        Value::Void,
    ];
    let expected = ["42", "10.48", "true", r#""Hello World""#, ""];
    let env = Environment::default();
    for (primitive_value, expected_output) in primitive_values.into_iter().zip(expected) {
        assert_eq!(
            write_to_string(evaluator(&env), primitive_value),
            expected_output
        )
    }
}

#[test]
fn test_primitive_arrays() {
    let primitive_array = Value::Array(vec![
        Value::Int(42),
        Value::Float(10.48),
        Value::Void,
        Value::Bool(true),
        Value::String("Hello World".into()),
    ]);
    let expected = r#"[42,10.48,true,"Hello World",]"#;
    let env = Environment::default();
    assert_eq!(write_to_string(evaluator(&env), primitive_array), expected)
}

#[test]
fn test_instance_simple_render_element() {
    let structure = Struct {
        package_state: PackageState::Render,
        ..Default::default()
    };
    let fields = [
        ("size", Value::Int(42)),
        (RENDER_TAG, Value::String("Tag".into())),
    ]
    .into_iter()
    .collect();
    let expected = "Tag: {size: 42,}";
    test_instance_to_value(structure, fields, expected)
}

#[test]
fn test_instance_render_element_with_default_fields() {
    let default_fields = [("font", Value::String("Code".into()))]
        .into_iter()
        .collect();
    let structure = Struct {
        package_state: PackageState::Render,
        default_fields,
        ..Default::default()
    };
    let fields = [
        ("size", Value::Int(42)),
        (RENDER_TAG, Value::String("Tag".into())),
    ]
    .into_iter()
    .collect();
    let expected = r#"Tag: {size: 42,font: "Code",}"#;
    test_instance_to_value(structure, fields, expected)
}

#[test]
fn test_instance_render_element_with_void_field() {
    let structure = Struct {
        package_state: PackageState::Render,
        ..Default::default()
    };
    let fields = [
        ("size", Value::Int(42)),
        (RENDER_TAG, Value::String("Tag".into())),
        ("_is_view", Value::Void),
    ]
    .into_iter()
    .collect();
    let expected = r#"Tag: {size: 42,}"#;
    test_instance_to_value(structure, fields, expected)
}

#[test]
fn test_instance_render_name_as_tag() {
    let structure = Struct {
        package_state: PackageState::Render,
        name: "Tag",
        ..Default::default()
    };
    let fields = [("size", Value::Int(42))].into_iter().collect();
    let expected = r#"Tag: {size: 42,}"#;
    test_instance_to_value(structure, fields, expected)
}

#[test]
fn test_instance_normal() {
    let end_structure = Struct {
        package_state: PackageState::Render,
        name: "Tag",
        ..Default::default()
    };
    let end_point = Value::Instance(Rc::new(Instance {
        structure: Rc::new(end_structure),
        fields: [("size", Value::Int(42))].into_iter().collect(),
    }));

    let fields = [("rendered", end_point), ("irrelevant", Value::Float(3.0))]
        .into_iter()
        .collect();
    let expected = r#"Tag: {size: 42,}"#;
    test_instance_to_value(Struct::default(), fields, expected)
}

fn test_instance_to_value(structure: Struct, fields: HashMap<&str, Value>, expected: &str) {
    let env = Environment::default();
    let evaluator = evaluator(&env);
    let value = Value::Instance(Rc::new(Instance {
        structure: Rc::new(structure),
        fields,
    }));
    let actual = write_to_string(evaluator, value);
    assert_eq!(actual, expected)
}
