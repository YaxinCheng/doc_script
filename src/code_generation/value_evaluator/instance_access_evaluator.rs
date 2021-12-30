use super::expression_evaluator::ExpressionEvaluator;
use super::value::Value;
use crate::ast::ConstantDeclaration;

pub struct InstanceAccessEvaluator<'ast, 'a, 'env, 'res> {
    expr_evaluator: &'res mut ExpressionEvaluator<'ast, 'a, 'env>,
    self_ref: Option<Value<'ast, 'a>>,
}

impl<'ast, 'a, 'env, 'res> InstanceAccessEvaluator<'ast, 'a, 'env, 'res> {
    pub fn new(
        expr_resolver: &'res mut ExpressionEvaluator<'ast, 'a, 'env>,
        self_ref: Option<Value<'ast, 'a>>,
    ) -> Self {
        Self {
            expr_evaluator: expr_resolver,
            self_ref,
        }
    }

    pub fn evaluate(
        self,
        receiver: &'ast ConstantDeclaration<'a>,
        instance_access: &[&'a str],
    ) -> Value<'ast, 'a> {
        let mut value = self
            .expr_evaluator
            .evaluate(&receiver.value, self.self_ref);
        for name in instance_access {
            value = match value {
                Value::Instance(instance) => instance
                    .field(name)
                    .or_else(|| instance.attribute(self.expr_evaluator, name))
                    .unwrap_or_else(|| panic!("field `{}` does not exist", name)),
                _ => unreachable!("Field access can only happen on struct instance"),
            }
        }
        value
    }
}
