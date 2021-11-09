use super::{check_unpack, debug_check};
use super::{Name, Parameter, Statement};
use super::{Node, NodeKind};
use crate::search::{BreadthFirst, DepthFirst};
use crate::tokenizer::{LiteralKind, Token, TokenKind};

#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub enum Expression<'a> {
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
    MethodInvocation {
        name: Name<'a>,
        parameters: Vec<Parameter<'a>>,
    },
    ChainingMethodInvocation {
        receiver: Box<Expression<'a>>,
        name: &'a str,
        parameters: Vec<Parameter<'a>>,
    },
}

impl<'a> From<Node<'a>> for Expression<'a> {
    fn from(node: Node<'a>) -> Self {
        match node.kind() {
            Some(NodeKind::Block) => Self::block(node),
            Some(NodeKind::Literal) => Self::literal(node),
            Some(NodeKind::StructInitExpression) => Self::struct_init(node),
            Some(NodeKind::MethodInvocation) => Self::method_invocation(node),
            Some(NodeKind::ChainingMethodInvocation) => Self::chaining_method_invocation(node),
            Some(NodeKind::Expression) => {
                let child = node
                    .children()
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
            |node| node.children().unwrap_or_default(),
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

    fn method_invocation(node: Node<'a>) -> Expression<'a> {
        let mut children = check_unpack!(node, NodeKind::MethodInvocation);
        let parameters = Self::eat_parameters(&mut children);
        let name = children.pop().map(Name::from).expect("Name is missing");
        Expression::MethodInvocation { name, parameters }
    }

    fn chaining_method_invocation(node: Node<'a>) -> Expression<'a> {
        let mut children = check_unpack!(node, NodeKind::ChainingMethodInvocation);
        let parameters = Self::eat_parameters(&mut children);
        let name = children
            .pop()
            .and_then(|node| node.token())
            .map(|token| token.lexeme)
            .expect("Method name is missing");
        let _dot = children.pop();
        debug_check! { _dot, Some(Node::Leaf(Token { kind: TokenKind::Separator, lexeme: "." })) };
        if matches!(
            children.last().and_then(Node::token),
            Some(Token {
                kind: TokenKind::NewLine,
                lexeme: _
            })
        ) {
            children.pop();
        }
        let receiver = children
            .pop()
            .map(Expression::from)
            .map(Box::new)
            .expect("Receiver missing");
        Expression::ChainingMethodInvocation {
            receiver,
            name,
            parameters,
        }
    }

    fn eat_struct_body_init(nodes: &mut Vec<Node<'a>>) -> Vec<Expression<'a>> {
        let last_node = nodes.pop().expect("Expect StructContent");
        let children = check_unpack!(last_node, NodeKind::StructContent);
        let mut body = BreadthFirst::find_from(
            children,
            |node| matches!(node.kind(), Some(NodeKind::Expression)),
            |node| node.children().unwrap_or_default(),
        )
        .map(Expression::from)
        .collect::<Vec<_>>();
        body.reverse();
        body
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
                |node| node.children().unwrap_or_default(),
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
