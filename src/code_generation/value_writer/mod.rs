#[cfg(test)]
mod tests;

use super::value::{Instance, Value};
use super::value_evaluator::ExpressionEvaluator;
use crate::code_generation::value::PackageState;
use core::fmt::Result;
use std::fmt::Write;
use std::rc::Rc;

const RENDERED: &str = "rendered";
const RENDER_TAG: &str = "__RENDER_TAG";

pub fn write_to_string<'ast, 'a>(
    evaluator: ExpressionEvaluator<'ast, 'a, '_>,
    value: Value<'ast, 'a>,
) -> String {
    let mut formatter = ValueFormatter::new(evaluator);
    formatter.format(&value).expect("Failed to format");
    formatter.export()
}

struct ValueFormatter<'ast, 'a, 'env> {
    pub evaluator: ExpressionEvaluator<'ast, 'a, 'env>,
    output: String,
}

impl<'ast, 'a, 'env> ValueFormatter<'ast, 'a, 'env> {
    pub fn new(evaluator: ExpressionEvaluator<'ast, 'a, 'env>) -> Self {
        ValueFormatter {
            evaluator,
            output: String::new(),
        }
    }

    pub fn format(&mut self, value: &Value<'ast, 'a>) -> Result {
        match value {
            Value::Int(int) => write!(self.output, "{int}"),
            Value::Float(float) => write!(self.output, "{float}"),
            Value::Bool(bool) => write!(self.output, "{bool}"),
            Value::String(string) => write!(self.output, "{string:?}"),
            Value::Void => Result::Ok(()),
            Value::Array(values) => self.format_array(values),
            Value::Instance(instance) => self.format_instance(instance),
        }
    }

    pub fn export(self) -> String {
        self.output
    }

    fn format_array(&mut self, array: &[Value<'ast, 'a>]) -> Result {
        write!(self.output, "[")?;
        for value in array {
            if matches!(value, Value::Void) {
                continue;
            }
            self.format(value)?;
            self.output.write_char(',')?;
        }
        write!(self.output, "]")
    }

    fn format_instance(&mut self, instance: &Rc<Instance<'ast, 'a>>) -> Result {
        match instance.structure.package_state {
            PackageState::Render => self.format_render_instance(instance),
            PackageState::Normal => self.format_normal_instance(instance),
        }
    }

    fn format_render_instance(&mut self, instance: &Rc<Instance<'ast, 'a>>) -> Result {
        let tag = match instance.field(RENDER_TAG) {
            Some(Value::String(tag)) => tag,
            None => instance.structure.name.into(),
            _ => unreachable!("{RENDER_TAG} is not defined as string"),
        };
        write!(self.output, "{tag}: {{")?;
        instance
            .fields()
            .into_iter()
            .chain(instance.attributes(&mut self.evaluator).into_iter())
            .filter(|(name, _)| *name != RENDER_TAG)
            .filter(|(_, value)| !matches!(value, Value::Void))
            .try_for_each(|(name, value)| self.format_key_value(name, &value))?;
        write!(self.output, "}}")
    }

    fn format_key_value(&mut self, key: &str, value: &Value<'ast, 'a>) -> Result {
        write!(self.output, "{key}: ")?;
        self.format(value)?;
        write!(self.output, ",")
    }

    fn format_normal_instance(&mut self, instance: &Rc<Instance<'ast, 'a>>) -> Result {
        instance
            .field(RENDERED)
            .or_else(|| instance.attribute(&mut self.evaluator, RENDERED))
            .map(|rendered| self.format(&rendered))
            .expect("rendered is not defined")
    }
}
