use crate::ast::Name;
use crate::env::checks::type_checking::types::Types;
use crate::env::Environment;
use crate::env::Resolved;
use crate::tokenizer::LiteralKind;

pub(in crate::env) fn resolve_type_name<'ast, 'a>(
    environment: &Environment<'ast, 'a>,
    name: &'ast Name<'a>,
) -> Option<Types<'ast, 'a>> {
    environment
        .resolved_names
        .get(name)
        .map(|resolved| match resolved {
            Resolved::Struct(r#struct) => Types::Struct(r#struct),
            Resolved::Trait(r#trait) => Types::Trait(r#trait),
            _ => panic!("Type is not valid"),
        })
        .or_else(|| primitive_type(name))
}

fn primitive_type<'ast, 'a>(name: &Name) -> Option<Types<'ast, 'a>> {
    use crate::ast::Moniker;
    match &name.moniker {
        Moniker::Simple("Int") => Some(Types::Int),
        Moniker::Simple("Float") => Some(Types::Float),
        Moniker::Simple("String") => Some(Types::String),
        Moniker::Simple("Bool") => Some(Types::Bool),
        Moniker::Simple("Void") => Some(Types::Void),
        Moniker::Simple("Children") => Some(Types::Children),
        Moniker::Qualified(full_name) => match *full_name.as_ref() {
            ["std", "Int"] => Some(Types::Int),
            ["std", "Float"] => Some(Types::Int),
            ["std", "String"] => Some(Types::Int),
            ["std", "Bool"] => Some(Types::Int),
            ["std", "Void"] => Some(Types::Void),
            ["std", "Children"] => Some(Types::Children),
            _ => None,
        },
        _ => None,
    }
}

pub(in crate::env) fn resolve_literal<'ast, 'a>(literal_kind: &LiteralKind) -> Types<'ast, 'a> {
    match literal_kind {
        LiteralKind::Binary | LiteralKind::Hex | LiteralKind::Integer => Types::Int,
        LiteralKind::Boolean => Types::Bool,
        LiteralKind::Floating => Types::Float,
        LiteralKind::String => Types::String,
    }
}
