use crate::ast::{ConstantDeclaration, Field, StructDeclaration, TraitDeclaration};
use crate::env::TypedElement;
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, Eq)]
pub enum Types<'ast, 'a> {
    Void,
    Int,
    Float,
    Bool,
    String,
    Struct(&'ast StructDeclaration<'a>),
    Trait(&'ast TraitDeclaration<'a>),
}

impl<'ast, 'a> PartialEq for Types<'ast, 'a> {
    fn eq(&self, other: &Self) -> bool {
        use Types::*;
        match (self, other) {
            (Int, Int) | (Float, Float) | (Void, Void) | (Bool, Bool) | (String, String) => true,
            (Struct(self_struct), Struct(other_struct)) => {
                std::ptr::eq(*self_struct, *other_struct)
            }
            (Trait(self_trait), Trait(other_trait)) => std::ptr::eq(*self_trait, *other_trait),
            _ => false,
        }
    }
}

impl<'ast, 'a> Types<'ast, 'a> {
    pub fn access(&self, name: &str) -> Option<TypedElement<'ast, 'a>> {
        self.field(name)
            .map(TypedElement::Field)
            .or_else(|| self.attribute(name).map(TypedElement::Constant))
    }

    pub fn field(&self, name: &str) -> Option<&'ast Field<'a>> {
        match self {
            Types::Struct(r#struct) => &r#struct.fields,
            Types::Trait(r#trait) => &r#trait.required,
            _ => None?,
        }
        .iter()
        .find(|field| field.name == name)
    }

    pub fn attribute(&self, name: &str) -> Option<&'ast ConstantDeclaration<'a>> {
        let struct_body = match self {
            Types::Struct(r#struct) => r#struct.body.as_ref()?,
            _ => None?,
        };
        struct_body
            .attributes
            .iter()
            .find(|constant| constant.name == name)
    }

    pub fn fields(&self) -> &'ast [Field<'a>] {
        match self {
            Self::Struct(struct_declaration) => &struct_declaration.fields,
            Self::Trait(trait_declaration) => &trait_declaration.required,
            _ => &[],
        }
    }
}

impl<'ast, 'a> Display for Types<'ast, 'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trait(r#trait) => write!(f, "{}", r#trait.name),
            Self::Struct(r#struct) => write!(f, "{}", r#struct.name),
            other => write!(f, "{:?}", other),
        }
    }
}
