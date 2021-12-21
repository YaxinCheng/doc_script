use super::check_unpack;
use super::Import;
use super::{Node, NodeKind};
use crate::ast::{debug_check, ConstantDeclaration, StructDeclaration};
use crate::search::BreadthFirst;
#[cfg(test)]
use enum_as_inner::EnumAsInner;

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub struct CompilationUnit<'a> {
    pub(crate) declarations: Vec<Declaration<'a>>,
}

impl<'a> From<Node<'a>> for CompilationUnit<'a> {
    fn from(node: Node<'a>) -> Self {
        let children = check_unpack!(node, NodeKind::CompilationUnit);
        let declarations = BreadthFirst::find_from(
            children,
            |node| matches!(node.kind(), Some(NodeKind::DeclarationStatement)),
            |node| node.children().unwrap_or_default(),
        )
        .map(Declaration::from)
        .collect();
        CompilationUnit { declarations }
    }
}

#[cfg_attr(test, derive(Debug, Eq, PartialEq, EnumAsInner))]
pub enum Declaration<'a> {
    Import(Import<'a>),
    Constant(ConstantDeclaration<'a>),
    Struct(StructDeclaration<'a>),
}

impl<'a> From<Node<'a>> for Declaration<'a> {
    fn from(node: Node<'a>) -> Self {
        match node {
            Node::Internal {
                kind: NodeKind::ImportDeclarationStatement,
                mut children,
            } => {
                let _end_of_line = children.pop();
                debug_check! { _end_of_line, Some(Node::Internal { kind: NodeKind::EOL, .. }) }
                children
                    .pop()
                    .map(Import::from)
                    .map(Declaration::Import)
                    .expect("ImportDeclaration should have one child")
            }
            Node::Internal {
                kind: NodeKind::ConstantDeclarationStatement,
                mut children,
            } => {
                let _end_of_line = children.pop();
                debug_check! { _end_of_line, Some(Node::Internal { kind: NodeKind::EOL, .. }) }
                children
                    .pop()
                    .map(ConstantDeclaration::from)
                    .map(Declaration::Constant)
                    .expect("ConstantDeclaration should have one child")
            }
            Node::Internal {
                kind: NodeKind::StructDeclarationStatement,
                mut children,
            } => {
                let _end_of_line = children.pop();
                debug_check! { _end_of_line, Some(Node::Internal { kind: NodeKind::EOL, .. }) }
                children
                    .pop()
                    .map(StructDeclaration::from)
                    .map(Declaration::Struct)
                    .expect("StructDeclaration should have one child")
            }
            Node::Internal {
                kind: NodeKind::DeclarationStatement,
                mut children,
            } => children
                .pop()
                .map(Declaration::from)
                .expect("Declaration should have one child"),
            kind => unreachable!("Unexpected kind: {:?}", kind),
        }
    }
}
