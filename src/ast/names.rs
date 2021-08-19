use super::{check_unpack, debug_check};
use super::{Expression, Node, NodeKind};
use crate::search::BreadthFirst;
#[cfg(debug_assertions)]
use crate::tokenizer::{Token, TokenKind};

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub enum Name<'a> {
    Simple(&'a str),
    Qualified(Vec<&'a str>),
}

impl<'a> From<Node<'a>> for Name<'a> {
    fn from(node: Node<'a>) -> Self {
        match node {
            Node::Internal {
                kind: NodeKind::SimpleName,
                mut children,
            } => {
                let leaf = children
                    .pop()
                    .expect("Expect Leaf")
                    .token()
                    .expect("Expect Token");
                Name::Simple(leaf.lexeme)
            }
            Node::Internal {
                kind: NodeKind::QualifiedName,
                children,
            } => {
                let mut names = BreadthFirst::find_from(
                    children,
                    |node| node.is_leaf(),
                    |node| node.children_owned().unwrap_or_default(),
                )
                .filter_map(|node| match node.token()? {
                    Token {
                        kind: TokenKind::Identifier,
                        lexeme,
                    } => Some(lexeme),
                    _ => None,
                })
                .collect::<Vec<_>>();
                names.reverse();
                Name::Qualified(names)
            }
            Node::Internal {
                kind: NodeKind::Name,
                mut children,
            } => children
                .pop()
                .map(Name::from)
                .expect("Name should have one child"),
            kind => unreachable!("Unexpected kind: {:?}", kind),
        }
    }
}

#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub enum Parameter<'a> {
    Plain(Expression<'a>),
    Labelled {
        label: &'a str,
        content: Expression<'a>,
    },
}

impl<'a> From<Node<'a>> for Parameter<'a> {
    fn from(node: Node<'a>) -> Self {
        let mut children = check_unpack!(node, NodeKind::Parameter);
        let expression = children
            .pop()
            .map(Expression::from)
            .expect("Expect Expression");
        if children.is_empty() {
            Parameter::Plain(expression)
        } else {
            let _colon = children.pop();
            debug_check! { _colon, Some(Node::Leaf(Token { kind: TokenKind::Separator, lexeme: ":" })) };
            let label = children
                .pop()
                .and_then(|node| node.token())
                .expect("Expect Identifier")
                .lexeme;
            Parameter::Labelled {
                label,
                content: expression,
            }
        }
    }
}
