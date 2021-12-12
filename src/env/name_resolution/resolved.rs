use super::super::scope::ScopeId;
use crate::ast::{ConstantDeclaration, Field, StructDeclaration};
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
