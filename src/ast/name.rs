use super::{Node, NodeKind};
use crate::env::scope::*;
use crate::search::BreadthFirst;
#[cfg(debug_assertions)]
use crate::tokenizer::{Token, TokenKind};
use scope_macro::Scoped;
use std::fmt::{Display, Formatter};

#[cfg_attr(test, derive(Ord, PartialOrd))]
#[derive(PartialEq, Eq, Clone, Scoped, Hash, Debug)] // derive(Hash) assumes scope is always not None
pub struct Name<'a> {
    pub moniker: Moniker<'a>,
    scope: Option<ScopeId>,
}

#[cfg_attr(test, derive(Ord, PartialOrd))]
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Moniker<'a> {
    Simple(&'a str),
    Qualified(Box<[&'a str]>),
}

impl<'a> Name<'a> {
    pub const fn simple(text: &'a str) -> Self {
        Name {
            moniker: Moniker::Simple(text),
            scope: None,
        }
    }

    pub fn qualified(names: Vec<&'a str>) -> Self {
        Name {
            moniker: Moniker::Qualified(names.into_boxed_slice()),
            scope: None,
        }
    }
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
                Name::simple(leaf.lexeme)
            }
            Node::Internal {
                kind: NodeKind::QualifiedName,
                children,
            } => {
                let mut names = BreadthFirst::find_from(
                    children,
                    |node| node.is_leaf(),
                    |node| node.children().unwrap_or_default(),
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
                Name::qualified(names)
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

impl<'a> Display for Name<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.moniker)
    }
}

impl<'a> Display for Moniker<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Moniker::Simple(name) => write!(f, "{}", name),
            Moniker::Qualified(names) => write!(f, "{}", names.join(".")),
        }
    }
}

impl<'a> AsRef<[&'a str]> for Moniker<'a> {
    fn as_ref(&self) -> &[&'a str] {
        self.as_slice()
    }
}

impl<'a> Moniker<'a> {
    pub fn as_slice(&self) -> &[&'a str] {
        match self {
            Moniker::Simple(name) => std::slice::from_ref(name),
            Moniker::Qualified(names) => names,
        }
    }
}
