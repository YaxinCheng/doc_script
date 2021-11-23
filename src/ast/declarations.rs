use super::{debug_check, Expression, Field, Node, NodeKind};
use crate::ast::check_unpack;
use crate::search::BreadthFirst;
#[cfg(debug_assertions)]
use crate::tokenizer::{Token, TokenKind};

#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub struct ConstantDeclaration<'a> {
    pub name: &'a str,
    pub value: Expression<'a>,
}

impl<'a> From<Node<'a>> for ConstantDeclaration<'a> {
    fn from(node: Node<'a>) -> Self {
        let mut children = check_unpack!(node, NodeKind::ConstantDeclaration);
        let _end_of_line = children.pop();
        debug_check! { _end_of_line, Some(Node::Internal { kind: NodeKind::EOL, .. }) }
        let value = children
            .pop()
            .map(Expression::from)
            .expect("Expect Expression");
        let _equal_sign = children.pop();
        debug_check! { _equal_sign, Some(Node::Leaf( Token { kind: TokenKind::Operator, lexeme: "=" })) };
        let name = children
            .pop()
            .and_then(|leaf| leaf.token())
            .map(|token| token.lexeme)
            .expect("Failed to find name for constant");
        ConstantDeclaration { name, value }
    }
}

#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub struct StructDeclaration<'a> {
    pub name: &'a str,
    pub fields: Vec<Field<'a>>,
}

impl<'a> From<Node<'a>> for StructDeclaration<'a> {
    fn from(node: Node<'a>) -> Self {
        let mut children = check_unpack!(node, NodeKind::StructDeclaration);
        let _end_of_line = children.pop();
        debug_check! { _end_of_line, Some(Node::Internal { kind: NodeKind::EOL, .. }) }
        let _close_bracket = children.pop();
        debug_check! { _close_bracket, Some(Node::Leaf(Token { kind: TokenKind::Separator, lexeme: "}" })) }
        let fields = children.pop().expect("Expect Fields");
        let fields = BreadthFirst::find(
            fields,
            |node| matches!(node.kind(), Some(NodeKind::Field)),
            |node| node.children().unwrap_or_default(),
        )
        .map(Field::from)
        .collect::<Vec<_>>();
        let _open_bracket = children.pop();
        debug_check! { _open_bracket, Some(Node::Leaf(Token { kind: TokenKind::Separator, lexeme: "{" })) }
        let name = children
            .pop()
            .and_then(|node| node.token())
            .map(|token| token.lexeme)
            .expect("Expect struct name");
        StructDeclaration { name, fields }
    }
}
