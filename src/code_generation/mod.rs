use crate::env::Environment;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

mod value;
mod value_evaluator;
mod value_writer;

pub fn generate_code<P: AsRef<Path>>(environment: &Environment, output: P) {
    let (evaluator, evaluated_value) = resolve_value(environment);
    write_value_to_file(evaluator, evaluated_value, output)
}

#[cfg(test)]
pub fn generate_code_to_buffer(environment: &Environment) -> Vec<u8> {
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

fn write_value_to_file<'ast, 'a, P: AsRef<Path>>(
    evaluator: value_evaluator::ExpressionEvaluator<'ast, 'a, '_>,
    evaluated_value: value::Value<'ast, 'a>,
    output: P,
) {
    // TODO: use std::io::Error instead
    let buffer = write_value_to_buffer(evaluator, evaluated_value);
    let mut output = BufWriter::new(File::create(output).expect("Failed to create file"));
    output.write_all(&buffer).expect("Failed to write")
}

fn write_value_to_buffer<'ast, 'a>(
    evaluator: value_evaluator::ExpressionEvaluator<'ast, 'a, '_>,
    evaluated_value: value::Value<'ast, 'a>,
) -> Vec<u8> {
    let mut buffer = Vec::new();
    value_writer::write(evaluator, evaluated_value, &mut buffer);
    buffer
}
