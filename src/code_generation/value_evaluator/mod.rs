use super::value;
use crate::ast::ConstantDeclaration;

mod expression_evaluator;
mod instance_access_evaluator;
mod instance_evaluator;
mod literal_evaluator;
mod string_evaluator;
mod struct_evaluator;
#[cfg(test)]
mod tests;

pub use expression_evaluator::ExpressionEvaluator;

pub fn evaluate<'ast, 'a>(
    evaluator: &mut ExpressionEvaluator<'ast, 'a, '_>,
    entry: &'ast ConstantDeclaration<'a>,
) -> value::Value<'ast, 'a> {
    evaluator.evaluate(&entry.value, None)
}
