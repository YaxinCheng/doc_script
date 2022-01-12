use super::instance_access_evaluator::InstanceAccessEvaluator;
use super::instance_evaluator::InstanceEvaluator;
use super::literal_evaluator;
use super::struct_evaluator::{Struct, StructEvaluator};
use super::value::Value;
use crate::ast::{
    Accessor, Block, Expression, Name, Parameter, Statement, StructDeclaration, StructInitContent,
};
use crate::env::{Environment, Resolved};
use std::collections::HashMap;
use std::rc::Rc;

pub struct ExpressionEvaluator<'ast, 'a, 'env> {
    env: &'env Environment<'ast, 'a>,
    resolved_struct: HashMap<&'ast StructDeclaration<'a>, Rc<Struct<'ast, 'a>>>,
}

macro_rules! cached {
    ($map: expr, $key: expr, $loader: expr) => {
        match $map.get($key) {
            Some(cached) => cached.clone(),
            None => {
                let loaded = $loader($key);
                $map.insert($key, loaded.clone());
                loaded
            }
        }
    };
}

impl<'ast, 'a, 'env> ExpressionEvaluator<'ast, 'a, 'env> {
    pub fn with_environment(env: &'env Environment<'ast, 'a>) -> Self {
        Self {
            env,
            resolved_struct: HashMap::new(),
        }
    }

    pub fn evaluate(
        &mut self,
        expression: &'ast Expression<'a>,
        self_ref: Option<Value<'ast, 'a>>,
    ) -> Value<'ast, 'a> {
        match expression {
            Expression::Void => Value::Void,
            Expression::ConstUse(name) => self.evaluate_name(name, self_ref),
            Expression::Literal { kind, lexeme } => literal_evaluator::evaluate(kind, lexeme),
            Expression::StructInit {
                name,
                parameters,
                init_content,
            } => self.evaluate_struct_initialization(name, parameters, init_content, self_ref),
            Expression::FieldAccess {
                receiver,
                field_names,
            } => self.evaluate_field_access(receiver, field_names, self_ref),
            Expression::ChainingMethodInvocation {
                receiver,
                accessors,
            } => self.evaluate_chaining_methods(receiver, accessors, self_ref),
            Expression::Block(block) => self.evaluate_block(block, self_ref),
            Expression::SelfRef(_) => Self::evaluate_self(self_ref),
        }
    }

    fn evaluate_name(
        &mut self,
        name: &'ast Name<'a>,
        self_ref: Option<Value<'ast, 'a>>,
    ) -> Value<'ast, 'a> {
        let resolved = self
            .env
            .resolved_names
            .get(name)
            .unwrap_or_else(|| panic!("name `{}` is not resolved", name));
        match resolved {
            Resolved::Constant(constant) => self.evaluate(&constant.value, self_ref),
            Resolved::InstanceAccess(receiver, accesses) => {
                InstanceAccessEvaluator::new(self, self_ref).evaluate(receiver, accesses)
            }
            _ => unreachable!("name `{}` is not resolved to constant or field", name),
        }
    }

    fn evaluate_struct_initialization(
        &mut self,
        name: &'ast Name<'a>,
        parameters: &'ast [Parameter<'a>],
        init_content: &'ast Option<StructInitContent<'a>>,
        self_ref: Option<Value<'ast, 'a>>,
    ) -> Value<'ast, 'a> {
        let struct_definition = self
            .env
            .resolved_names
            .get(name)
            .unwrap_or_else(|| panic!("name `{}` is not resolved", name));
        let struct_declaration = match struct_definition {
            Resolved::Struct(definition) => *definition,
            _ => unreachable!("name `{}` is not resolved to struct", name),
        };
        let structure = cached!(self.resolved_struct, struct_declaration, |declaration| {
            StructEvaluator(self).evaluate(declaration)
        });
        let instance = InstanceEvaluator::new(self, self_ref).evaluate(
            structure,
            &struct_declaration.fields,
            parameters,
            init_content,
        );
        Value::Instance(Rc::new(instance))
    }

    fn evaluate_field_access(
        &mut self,
        receiver: &'ast Expression<'a>,
        access_names: &'ast [&'a str],
        self_ref: Option<Value<'ast, 'a>>,
    ) -> Value<'ast, 'a> {
        let mut value = self.evaluate(receiver, self_ref);
        for name in access_names {
            value = match value {
                Value::Instance(instance) => instance
                    .field(name)
                    .or_else(|| instance.attribute(self, name))
                    .unwrap_or_else(|| panic!("field `{}` does not exist", name)),
                _ => unreachable!("Field access can only happen on struct instance"),
            }
        }
        value
    }

    fn evaluate_chaining_methods(
        &mut self,
        receiver: &'ast Expression<'a>,
        accessors: &'ast [Accessor<'a>],
        self_ref: Option<Value<'ast, 'a>>,
    ) -> Value<'ast, 'a> {
        let mut value = self.evaluate(receiver, self_ref.clone());
        let instance = match &mut value {
            Value::Instance(instance) => Rc::make_mut(instance),
            _ => unreachable!("Chaining methods can only happen on structure"),
        };
        for Accessor { identifier, value } in accessors {
            if let Some(expression) = value {
                instance.set_field(identifier, self.evaluate(expression, self_ref.clone()))
            } else {
                instance.reset_field(identifier)
            }
        }
        value
    }

    fn evaluate_block(
        &mut self,
        block: &'ast Block<'a>,
        self_ref: Option<Value<'ast, 'a>>,
    ) -> Value<'ast, 'a> {
        match block.statements.last() {
            Some(Statement::ConstantDeclaration(_)) | None => Value::Void,
            Some(Statement::Expression(expr)) => self.evaluate(expr, self_ref),
        }
    }

    fn evaluate_self(self_ref: Option<Value<'ast, 'a>>) -> Value<'ast, 'a> {
        self_ref
            .clone()
            .expect("self does not exist in current scope")
    }
}
