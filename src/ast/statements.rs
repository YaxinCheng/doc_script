use super::debug_check;
use super::Expression;
use super::{Node, NodeKind};
#[cfg(debug_assertions)]
use crate::tokenizer::{Token, TokenKind};

#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub enum Statement<'a> {
    Expression(Expression<'a>),
    ConstantDeclaration {
        name: &'a str,
        value: Expression<'a>,
    },
}

impl<'a> From<Node<'a>> for Statement<'a> {
    fn from(node: Node<'a>) -> Self {
        match node {
            Node::Internal {
                kind: NodeKind::ExpressionStatement,
                mut children,
            } => {
                let _end_of_line = children.pop();
                debug_check! { _end_of_line, Some(Node::Leaf(
                Token { kind: TokenKind::Separator, lexeme: ";" }
                    | Token { kind: TokenKind::NewLine, lexeme: _ })) };
                let expression = children
                    .pop()
                    .map(Expression::from)
                    .expect("Expect Expression");
                Statement::Expression(expression)
            }
            Node::Internal {
                kind: NodeKind::ConstantDeclarationStatement,
                mut children,
            } => {
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
                    .map(|token| match token {
                        Token {
                            kind: TokenKind::Identifier,
                            lexeme,
                        } => lexeme,
                        token => unreachable!("Unexpected token: {:?}", token),
                    })
                    .expect("Failed to find name for constant");
                Statement::ConstantDeclaration { name, value }
            }
            node => unreachable!("Unexpected node reached: {:?}", node),
        }
    }
}
