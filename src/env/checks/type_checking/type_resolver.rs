use crate::ast::Name;
use crate::env::checks::type_checking::types::Types;
use crate::env::Environment;
use crate::env::Resolved;
use crate::tokenizer::LiteralKind;

pub(in crate::env) fn resolve_type_name<'ast, 'a>(
    environment: &Environment<'ast, 'a>,
    name: &'ast Name<'a>,
    is_collection: bool,
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
        .map(|found_type| {
            if is_collection {
                found_type.collection_type()
            } else {
                found_type
            }
        })
}

fn primitive_type<'ast, 'a>(name: &Name) -> Option<Types<'ast, 'a>> {
    use crate::ast::Moniker;
    match &name.moniker {
        Moniker::Simple("Int") => Some(Types::INT),
        Moniker::Simple("Float") => Some(Types::FLOAT),
        Moniker::Simple("String") => Some(Types::STRING),
        Moniker::Simple("Bool") => Some(Types::BOOL),
        Moniker::Simple("Void") => Some(Types::VOID),
        Moniker::Qualified(full_name) => match *full_name.as_ref() {
            ["std", "Int"] => Some(Types::INT),
            ["std", "Float"] => Some(Types::INT),
            ["std", "String"] => Some(Types::INT),
            ["std", "Bool"] => Some(Types::INT),
            ["std", "Void"] => Some(Types::VOID),
            _ => None,
        },
        _ => None,
    }
}

pub(in crate::env) fn resolve_literal<'ast, 'a>(literal_kind: &LiteralKind) -> Types<'ast, 'a> {
    match literal_kind {
        LiteralKind::Binary | LiteralKind::Hex | LiteralKind::Integer => Types::INT,
        LiteralKind::Boolean => Types::BOOL,
        LiteralKind::Floating => Types::FLOAT,
        LiteralKind::String => Types::STRING,
    }
}
