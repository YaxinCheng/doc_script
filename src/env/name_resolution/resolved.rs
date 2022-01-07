use super::super::scope::ScopeId;
use crate::ast::{ConstantDeclaration, StructDeclaration};
use crate::env::scope::DeclaredElement;
#[cfg(test)]
use enum_as_inner::EnumAsInner;

#[cfg_attr(test, derive(Debug, EnumAsInner))]
#[derive(Clone)]
pub enum Resolved<'ast, 'a> {
    Module(ScopeId),
    Constant(&'ast ConstantDeclaration<'a>),
    Struct(&'ast StructDeclaration<'a>),
    InstanceAccess(&'ast ConstantDeclaration<'a>, Vec<&'a str>),
}

impl<'ast, 'a> From<DeclaredElement<'ast, 'a>> for Resolved<'ast, 'a> {
    fn from(declared: DeclaredElement<'ast, 'a>) -> Self {
        match declared {
            DeclaredElement::Struct(r#struct) => Self::Struct(r#struct),
            DeclaredElement::Constant(constant) => Self::Constant(constant),
        }
    }
}
