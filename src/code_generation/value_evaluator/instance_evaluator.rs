use super::expression_evaluator::ExpressionEvaluator;
use super::value::Value;
use super::value::{Instance, Struct};
use crate::ast::{Field, Parameter, StructInitContent};
use std::collections::HashMap;
use std::rc::Rc;

pub struct InstanceEvaluator<'ast, 'a, 'env, 'res> {
    expr_resolver: &'res mut ExpressionEvaluator<'ast, 'a, 'env>,
    self_ref: Option<Value<'ast, 'a>>,
}

const INIT_CONTENT: &str = "children";

impl<'ast, 'a, 'env, 'res> InstanceEvaluator<'ast, 'a, 'env, 'res> {
    pub fn new(
        expr_resolver: &'res mut ExpressionEvaluator<'ast, 'a, 'env>,
        self_ref: Option<Value<'ast, 'a>>,
    ) -> Self {
        Self {
            expr_resolver,
            self_ref,
        }
    }

    pub fn evaluate(
        mut self,
        structure: Rc<Struct<'ast, 'a>>,
        fields: &'ast [Field<'a>],
        parameters: &'ast [Parameter<'a>],
        init_content: &'ast Option<StructInitContent<'a>>,
    ) -> Instance<'ast, 'a> {
        let is_labelled_parameter = parameters
            .first()
            .map(Parameter::is_labelled)
            .unwrap_or(false);
        let mut instance_fields = match is_labelled_parameter {
            true => self.add_labelled_parameters(parameters),
            false => self.add_positional_parameters(parameters, fields),
        };
        if let Some(init_content) = init_content {
            instance_fields.insert(INIT_CONTENT, self.resolve_init_content(init_content));
        }
        Instance {
            structure,
            fields: instance_fields,
        }
    }

    fn add_labelled_parameters(
        &mut self,
        parameter: &'ast [Parameter<'a>],
    ) -> HashMap<&'a str, Value<'ast, 'a>> {
        parameter
            .iter()
            .map(|parameter| match parameter {
                Parameter::Labelled { label, content } => (
                    *label,
                    self.expr_resolver.evaluate(content, self.self_ref.clone()),
                ),
                _ => unreachable!("Checked before"),
            })
            .collect()
    }

    fn add_positional_parameters(
        &mut self,
        parameter: &'ast [Parameter<'a>],
        fields: &'ast [Field<'a>],
    ) -> HashMap<&'a str, Value<'ast, 'a>> {
        fields
            .iter()
            .map(|field| field.name)
            .zip(
                parameter
                    .iter()
                    .map(Parameter::expression)
                    .map(|expr| self.expr_resolver.evaluate(expr, self.self_ref.clone())),
            )
            .collect()
    }

    fn resolve_init_content(
        &mut self,
        init_content: &'ast StructInitContent<'a>,
    ) -> Value<'ast, 'a> {
        Value::Array(
            init_content
                .0
                .iter()
                .map(|expr| self.expr_resolver.evaluate(expr, self.self_ref.clone()))
                .collect(),
        )
    }
}
