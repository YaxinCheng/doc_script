use super::super::scope::ScopeId;
use crate::ast::{ConstantDeclaration, Field, StructDeclaration};
use crate::env::scope::DeclaredElement;
#[cfg(test)]
use enum_as_inner::EnumAsInner;

#[cfg_attr(test, derive(Debug, EnumAsInner))]
#[derive(Copy, Clone)]
pub enum Resolved<'ast, 'a> {
    Module(ScopeId),
    Constant(&'ast ConstantDeclaration<'a>),
    Struct(&'ast StructDeclaration<'a>),
    Field(&'ast Field<'a>),
}

impl<'ast, 'a> From<DeclaredElement<'ast, 'a>> for Resolved<'ast, 'a> {
    fn from(declared: DeclaredElement<'ast, 'a>) -> Self {
        match declared {
            DeclaredElement::Struct(r#struct) => Self::Struct(r#struct),
            DeclaredElement::Field(field) => Self::Field(field),
            DeclaredElement::Constant(constant) => Self::Constant(constant),
        }
    }
}
