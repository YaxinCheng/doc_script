use crate::env::Environment;

mod value;
mod value_evaluator;
mod value_writer;

pub fn generate_code(environment: &Environment) -> Vec<u8> {
    let (evaluator, evaluated_value) = resolve_value(environment);
    write_value_to_buffer(evaluator, evaluated_value)
}

fn resolve_value<'ast, 'a, 'env>(
    environment: &'env Environment<'ast, 'a>,
) -> (
    value_evaluator::ExpressionEvaluator<'ast, 'a, 'env>,
    value::Value<'ast, 'a>,
) {
    let entry = environment.entry().expect("Entry cannot be found");
    let mut evaluator = value_evaluator::ExpressionEvaluator::with_environment(environment);
    let evaluated_value = value_evaluator::evaluate(&mut evaluator, entry);
    (evaluator, evaluated_value)
}

fn write_value_to_buffer<'ast, 'a>(
    evaluator: value_evaluator::ExpressionEvaluator<'ast, 'a, '_>,
    evaluated_value: value::Value<'ast, 'a>,
) -> Vec<u8> {
    let mut buffer = Vec::new();
    value_writer::write(evaluator, evaluated_value, &mut buffer);
    buffer
}
