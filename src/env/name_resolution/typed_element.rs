use crate::ast::{ConstantDeclaration, Field};
#[cfg(test)]
use enum_as_inner::EnumAsInner;

#[cfg_attr(test, derive(Debug, EnumAsInner))]
#[derive(Copy, Clone)]
pub enum TypedElement<'ast, 'a> {
    Constant(&'ast ConstantDeclaration<'a>),
    Field(&'ast Field<'a>),
}

impl<'ast, 'a> PartialEq for TypedElement<'ast, 'a> {
    fn eq(&self, other: &Self) -> bool {
        use TypedElement::*;
        match (self, other) {
            (Constant(constant1), Constant(constant2)) => std::ptr::eq(*constant1, *constant2),
            (Field(field1), Field(field2)) => std::ptr::eq(*field1, *field2),
            _ => false,
        }
    }
}
impl<'ast, 'a> Eq for TypedElement<'ast, 'a> {}

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
