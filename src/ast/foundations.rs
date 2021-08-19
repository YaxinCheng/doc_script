use super::check_unpack;
use super::Expression;
use super::Import;
use super::{Node, NodeKind};
use crate::ast::Statement;
use crate::search::BreadthFirst;

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct CompilationUnit<'a> {
    pub(crate) declarations: Vec<Declaration<'a>>,
}

impl<'a> From<Node<'a>> for CompilationUnit<'a> {
    fn from(node: Node<'a>) -> Self {
        let children = check_unpack!(node, NodeKind::CompilationUnit);
        let declarations = BreadthFirst::find_from(
            children,
            |node| matches!(node.kind(), Some(NodeKind::Declaration)),
            |node| node.children_owned().unwrap_or_default(),
        )
        .map(Declaration::from)
        .collect();
        CompilationUnit { declarations }
    }
}

#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub enum Declaration<'a> {
    Import(Import<'a>),
    Constant(Expression<'a>),
}

impl<'a> From<Node<'a>> for Declaration<'a> {
    fn from(node: Node<'a>) -> Self {
        match node {
            Node::Internal {
                kind: NodeKind::ImportDeclaration,
                mut children,
            } => children
                .pop()
                .map(Import::from)
                .map(Declaration::Import)
                .expect("ImportDeclaration should have one child"),
            Node::Internal {
                kind: NodeKind::ConstantDeclaration,
                mut children,
            } => children
                .pop()
                .map(Statement::from)
                .map(Expression::from)
                .map(Declaration::Constant)
                .expect("ConstantDeclaration should have one child"),
            Node::Internal {
                kind: NodeKind::Declaration,
                mut children,
            } => children
                .pop()
                .map(Declaration::from)
                .expect("Declaration should have one child"),
            kind => unreachable!("Unexpected kind: {:?}", kind),
        }
    }
}
