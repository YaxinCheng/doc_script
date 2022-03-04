mod error;
mod output;
#[cfg(test)]
mod tests;

use super::value::{Instance, Value};
use super::value_evaluator::ExpressionEvaluator;
use crate::code_generation::value::PackageState;
use std::rc::Rc;

use error::Error;
use output::Output;

type Result<T, E = Error> = std::result::Result<T, E>;

const RENDERED: &str = "rendered";
const RENDER_TAG: &str = "__RENDER_TAG";

pub fn write<'ast, 'a, Out: Output>(
    evaluator: ExpressionEvaluator<'ast, 'a, '_>,
    value: Value<'ast, 'a>,
    output: &mut Out,
) {
    let mut formatter = ValueFormatter::new(evaluator, output);
    if let Err(error @ Error::IoError(_)) = formatter.format(&value) {
        panic!("{}", error)
    }
}

struct ValueFormatter<'ast, 'a, 'env, 'out, Out: Output> {
    evaluator: ExpressionEvaluator<'ast, 'a, 'env>,
    output: &'out mut Out,
}

impl<'ast, 'a, 'env, 'out, Out: Output> ValueFormatter<'ast, 'a, 'env, 'out, Out> {
    pub fn new(evaluator: ExpressionEvaluator<'ast, 'a, 'env>, output: &'out mut Out) -> Self {
        ValueFormatter { evaluator, output }
    }

    pub fn format(&mut self, value: &Value<'ast, 'a>) -> Result<()> {
        match value {
            Value::Int(int) => write!(self.output, "{int}").map_err(Error::from),
            Value::Float(float) => write!(self.output, "{float}").map_err(Error::from),
            Value::Bool(bool) => write!(self.output, "{bool}").map_err(Error::from),
            Value::String(string) => write!(self.output, "{string:?}").map_err(Error::from),
            Value::Void => Err(Error::EmptyContent),
            Value::Array(values) => self.format_array(values),
            Value::Instance(instance) => self.format_instance(instance),
        }
    }

    fn format_array(&mut self, array: &[Value<'ast, 'a>]) -> Result<()> {
        let init_len = self.output.position();
        write!(self.output, "[")?;
        let pre_format_len = self.output.position();
        for value in array {
            match self.format(value) {
                Err(Error::EmptyContent) => (),
                error @ Err(_) => error?,
                Ok(()) => self.output.write_all(b",")?,
            };
        }
        if self.output.position() == pre_format_len {
            self.output.truncate(init_len);
            Err(Error::EmptyContent)
        } else {
            Ok(write!(self.output, "]")?)
        }
    }

    fn format_instance(&mut self, instance: &Rc<Instance<'ast, 'a>>) -> Result<()> {
        match instance.structure.package_state {
            PackageState::Render => self.format_render_instance(instance),
            PackageState::Normal => self.format_normal_instance(instance),
        }
    }

    fn format_render_instance(&mut self, instance: &Rc<Instance<'ast, 'a>>) -> Result<()> {
        let tag = match instance.field(RENDER_TAG) {
            Some(Value::String(tag)) => tag,
            None => instance.structure.name.into(),
            _ => unreachable!("{RENDER_TAG} is not defined as string"),
        };
        let initial_len = self.output.position();
        write!(self.output, "{tag}: {{")?;
        let pre_format_len = self.output.position();
        let fields_attrs = instance
            .fields()
            .into_iter()
            .chain(instance.attributes(&mut self.evaluator).into_iter())
            .filter(|(name, _)| *name != RENDER_TAG);
        for (name, value) in fields_attrs {
            match self.format_key_value(name, &value) {
                Err(Error::EmptyContent) => (),
                err @ Err(_) => err?,
                Ok(()) => self.output.write_all(b",")?,
            }
        }
        if self.output.position() == pre_format_len {
            self.output.truncate(initial_len);
            Err(Error::EmptyContent)
        } else {
            Ok(write!(self.output, "}}")?)
        }
    }

    fn format_key_value(&mut self, key: &str, value: &Value<'ast, 'a>) -> Result<()> {
        let init_len = self.output.position();
        write!(self.output, "{key}: ")?;
        match self.format(value) {
            empty_err @ Err(Error::EmptyContent) => {
                self.output.truncate(init_len);
                empty_err
            }
            other => other,
        }
    }

    fn format_normal_instance(&mut self, instance: &Rc<Instance<'ast, 'a>>) -> Result<()> {
        instance
            .field(RENDERED)
            .or_else(|| instance.attribute(&mut self.evaluator, RENDERED))
            .map(|rendered| self.format(&rendered))
            .expect("rendered is not defined")
    }
}
