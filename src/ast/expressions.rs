use super::{check_unpack, debug_check};
use super::{Name, Parameter, Statement};
use super::{Node, NodeKind};
use crate::search::{BreadthFirst, DepthFirst};
use crate::tokenizer::{LiteralKind, Token, TokenKind};

#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub enum Expression<'a> {
    Constant {
        name: &'a str,
        data: Box<Expression<'a>>,
    },
    Block(Vec<Statement<'a>>),
    StructInit {
        name: Name<'a>,
        parameters: Vec<Parameter<'a>>,
        body: Vec<Expression<'a>>,
    },
    Literal {
        kind: LiteralKind,
        lexeme: &'a str,
    },
}

impl<'a> From<Node<'a>> for Expression<'a> {
    fn from(node: Node<'a>) -> Self {
        match node.kind() {
            Some(NodeKind::ConstantDeclarationExpression) => Self::constant_expression(node),
            Some(NodeKind::Block) => Self::block(node),
            Some(NodeKind::Literal) => Self::literal(node),
            Some(NodeKind::StructInitExpression) => Self::struct_init(node),
            Some(NodeKind::Expression) => {
                let child = node
                    .children_owned()
                    .and_then(|mut children| children.pop())
                    .expect("Expression should have one child");
                Expression::from(child)
            }
            None => unreachable!("Unexpected leaf node reached: {:?}", node),
            Some(kind) => unreachable!("Unexpected kind reached: {:?}", kind),
        }
    }
}

impl<'a> Expression<'a> {
    fn constant_expression(node: Node<'a>) -> Expression<'a> {
        let mut children = check_unpack!(node, NodeKind::ConstantDeclarationExpression);
        let value = children
            .pop()
            .map(Expression::from)
            .expect("Expect Expression");
        let _equal_sign = children.pop();
        debug_check! { _equal_sign, Some(Node::Leaf(Token { kind: TokenKind::Operator, lexeme: "=" })) };
        let name = match children.pop() {
            Some(Node::Leaf(Token {
                kind: TokenKind::Identifier,
                lexeme,
            })) => lexeme,
            node => unreachable!(
                "Unexpected node obtained while expecting Identifier: {:?}",
                node
            ),
        };
        Expression::Constant {
            name,
            data: Box::new(value),
        }
    }

    fn block(node: Node<'a>) -> Expression<'a> {
        let mut children = check_unpack!(node, NodeKind::Block);
        let _close_bracket = children.pop();
        debug_check! { _close_bracket, Some(Node::Leaf(Token { kind: TokenKind::Separator, lexeme: "}" })) };
        let statements = children.pop().expect("Expect Statements");
        debug_assert!(
            matches!(statements.kind(), Some(NodeKind::Statements)),
            "Expect Statements, but wrong kind"
        );
        let statements = DepthFirst::find(
            statements,
            |node| matches!(node.kind(), Some(NodeKind::Statement)),
            |node| node.children_owned().unwrap_or_default(),
        )
        .map(Statement::from)
        .collect();
        Expression::Block(statements)
    }

    fn literal(node: Node<'a>) -> Expression<'a> {
        let mut children = check_unpack!(node, NodeKind::Literal);
        let token = children
            .pop()
            .expect("Empty children for literal")
            .token()
            .expect("Literal should have tokens");
        match token {
            Token {
                kind: TokenKind::Literal(literal_kind),
                lexeme,
            } => Expression::Literal {
                kind: literal_kind,
                lexeme,
            },
            token => unreachable!("Unexpected non-literal token: {:?}", token),
        }
    }

    fn struct_init(node: Node<'a>) -> Expression<'a> {
        let mut children = check_unpack!(node, NodeKind::StructInitExpression);
        let body = Self::eat_struct_body_init(&mut children);
        let parameters = Self::eat_parameters(&mut children);
        let name = children.pop().map(Name::from).expect("Name is missing");
        Expression::StructInit {
            name,
            parameters,
            body,
        }
    }

    fn eat_struct_body_init(nodes: &mut Vec<Node<'a>>) -> Vec<Expression<'a>> {
        match nodes.last().and_then(Node::kind) {
            Some(NodeKind::ArrayLiteral) => BreadthFirst::find(
                nodes.pop().unwrap(),
                |node| matches!(node.kind(), Some(NodeKind::Expression)),
                |node| node.children_owned().unwrap_or_default(),
            )
            .map(Expression::from)
            .collect(),
            _ => vec![],
        }
    }

    fn eat_parameters(nodes: &mut Vec<Node<'a>>) -> Vec<Parameter<'a>> {
        let has_close_bracket = matches!(
            nodes.last().and_then(Node::token),
            Some(Token {
                kind: TokenKind::Separator,
                lexeme: ")"
            })
        );
        if has_close_bracket {
            let _close_bracket = nodes.pop();
            debug_check! { _close_bracket, Some(Node::Leaf(Token { kind: TokenKind::Separator, lexeme: ")" })) };
            let parameters = nodes.pop().expect("Expect parameter or open bracket");
            let parameters = BreadthFirst::find(
                parameters,
                |node| matches!(node.kind(), Some(NodeKind::Parameter)),
                |node| node.children_owned().unwrap_or_default(),
            )
            .map(Parameter::from)
            .collect();
            let _open_bracket = nodes.pop();
            debug_check! { _open_bracket, Some(Node::Leaf(Token { kind: TokenKind::Separator, lexeme: "(" })) };
            parameters
        } else {
            vec![]
        }
    }
}
