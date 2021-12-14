use super::super::scope::ScopeId;
use super::typed_element::TypedElement;
use crate::ast::{ConstantDeclaration, Field, StructDeclaration};
use crate::env::scope::DeclaredElement;
#[cfg(test)]
use enum_as_inner::EnumAsInner;

#[cfg_attr(test, derive(Debug, EnumAsInner))]
#[derive(Clone)]
pub enum Resolved<'ast, 'a> {
    Module(ScopeId),
    Constant(&'ast ConstantDeclaration<'a>),
    Struct(&'ast StructDeclaration<'a>),
    InstanceAccess(Vec<TypedElement<'ast, 'a>>),
    Field(&'ast Field<'a>),
}

impl<'ast, 'a> From<DeclaredElement<'ast, 'a>> for Resolved<'ast, 'a> {
    fn from(declared: DeclaredElement<'ast, 'a>) -> Self {
        match declared {
            DeclaredElement::Struct(r#struct) => Self::Struct(r#struct),
            DeclaredElement::Constant(constant) => Self::Constant(constant),
            DeclaredElement::Field(field) => Self::Field(field),
        }
    }
}
