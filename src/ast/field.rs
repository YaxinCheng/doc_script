use super::check_unpack;
use super::Name;
use super::{Node, NodeKind};
use crate::ast::{debug_check, weeder, Expression};
use crate::search::DepthFirst;
#[cfg(debug_assertions)]
use crate::tokenizer::{Token, TokenKind};

#[derive(Debug, Eq, PartialEq)]
pub struct Field<'a> {
    pub name: &'a str,
    pub field_type: Type<'a>,
    pub default_value: Option<Expression<'a>>,
}

impl<'a> From<Node<'a>> for Field<'a> {
    fn from(node: Node<'a>) -> Self {
        let mut children = check_unpack!(node, NodeKind::PlainField | NodeKind::DefaultField);
        let default_value = Self::eat_default_value(&mut children);
        let field_type = children.pop().map(Type::from).expect("Expect field_type");
        let _colon = children.pop();
        debug_check! { _colon, Some(Node::Leaf(Token { kind: TokenKind::Separator, lexeme: ":" })) };
        let name = children
            .pop()
            .and_then(|node| node.token())
            .map(|token| token.lexeme)
            .expect("Expect field name");
        Field {
            name,
            field_type,
            default_value,
        }
    }
}

impl<'a> Field<'a> {
    fn eat_default_value(children: &mut Vec<Node<'a>>) -> Option<Expression<'a>> {
        match children.last() {
            Some(Node::Internal {
                kind: NodeKind::Expression,
                ..
            }) => {
                let expression = children.pop().map(Expression::from);
                let _equal_sign = children.pop();
                debug_check! { _equal_sign, Some(Node::Leaf( Token { kind: TokenKind::Operator, lexeme: "=" })) };
                expression
            }
            _ => None,
        }
    }

    pub fn find_all_fields(fields_node: Node<'a>) -> Vec<Field<'a>> {
        let fields = DepthFirst::find(
            fields_node,
            |node| {
                matches!(
                    node.kind(),
                    Some(NodeKind::PlainField | NodeKind::DefaultField)
                )
            },
            |node| {
                let mut children = node.children().unwrap_or_default();
                children.reverse();
                children
            },
        )
        .map(Field::from)
        .collect::<Vec<_>>();
        weeder::fields::weed(&fields);
        fields
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Type<'a> {
    pub name: Name<'a>,
    pub is_collection: bool,
}

impl<'a> From<Node<'a>> for Type<'a> {
    fn from(node: Node<'a>) -> Self {
        let mut children = check_unpack!(node, NodeKind::Type);
        let is_collection = children.len() > 1;
        if is_collection {
            let _close_bracket = children.pop();
            debug_check! { _close_bracket, Some(Node::Leaf(Token { kind: TokenKind::Separator, lexeme: "]" })) };
        }
        let name = children.pop().map(Name::from).expect("Expect Name");
        #[cfg(debug_assertions)]
        if is_collection {
            let _open_bracket = children.pop();
            debug_check! { _open_bracket, Some(Node::Leaf(Token { kind: TokenKind::Separator, lexeme: "[" })) };
        }
        Type {
            name,
            is_collection,
        }
    }
}
