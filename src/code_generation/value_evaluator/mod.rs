use crate::ast::ConstantDeclaration;
use crate::env::Environment;

mod expression_evaluator;
mod instance_access_evaluator;
mod instance_evaluator;
mod literal_evaluator;
mod string_evaluator;
mod struct_evaluator;
#[cfg(test)]
mod tests;
mod value;

pub fn evaluate<'ast, 'a>(
    environment: &Environment<'ast, 'a>,
    entry: &'ast ConstantDeclaration<'a>,
) -> value::Value<'ast, 'a> {
    expression_evaluator::ExpressionEvaluator::with_environment(environment)
        .evaluate(&entry.value, None)
}
