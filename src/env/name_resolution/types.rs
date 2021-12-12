use crate::ast::{ConstantDeclaration, Field, StructDeclaration};

#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub enum Types<'ast, 'a> {
    Void,
    Int,
    Float,
    Bool,
    String,
    Struct(&'ast StructDeclaration<'a>),
}

impl<'ast, 'a> Types<'ast, 'a> {
    pub fn field(&self, name: &str) -> Option<&'ast Field<'a>> {
        match self {
            Types::Struct(r#struct) => r#struct.fields.iter().find(|field| field.name == name),
            _ => None,
        }
    }

    pub fn attribute(&self, name: &str) -> Option<&'ast ConstantDeclaration<'a>> {
        let struct_body = match self {
            Types::Struct(r#struct) => &r#struct.body,
            _ => None?,
        };
        struct_body
            .attributes
            .iter()
            .find(|constant| constant.name == name)
    }
}
