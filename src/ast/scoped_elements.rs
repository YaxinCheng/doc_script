use crate::ast::{check_unpack, ConstantDeclaration, Expression, Statement};
use crate::env::scope::*;
use crate::parser::{Node, NodeKind};
use crate::search::BreadthFirst;
use scope_macro::Scoped;

#[derive(Scoped)]
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub struct Block<'a> {
    pub statements: Vec<Statement<'a>>,
    scope: Option<ScopeId>,
}

impl<'a> FromIterator<Statement<'a>> for Block<'a> {
    fn from_iter<T: IntoIterator<Item = Statement<'a>>>(iter: T) -> Self {
        let statements = iter.into_iter().collect();
        Self {
            statements,
            scope: None,
        }
    }
}

#[derive(Scoped)]
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub struct StructInitContent<'a> {
    pub expressions: Vec<Expression<'a>>,
    scope: Option<ScopeId>,
}

impl<'a> From<Node<'a>> for StructInitContent<'a> {
    fn from(node: Node<'a>) -> Self {
        let children = check_unpack!(node, NodeKind::StructInitContent);
        let mut expressions = BreadthFirst::find_from(
            children,
            |node| matches!(node.kind(), Some(NodeKind::Expression)),
            |node| node.children().unwrap_or_default(),
        )
        .map(Expression::from)
        .collect::<Vec<_>>();
        expressions.reverse();
        StructInitContent {
            expressions,
            scope: None,
        }
    }
}

#[cfg(test)]
impl<'a> From<Vec<Expression<'a>>> for StructInitContent<'a> {
    fn from(expressions: Vec<Expression<'a>>) -> Self {
        Self {
            expressions,
            scope: None,
        }
    }
}

#[derive(Default, Scoped)]
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub struct StructBody<'a> {
    pub attributes: Vec<ConstantDeclaration<'a>>,
    scope: Option<ScopeId>,
}

impl<'a> FromIterator<ConstantDeclaration<'a>> for StructBody<'a> {
    fn from_iter<T: IntoIterator<Item = ConstantDeclaration<'a>>>(iter: T) -> Self {
        let declarations = iter.into_iter().collect();
        Self {
            attributes: declarations,
            scope: None,
        }
    }
}

#[cfg(test)]
impl<'a> From<Vec<ConstantDeclaration<'a>>> for StructBody<'a> {
    fn from(declarations: Vec<ConstantDeclaration<'a>>) -> Self {
        Self {
            attributes: declarations,
            scope: None,
        }
    }
}
