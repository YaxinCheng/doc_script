use crate::ast::{ConstantDeclaration, Field};
#[cfg(test)]
use enum_as_inner::EnumAsInner;

#[cfg_attr(test, derive(Debug, EnumAsInner))]
#[derive(Copy, Clone)]
pub enum TypedElement<'ast, 'a> {
    Constant(&'ast ConstantDeclaration<'a>),
    Field(&'ast Field<'a>),
}

impl<'ast, 'a> From<&'ast ConstantDeclaration<'a>> for TypedElement<'ast, 'a> {
    fn from(constant: &'ast ConstantDeclaration<'a>) -> Self {
        Self::Constant(constant)
    }
}

impl<'ast, 'a> From<&'ast Field<'a>> for TypedElement<'ast, 'a> {
    fn from(field: &'ast Field<'a>) -> Self {
        Self::Field(field)
    }
}
