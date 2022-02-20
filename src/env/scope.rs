use crate::ast::{ConstantDeclaration, StructDeclaration, TraitDeclaration};
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
    pub declared: HashMap<&'a str, DeclaredElement<'ast, 'a>>,
}

#[cfg_attr(test, derive(Debug, EnumAsInner))]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum DeclaredElement<'ast, 'a> {
    Constant(&'ast ConstantDeclaration<'a>),
    Struct(&'ast StructDeclaration<'a>),
    Trait(&'ast TraitDeclaration<'a>),
}

impl<'ast, 'a> From<&'ast ConstantDeclaration<'a>> for DeclaredElement<'ast, 'a> {
    fn from(constant: &'ast ConstantDeclaration<'a>) -> Self {
        Self::Constant(constant)
    }
}

impl<'ast, 'a> From<&'ast StructDeclaration<'a>> for DeclaredElement<'ast, 'a> {
    fn from(structure: &'ast StructDeclaration<'a>) -> Self {
        Self::Struct(structure)
    }
}

impl<'ast, 'a> From<&'ast TraitDeclaration<'a>> for DeclaredElement<'ast, 'a> {
    fn from(structure: &'ast TraitDeclaration<'a>) -> Self {
        Self::Trait(structure)
    }
}

impl<'ast, 'a> PartialEq<TraitDeclaration<'a>> for DeclaredElement<'ast, 'a> {
    fn eq(&self, other: &TraitDeclaration<'a>) -> bool {
        let trait_declaration = match self {
            DeclaredElement::Trait(trait_declaration) => trait_declaration,
            _ => return false,
        };
        std::ptr::eq(*trait_declaration, other)
    }
}

impl<'ast, 'a> PartialEq<StructDeclaration<'a>> for DeclaredElement<'ast, 'a> {
    fn eq(&self, other: &StructDeclaration<'a>) -> bool {
        let struct_declaration = match self {
            DeclaredElement::Struct(struct_declaration) => struct_declaration,
            _ => return false,
        };
        std::ptr::eq(*struct_declaration, other)
    }
}

impl<'ast, 'a> PartialEq<ConstantDeclaration<'a>> for DeclaredElement<'ast, 'a> {
    fn eq(&self, other: &ConstantDeclaration<'a>) -> bool {
        let constant_declaration = match self {
            DeclaredElement::Constant(constant_declaration) => constant_declaration,
            _ => return false,
        };
        std::ptr::eq(*constant_declaration, other)
    }
}

impl<'ast, 'a> DeclaredElement<'ast, 'a> {
    pub fn name(&self) -> &str {
        match self {
            DeclaredElement::Constant(constant) => constant.name,
            DeclaredElement::Struct(struct_declaration) => struct_declaration.name,
            DeclaredElement::Trait(trait_declaration) => trait_declaration.name,
        }
    }
}
