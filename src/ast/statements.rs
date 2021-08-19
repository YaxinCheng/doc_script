use super::Expression;
use super::{check_unpack, debug_check};
use super::{Node, NodeKind};
#[cfg(debug_assertions)]
use crate::tokenizer::{Token, TokenKind};

#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub struct Statement<'a>(Expression<'a>);

impl<'a> From<Node<'a>> for Statement<'a> {
    fn from(node: Node<'a>) -> Self {
        let mut children = check_unpack!(
            node,
            NodeKind::Statement | NodeKind::ConstantDeclarationStatement
        );
        let _end_of_line = children.pop();
        debug_check! { _end_of_line, Some(Node::Leaf( Token { kind: TokenKind::Separator, lexeme: ";" })) };
        let expression = children
            .pop()
            .map(Expression::from)
            .expect("Expect Expression");
        Statement(expression)
    }
}

impl<'a> From<Statement<'a>> for Expression<'a> {
    fn from(stmt: Statement<'a>) -> Self {
        stmt.0
    }
}
