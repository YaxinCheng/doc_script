use super::expression_evaluator::ExpressionEvaluator;
use super::value::Value;
use crate::ast::{Expression, Field, StructBody, StructDeclaration};
use std::collections::HashMap;
use std::rc::Rc;

#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Struct<'ast, 'a> {
    pub default_fields: HashMap<&'a str, Value<'ast, 'a>>,
    pub attributes: HashMap<&'a str, &'ast Expression<'a>>,
}

pub struct StructEvaluator<'ast, 'a, 'env, 'res>(pub &'res mut ExpressionEvaluator<'ast, 'a, 'env>);

impl<'ast, 'a, 'env, 'res> StructEvaluator<'ast, 'a, 'env, 'res> {
    pub fn evaluate(
        mut self,
        struct_definition: &'ast StructDeclaration<'a>,
    ) -> Rc<Struct<'ast, 'a>> {
        let default_fields = self.resolve_default_fields(&struct_definition.fields);
        let attributes = struct_definition
            .body
            .as_ref()
            .map(Self::resolve_attributes)
            .unwrap_or_default();
        Rc::new(Struct {
            default_fields,
            attributes,
        })
    }

    fn resolve_default_fields(
        &mut self,
        fields: &'ast [Field<'a>],
    ) -> HashMap<&'a str, Value<'ast, 'a>> {
        fields
            .iter()
            .filter_map(|field| {
                Some((
                    field.name,
                    field
                        .default_value
                        .as_ref()
                        .map(|expr| self.0.evaluate(expr, None))?,
                ))
            })
            .collect()
    }

    fn resolve_attributes(
        struct_body: &'ast StructBody<'a>,
    ) -> HashMap<&'a str, &'ast Expression<'a>> {
        struct_body
            .attributes
            .iter()
            .map(|attribute| (attribute.name, &attribute.value))
            .collect()
    }
}
