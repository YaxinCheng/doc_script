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
    Single(Name<'a>),
    Multiple {
        prefix: Name<'a>,
        suffices: Vec<Name<'a>>,
    },
    Wildcard(Name<'a>),
}

impl<'a> From<Node<'a>> for Import<'a> {
    fn from(node: Node<'a>) -> Self {
        match node {
            Node::Internal {
                kind: NodeKind::SingleImportDeclarationStatement,
                mut children,
            } => Import::Single(Name::from(
                children.pop().expect("Import should have one child"),
            )),
            Node::Internal {
                kind: NodeKind::WildcardImportDeclarationStatement,
                mut children,
            } => Import::Wildcard(Name::from(children.swap_remove(1))),
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
                let prefix = children.pop().map(Name::from).expect("Expect Name");
                let suffices = BreadthFirst::find(
                    suffices,
                    |node| matches!(node.kind(), Some(NodeKind::Name)),
                    |node| node.children().unwrap_or_default(),
                )
                .map(Name::from)
                .collect();
                Import::Multiple { prefix, suffices }
            }
            kind => unreachable!("Unexpected kind: {:?}", kind),
        }
    }
}
