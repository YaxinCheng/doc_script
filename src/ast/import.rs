use super::debug_check;
use super::Name;
use super::{Node, NodeKind};
use crate::search::BreadthFirst;
#[cfg(debug_assertions)]
use crate::tokenizer::{Token, TokenKind};
#[cfg(test)]
use enum_as_inner::EnumAsInner;

#[cfg_attr(test, derive(Debug, Eq, PartialEq, EnumAsInner))]
pub enum Import<'a> {
    Single(Vec<&'a str>),
    Multiple {
        prefix: Vec<&'a str>,
        suffices: Vec<Vec<&'a str>>,
    },
    Wildcard(Vec<&'a str>),
}

impl<'a> From<Node<'a>> for Import<'a> {
    fn from(node: Node<'a>) -> Self {
        match node {
            Node::Internal {
                kind: NodeKind::SingleImportDeclarationStatement,
                mut children,
            } => Import::Single(Name::find_raw_name_lexeme(
                children.pop().expect("Import should have one child"),
            )),
            Node::Internal {
                kind: NodeKind::WildcardImportDeclarationStatement,
                mut children,
            } => Import::Wildcard(Name::find_raw_name_lexeme(children.swap_remove(1))),
            Node::Internal {
                kind: NodeKind::MultipleImportDeclarationStatement,
                mut children,
            } => {
                let _close_brackets = children.pop();
                debug_check! { _close_brackets, Some(Node::Leaf(Token { kind: TokenKind::Separator, lexeme: "}" })) };
                let suffices = children.pop().expect("Expect CommaSeparatedNames");
                let _open_brackets = children.pop();
                debug_check! { _open_brackets, Some(Node::Leaf(Token { kind: TokenKind::Separator, lexeme: "{" })) };
                let _dot = children.pop();
                debug_check! { _dot, Some(Node::Leaf(Token { kind: TokenKind::Separator, lexeme: "." })) };
                let prefix = children
                    .pop()
                    .map(Name::find_raw_name_lexeme)
                    .expect("Expect Name");
                let suffices = BreadthFirst::find(
                    suffices,
                    |node| matches!(node.kind(), Some(NodeKind::Name)),
                    |node| node.children().unwrap_or_default(),
                )
                .map(Name::find_raw_name_lexeme)
                .collect();
                Import::Multiple { prefix, suffices }
            }
            Node::Internal {
                kind: NodeKind::ImportDeclaration,
                mut children,
            } => children
                .pop()
                .map(Self::from)
                .expect("ImportDeclaration should have one child"),
            kind => unreachable!("Unexpected kind: {:?}", kind),
        }
    }
}
