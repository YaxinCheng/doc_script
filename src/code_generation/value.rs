use super::value_evaluator::ExpressionEvaluator;
use crate::ast::Expression;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

type Str<'a> = Cow<'a, str>;

#[cfg_attr(test, derive(Debug, PartialEq))]
#[derive(Clone)]
pub enum Value<'ast, 'a> {
    Int(isize),
    Float(f32),
    Bool(bool),
    String(Str<'a>),
    Instance(Rc<Instance<'ast, 'a>>),
    Array(Vec<Value<'ast, 'a>>),
    Void,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
#[derive(Clone)]
pub struct Instance<'ast, 'a> {
    pub structure: Rc<Struct<'ast, 'a>>,
    pub fields: HashMap<&'a str, Value<'ast, 'a>>,
}

impl<'ast, 'a> Instance<'ast, 'a> {
    pub fn field(&self, name: &str) -> Option<Value<'ast, 'a>> {
        self.fields
            .get(name)
            .or_else(|| self.structure.default_fields.get(name))
            .cloned()
    }

    pub fn set_field(&mut self, name: &'a str, value: Value<'ast, 'a>) {
        self.fields.insert(name, value);
    }

    pub fn reset_field(&mut self, name: &str) {
        self.fields.remove(name);
    }

    pub fn attribute<'env>(
        self: &Rc<Self>,
        expression_resolver: &mut ExpressionEvaluator<'ast, 'a, 'env>,
        name: &str,
    ) -> Option<Value<'ast, 'a>> {
        self.structure
            .attributes
            .get(name)
            .copied()
            .map(|attr| expression_resolver.evaluate(attr, Some(Value::Instance(Rc::clone(self)))))
    }

    pub fn attributes<'env>(
        self: &Rc<Self>,
        expression_resolver: &mut ExpressionEvaluator<'ast, 'a, 'env>,
    ) -> Vec<(&'a str, Value<'ast, 'a>)> {
        let attributes = self
            .structure
            .attributes
            .iter()
            .map(|(name, attr)| {
                (
                    *name,
                    expression_resolver.evaluate(attr, Some(Value::Instance(Rc::clone(self)))),
                )
            })
            .collect::<Vec<_>>();
        #[cfg(test)]
        let attributes = {
            let mut attributes = attributes;
            attributes.sort_by_key(|(name, _)| *name);
            attributes
        };
        attributes
    }

    pub fn fields(self: &Rc<Self>) -> Vec<(&'a str, Value<'ast, 'a>)> {
        let mut appeared_field = HashSet::new();
        let mut output =
            Vec::with_capacity(self.fields.len() + self.structure.default_fields.len());
        for (field_name, value) in self
            .fields
            .iter()
            .chain(self.structure.default_fields.iter())
        {
            if !appeared_field.insert(*field_name) {
                continue;
            }
            output.push((*field_name, value.clone()))
        }
        #[cfg(test)]
        output.sort_by_key(|(name, _)| *name);
        output
    }
}

#[cfg_attr(test, derive(Debug, PartialEq, Default))]
pub struct Struct<'ast, 'a> {
    pub name: &'a str,
    pub default_fields: HashMap<&'a str, Value<'ast, 'a>>,
    pub attributes: HashMap<&'a str, &'ast Expression<'a>>,
    pub package_state: PackageState,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
#[derive(Copy, Clone)]
pub enum PackageState {
    Render,
    Normal,
}

impl Default for PackageState {
    fn default() -> Self {
        Self::Normal
    }
}
