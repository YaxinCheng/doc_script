use super::instance_evaluator::Instance;
use std::borrow::Cow;
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
