use super::{debug_check, weeder, Expression, Field, Node, NodeKind};
use crate::ast::check_unpack;
use crate::ast::scoped_elements::StructBody;
use crate::search::BreadthFirst;
#[cfg(debug_assertions)]
use crate::tokenizer::{Token, TokenKind};

#[derive(Debug, Eq, PartialEq)]
pub struct ConstantDeclaration<'a> {
    pub name: &'a str,
    pub value: Expression<'a>,
}

impl<'a> From<Node<'a>> for ConstantDeclaration<'a> {
    fn from(node: Node<'a>) -> Self {
        let mut children = check_unpack!(node, NodeKind::ConstantDeclaration);
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

#[derive(Debug, Eq, PartialEq)]
pub struct StructDeclaration<'a> {
    pub name: &'a str,
    pub fields: Vec<Field<'a>>,
    pub body: Option<StructBody<'a>>,
}

impl<'a> From<Node<'a>> for StructDeclaration<'a> {
    fn from(node: Node<'a>) -> Self {
        let mut children = check_unpack!(node, NodeKind::StructDeclaration);
        let body = Self::eat_struct_body(&mut children);
        let fields = Self::eat_fields(&mut children);
        let name = children
            .pop()
            .and_then(|node| node.token())
            .map(|token| token.lexeme)
            .expect("Expect struct name");
        let struct_declaration = StructDeclaration { name, fields, body };
        weeder::structure::weed(&struct_declaration);
        struct_declaration
    }
}

impl<'a> StructDeclaration<'a> {
    fn eat_struct_body(children: &mut Vec<Node<'a>>) -> Option<StructBody<'a>> {
        if !matches!(
            children.last().and_then(Node::kind),
            Some(NodeKind::StructBody)
        ) {
            None
        } else {
            let body = BreadthFirst::find(
                children.pop().unwrap(),
                |node| matches!(node.kind(), Some(NodeKind::ConstantDeclaration)),
                |node| node.children().unwrap_or_default(),
            )
            .map(ConstantDeclaration::from)
            .collect::<StructBody>();
            if body.attributes.is_empty() {
                None
            } else {
                Some(body)
            }
        }
    }

    fn eat_fields(children: &mut Vec<Node<'a>>) -> Vec<Field<'a>> {
        if !matches!(
            children.last(),
            Some(Node::Leaf(Token {
                kind: TokenKind::Separator,
                lexeme: ")"
            }))
        ) {
            return vec![];
        }
        let _close_bracket = children.pop();
        debug_check! { _close_bracket, Some(Node::Leaf(Token { kind: TokenKind::Separator, lexeme: ")" })) }
        let fields = children
            .pop()
            .map(Field::find_all_fields)
            .expect("Expect Fields");
        let _open_bracket = children.pop();
        debug_check! { _open_bracket, Some(Node::Leaf(Token { kind: TokenKind::Separator, lexeme: "(" })) };
        fields
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct StructInitContent<'a>(pub Vec<Expression<'a>>);

impl<'a> From<Node<'a>> for StructInitContent<'a> {
    fn from(node: Node<'a>) -> Self {
        let children = check_unpack!(node, NodeKind::StructInitContent);
        let mut expressions = BreadthFirst::find_from(
            children,
            |node| matches!(node.kind(), Some(NodeKind::Expression)),
            |node| node.children().unwrap_or_default(),
        )
        .map(Expression::from)
        .collect::<Vec<_>>();
        expressions.reverse();
        StructInitContent(expressions)
    }
}

#[cfg(test)]
impl<'a> From<Vec<Expression<'a>>> for StructInitContent<'a> {
    fn from(expressions: Vec<Expression<'a>>) -> Self {
        Self(expressions)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct TraitDeclaration<'a> {
    pub name: &'a str,
    pub required: Vec<Field<'a>>,
}

impl<'a> From<Node<'a>> for TraitDeclaration<'a> {
    fn from(node: Node<'a>) -> Self {
        let mut children = check_unpack!(node, NodeKind::TraitDeclaration);
        let required = children
            .pop()
            .map(Self::eat_fields)
            .expect("TraitRequirement");
        let name = children
            .pop()
            .and_then(|node| node.token())
            .map(|token| token.lexeme)
            .expect("Expect trait name");
        Self { name, required }
    }
}

impl<'a> TraitDeclaration<'a> {
    fn eat_fields(requirement: Node<'a>) -> Vec<Field<'a>> {
        let mut children = check_unpack!(requirement, NodeKind::TraitRequirement);
        if children.is_empty() {
            return vec![];
        }
        let _close_bracket = children.pop();
        debug_check! { _close_bracket, Some(Node::Leaf(Token { kind: TokenKind::Separator, lexeme: ")" })) };
        match children.pop() {
            Some(Node::Leaf(Token {
                kind: TokenKind::Separator,
                lexeme: "(",
            })) => vec![],
            Some(fields_node) => {
                let fields = Field::find_all_fields(fields_node);
                let _open_bracket = children.pop();
                debug_check! { _open_bracket, Some(Node::Leaf(Token { kind: TokenKind::Separator, lexeme: "(" })) };
                fields
            }
            None => unreachable!("Brackets are balanced"),
        }
    }
}
