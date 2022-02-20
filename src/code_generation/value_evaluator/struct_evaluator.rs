use super::expression_evaluator::ExpressionEvaluator;
use super::value::{Struct, Value};
use crate::ast::{Expression, Field, StructBody, StructDeclaration};
use crate::code_generation::value::PackageState;
use crate::env::ModuleVerifier;
use std::collections::HashMap;
use std::rc::Rc;

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
        let module_verifier = ModuleVerifier::with_environment(self.0.env);
        let package_state = module_verifier
            .in_render(struct_definition)
            .then(|| PackageState::Render)
            .unwrap_or(PackageState::Normal);
        Rc::new(Struct {
            name: struct_definition.name,
            default_fields,
            attributes,
            package_state,
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
