use crate::tokenizer::{Token, TokenKind};

include!(concat!(env!("OUT_DIR"), "/node_kind.rs"));

#[derive(Debug)]
pub enum Node<'a> {
    Internal {
        kind: NodeKind,
        children: Vec<Node<'a>>,
    },
    Leaf(Token<'a>),
}

impl<'a> Node<'a> {
    #[cfg(test)]
    pub fn children(&self) -> &[Node<'a>] {
        match self {
            Node::Leaf(_) => &[],
            Node::Internal { kind: _, children } => children,
        }
    }

    pub fn token(&self) -> Option<Token<'a>> {
        match self {
            Node::Leaf(token) => Some(*token),
            Node::Internal { .. } => None,
        }
    }

    pub fn kind(&self) -> Option<NodeKind> {
        match self {
            Node::Internal { kind, .. } => Some(*kind),
            Node::Leaf(_) => None,
        }
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self, Node::Leaf(_))
    }

    pub fn children_owned(self) -> Option<Vec<Node<'a>>> {
        match self {
            Node::Internal { kind: _, children } => Some(children),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum Symbol<'a> {
    NonTerminal(NodeKind),
    Terminal(Token<'a>),
}

// Treats number literals of the same kind as equal
impl<'a> PartialEq for Symbol<'a> {
    fn eq(&self, other: &Self) -> bool {
        use super::LiteralKind::{Binary, Floating, Hex, Integer};
        use Symbol::*;
        use TokenKind::Literal;

        match (self, other) {
            (NonTerminal(self_kind), NonTerminal(other_kind)) => self_kind == other_kind,
            (Terminal(self_kind), Terminal(other_kind)) => {
                match (self_kind.kind, other_kind.kind) {
                    (Literal(Integer), Literal(Integer))
                    | (Literal(Floating), Literal(Floating))
                    | (Literal(Hex), Literal(Hex))
                    | (Literal(Binary), Literal(Binary)) => true,
                    (self_kind, other_kind) => self_kind == other_kind,
                }
            }
            _ => false,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Production {
    pub lhs: NodeKind,
    pub rhs: &'static [Symbol<'static>],
}

#[derive(Debug)]
pub struct ParseTree<'a> {
    pub root: Node<'a>,
}

impl<'a> From<Node<'a>> for ParseTree<'a> {
    fn from(node: Node<'a>) -> Self {
        debug_assert!(matches!(
            node,
            Node::Internal {
                kind: NodeKind::CompilationUnit,
                ..
            }
        ));

        ParseTree { root: node }
    }
}
