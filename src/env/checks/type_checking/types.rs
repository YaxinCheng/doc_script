use crate::ast::{ConstantDeclaration, Field, StructDeclaration, TraitDeclaration};
use crate::env::TypedElement;
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, Eq)]
pub enum Types<'ast, 'a> {
    Primitive(Primitive),
    Struct(&'ast StructDeclaration<'a>),
    Trait(&'ast TraitDeclaration<'a>),

    PrimitiveCollection(Primitive),
    StructCollection(&'ast StructDeclaration<'a>),
    TraitCollection(&'ast TraitDeclaration<'a>),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Primitive {
    Void,
    Int,
    Float,
    Bool,
    String,
}

impl<'ast, 'a> PartialEq for Types<'ast, 'a> {
    fn eq(&self, other: &Self) -> bool {
        use Types::*;
        match (self, other) {
            (Types::Primitive(self_primitive), Types::Primitive(other_primitive))
            | (
                Types::PrimitiveCollection(self_primitive),
                Types::PrimitiveCollection(other_primitive),
            ) => self_primitive == other_primitive,
            (Struct(self_struct), Struct(other_struct))
            | (StructCollection(self_struct), StructCollection(other_struct)) => {
                std::ptr::eq(*self_struct, *other_struct)
            }
            (Trait(self_trait), Trait(other_trait))
            | (TraitCollection(self_trait), TraitCollection(other_trait)) => {
                std::ptr::eq(*self_trait, *other_trait)
            }
            _ => false,
        }
    }
}

impl<'ast, 'a> Types<'ast, 'a> {
    pub const STRING: Self = Types::Primitive(Primitive::String);
    pub const VOID: Self = Types::Primitive(Primitive::Void);
    pub const INT: Self = Types::Primitive(Primitive::Int);
    pub const FLOAT: Self = Types::Primitive(Primitive::Float);
    pub const BOOL: Self = Types::Primitive(Primitive::Bool);

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
            Self::Primitive(primitive) => write!(f, "{:?}", primitive),

            Self::TraitCollection(r#trait) => write!(f, "[{}]", r#trait.name),
            Self::StructCollection(r#struct) => write!(f, "[{}]", r#struct.name),
            Self::PrimitiveCollection(primitive) => write!(f, "[{:?}]", primitive),
        }
    }
}

// Array related
impl<'ast, 'a> Types<'ast, 'a> {
    pub fn collection_type(self) -> Self {
        match self {
            Self::Primitive(primitive) => Self::PrimitiveCollection(primitive),
            Self::Struct(r#struct) => Self::StructCollection(r#struct),
            Self::Trait(r#trait) => Self::TraitCollection(r#trait),
            _ => self,
        }
    }

    pub fn element_type(self) -> Self {
        match self {
            Self::PrimitiveCollection(primitive) => Self::Primitive(primitive),
            Self::StructCollection(r#struct) => Self::Struct(r#struct),
            Self::TraitCollection(r#trait) => Self::Trait(r#trait),
            _ => self,
        }
    }
}
