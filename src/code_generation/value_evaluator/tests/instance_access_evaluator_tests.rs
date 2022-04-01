use super::super::expression_evaluator::ExpressionEvaluator;
use super::super::instance_access_evaluator::InstanceAccessEvaluator;
use super::get_constant;
use crate::ast::abstract_tree;
use crate::code_generation::value_evaluator::value::Value;
use crate::env::Environment;
use crate::formula_suppress::FormulaSuppress;
use crate::parser::parse;
use crate::tokenizer::tokenize;

#[test]
fn test_access_field_externally() {
    test_evaluated_value(
        r#"
    struct Test(field: Int = 42)
    const a = Test()
    "#,
        &["field"],
        Value::Int(42),
    )
}

#[test]
fn test_access_field_chain_externally() {
    test_evaluated_value(
        r#"
    struct Test(field: Int = 42)
    struct Eval(test: Test = Test(3))
    const a = Eval()
    "#,
        &["test", "field"],
        Value::Int(3),
    )
}

#[test]
fn test_access_attr() {
    test_evaluated_value(
        r#"
        struct Test(field: Int = 42) {
            const attr = 3 
        }
        const a = Test()
        "#,
        &["attr"],
        Value::Int(3),
    )
}

#[test]
fn test_access_attr_refers_to_field() {
    test_evaluated_value(
        r#"
    struct Test(field: Int) {
        const attr = self.field
    }
    const a = Test(42)
    "#,
        &["attr"],
        Value::Int(42),
    )
}

fn test_evaluated_value(program: &str, accesses: &[&str], expected: Value) {
    let checkers = FormulaSuppress::all();
    checkers.suppress();

    let mut syntax_trees = [abstract_tree(parse(tokenize(program)))];
    let env = Environment::builder()
        .add_modules(&[vec![]])
        .generate_scopes(&mut syntax_trees)
        .resolve_names(&syntax_trees)
        .build();
    let mut expr_evaluator = ExpressionEvaluator::with_environment(&env);
    let instance_access_evaluator = InstanceAccessEvaluator::new(&mut expr_evaluator, None);
    let receiver = get_constant(&syntax_trees[0]).expect("Constant not found");
    let actual = instance_access_evaluator.evaluate(receiver, accesses);
    assert_eq!(actual, expected)
}
