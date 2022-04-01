use super::super::expression_evaluator::ExpressionEvaluator;
use super::super::value::Value;
use super::get_constant;
use crate::ast::abstract_tree;
use crate::env::Environment;
use crate::formula_suppress::FormulaSuppress;
use crate::parser::parse;
use crate::tokenizer::tokenize;

#[test]
fn test_evaluate_to_instance() {
    let program = r#"
    struct Test(a: Int)
    const a = Test(42)
    "#;
    test_instance(program, "a", Value::Int(42))
}

#[test]
fn test_chaining_method() {
    let program = r#"
    struct Test(a: Int)
    const first = Test(42)
    const second = (first).a(84)
    "#;
    test_instance(program, "a", Value::Int(84))
}

#[test]
fn test_chaining_method_with_default() {
    let program = r#"
    struct Test(a: Int = 42)
    const first = Test(84)
    const second = (first).a()
    "#;
    test_instance(program, "a", Value::Int(42))
}

fn test_instance(program: &str, field: &str, expected: Value) {
    let checkers = FormulaSuppress::all();
    checkers.suppress();

    let mut syntax_trees = [abstract_tree(parse(tokenize(program)))];
    let env = Environment::builder()
        .add_modules(&[vec![]])
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .build();
    let constant = get_constant(&syntax_trees[0]).expect("Constant not found");
    let actual = ExpressionEvaluator::with_environment(&env).evaluate(&constant.value, None);
    let instance = match actual {
        Value::Instance(instance) => instance,
        _ => panic!("Not instance"),
    };
    let field = instance.field(field);
    assert_eq!(field, Some(expected))
}

#[test]
fn test_constant_evaluate() {
    test_expression(
        r#"
    const a = 42
    const b = a
    "#,
        Value::Int(42),
    )
}

#[test]
fn test_field_access() {
    test_expression(
        r#"
    struct A(val: Int)
    struct B(a: A)
    struct C(b: B)
    const value = C(B(A(42)))
    const target = value.b.a.val
    "#,
        Value::Int(42),
    )
}

#[test]
fn test_block() {
    let program = r#"
    const a = {
        const b = 42
        b
    }
    "#;
    test_expression(program, Value::Int(42))
}

#[test]
fn test_empty_block() {
    let program = r#"
    const a = {}
    "#;
    test_expression(program, Value::Void)
}

#[test]
fn test_block_returns_void() {
    let program = r#"
    const a = {
        const b = 4
    }
    "#;
    test_expression(program, Value::Void)
}

fn test_expression(program: &str, expected: Value) {
    let checkers = FormulaSuppress::all();
    checkers.suppress();

    let mut syntax_trees = [abstract_tree(parse(tokenize(program)))];
    let env = Environment::builder()
        .add_modules(&[vec![]])
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .build();
    let constant = get_constant(&syntax_trees[0]).expect("Constant not found");
    let actual = ExpressionEvaluator::with_environment(&env).evaluate(&constant.value, None);
    assert_eq!(actual, expected)
}
