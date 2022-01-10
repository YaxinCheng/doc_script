use super::{check_unpack, debug_check, Expression, Node, NodeKind};
#[cfg(debug_assertions)]
use crate::tokenizer::{Token, TokenKind};

#[derive(Debug, Eq, PartialEq)]
pub enum Parameter<'a> {
    Plain(Expression<'a>),
    Labelled {
        label: &'a str,
        content: Expression<'a>,
    },
}

impl<'a> From<Node<'a>> for Parameter<'a> {
    fn from(node: Node<'a>) -> Self {
        let mut children = check_unpack!(
            node,
            NodeKind::NamedParameter | NodeKind::PositionalParameter
        );
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

impl<'a> Parameter<'a> {
    pub fn is_labelled(&self) -> bool {
        match self {
            Parameter::Plain(_) => false,
            Parameter::Labelled { .. } => true,
        }
    }

    pub fn expression(&self) -> &Expression<'a> {
        match self {
            Parameter::Plain(content) | Parameter::Labelled { label: _, content } => content,
        }
    }

    pub fn expression_owned(self) -> Expression<'a> {
        match self {
            Parameter::Plain(content) | Parameter::Labelled { label: _, content } => content,
        }
    }
}
