use crate::ast::{ConstantDeclaration, Field, StructDeclaration};
#[cfg(test)]
use enum_as_inner::EnumAsInner;
use std::collections::{HashMap, HashSet};

pub(crate) type ScopeId = usize;
pub(crate) const GLOBAL_SCOPE: usize = 0;

pub trait Scoped {
    fn set_scope(&mut self, scope: ScopeId);
    fn scope(&self) -> ScopeId;
}

#[derive(Default)]
pub struct Scope<'ast, 'a> {
    pub parent: ScopeId,
    pub id: ScopeId,
    pub name_spaces: NameSpaces<'ast, 'a>,
}

impl<'ast, 'a> Scope<'ast, 'a> {
    #[cfg(not(test))]
    pub fn global() -> Self {
        use std::sync::Once;
        static ONCE: Once = Once::new();
        let mut scope: Option<Scope<'ast, 'a>> = None;
        ONCE.call_once(|| {
            scope.replace(Scope {
                parent: ScopeId::MAX,
                id: 0,
                ..Default::default()
            });
        });
        scope.expect("Global can only be called once")
    }

    #[cfg(test)]
    pub fn global() -> Self {
        Self::default()
    }
}

#[derive(Default)]
pub struct NameSpaces<'ast, 'a> {
    pub modules: HashMap<&'a str, ScopeId>,
    pub wildcard_imports: HashSet<ScopeId>,
    pub expressions: HashMap<Vec<&'a str>, ExpressionDeclaration<'ast, 'a>>,
    pub structs: HashMap<Vec<&'a str>, &'ast StructDeclaration<'a>>,
}

#[cfg_attr(test, derive(Debug, EnumAsInner))]
#[derive(Copy, Clone)]
pub enum ExpressionDeclaration<'ast, 'a> {
    Constant(&'ast ConstantDeclaration<'a>),
    Field(&'ast Field<'a>),
}

impl<'ast, 'a> From<&'ast ConstantDeclaration<'a>> for ExpressionDeclaration<'ast, 'a> {
    fn from(constant: &'ast ConstantDeclaration<'a>) -> Self {
        Self::Constant(constant)
    }
}

impl<'ast, 'a> From<&'ast Field<'a>> for ExpressionDeclaration<'ast, 'a> {
    fn from(field: &'ast Field<'a>) -> Self {
        Self::Field(field)
    }
}