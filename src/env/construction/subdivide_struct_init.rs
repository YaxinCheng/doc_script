use crate::ast::{Accessor, Expression, Name, Parameter};
use crate::env::Environment;

/// This function clarifies StructInit which is ambiguous given the grammar.
/// After this method, StructInit will be the case of calling Struct constructor,
/// the case that creates struct using chaining method will be made to ChainingMethodInvocation
/// explicitly
pub fn subdivide<'ast, 'a>(environment: &Environment<'ast, 'a>, struct_init: &mut Expression<'a>) {
    let (name, parameters, init_content) = match struct_init {
        Expression::StructInit {
            name,
            parameters,
            init_content,
        } => (name, parameters, init_content),
        _ => return,
    };
    if !(init_content.is_some()
        || name.moniker.as_slice().len() == 1
        || parameter_indicates_struct_init(parameters)
        || name_exists_directly_in_module(environment, name))
    {
        let (prefix_name, field_name) = split_name(name);
        let prefix_expression = Box::new(Expression::ConstUse(prefix_name));
        let field_value = parameters.pop().map(Parameter::expression_owned);
        *struct_init = Expression::ChainingMethodInvocation {
            receiver: prefix_expression,
            accessors: vec![Accessor {
                identifier: field_name,
                value: field_value,
            }],
        }
    }
}

fn parameter_indicates_struct_init(parameters: &[Parameter]) -> bool {
    parameters.len() > 1 || (parameters.len() == 1 && parameters[0].is_labelled())
}

/// Structs resides directly in modules. So modules.suffix is definitely a struct
fn name_exists_directly_in_module<'ast, 'a>(
    environment: &Environment<'ast, 'a>,
    name: &'ast Name<'a>,
) -> bool {
    debug_assert!(
        name.moniker.as_slice().len() > 1,
        "Moniker name should be >= 1"
    );
    let (_, prefix) = name.moniker.as_slice().split_last().unwrap();
    environment.find_module(prefix).is_some()
}

// Split name: `Name(a.b.c)` => `Name(a.b), "c"`
fn split_name<'a>(name: &Name<'a>) -> (Name<'a>, &'a str) {
    let (field_name, prefix) = name.moniker.as_slice().split_last().unwrap();
    let prefix_name = match prefix {
        [name] => Name::simple(name),
        prefix => Name::qualified(prefix.to_owned()),
    };
    (prefix_name, field_name)
}
