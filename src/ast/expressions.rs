use super::weeder;
use super::{check_unpack, debug_check};
use super::{Name, Parameter, Statement};
use super::{Node, NodeKind};
use crate::ast::scoped_elements::Block;
use crate::ast::StructInitContent;
use crate::search::{BreadthFirst, DepthFirst};
use crate::tokenizer::{LiteralKind, Token, TokenKind};
#[cfg(test)]
use enum_as_inner::EnumAsInner;

#[derive(Debug, Eq, PartialEq)]
pub struct Accessor<'a> {
    pub identifier: &'a str,
    pub value: Option<Expression<'a>>,
}

#[cfg_attr(test, derive(EnumAsInner))]
#[derive(Debug, Eq, PartialEq)]
pub enum Expression<'a> {
    Block(Block<'a>),
    StructInit {
        name: Name<'a>,
        parameters: Vec<Parameter<'a>>,
        init_content: Option<StructInitContent<'a>>,
    },
    Literal {
        kind: LiteralKind,
        lexeme: &'a str,
    },
    ChainingMethodInvocation {
        receiver: Box<Expression<'a>>,
        accessors: Vec<Accessor<'a>>,
    },
    ConstUse(Name<'a>),
    FieldAccess {
        receiver: Box<Expression<'a>>,
        field_names: Vec<&'a str>,
    },
}

impl<'a> From<Node<'a>> for Expression<'a> {
    fn from(node: Node<'a>) -> Self {
        match node.kind() {
            Some(NodeKind::Block) => Self::block(node),
            Some(NodeKind::Literal) => Self::literal(node),
            Some(NodeKind::StructInitExpression) => Self::struct_init(node),
            Some(NodeKind::ChainingMethodInvocation) => Self::chaining_method_invocation(node),
            Some(NodeKind::ConstantUse) => Self::const_use(node),
            Some(NodeKind::FieldAccess) => Self::field_access(node),
            Some(NodeKind::Expression | NodeKind::ChainableExpression) => {
                Self::expression_recursive(node)
            }
            None => unreachable!("Unexpected leaf node reached: {:?}", node),
            Some(kind) => unreachable!("Unexpected kind reached: {:?}", kind),
        }
    }
}

impl<'a> Expression<'a> {
    fn expression_recursive(node: Node<'a>) -> Expression<'a> {
        let mut children = node
            .children()
            .expect("Expression node should have children");
        let child = children.pop().expect("One child expected");
        match child {
            Node::Internal { .. } => Expression::from(child),
            Node::Leaf(Token {
                kind: TokenKind::Keyword,
                lexeme: "self",
            }) => Expression::ConstUse(Name::simple("$self")),
            Node::Leaf(_) => {
                Expression::from(children.pop().expect("Bracketed expression expected"))
            }
        }
    }

    fn block(node: Node<'a>) -> Expression<'a> {
        let mut children = check_unpack!(node, NodeKind::Block);
        let _close_bracket = children.pop();
        debug_check! { _close_bracket, Some(Node::Leaf(Token { kind: TokenKind::Separator, lexeme: "}" })) };
        let (statements, expression) =
            match children.pop().expect("Expect Statements or Expression") {
                expression
                @
                Node::Internal {
                    kind: NodeKind::Expression,
                    ..
                } => (children.pop().expect("Expect Statements"), Some(expression)),
                statements
                @
                Node::Internal {
                    kind: NodeKind::Statements,
                    ..
                } => (statements, None),
                node => unreachable!("Unexpected node: {:?}", node),
            };
        let expression = expression.map(Expression::from).map(Statement::Expression);
        let statements = DepthFirst::find(
            statements,
            |node| matches!(node.kind(), Some(NodeKind::Statement)),
            |node| {
                // keep the order correct
                let mut children = node.children().unwrap_or_default();
                children.reverse();
                children
            },
        )
        .map(Statement::from)
        .chain(expression)
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

    fn const_use(node: Node<'a>) -> Expression<'a> {
        let mut children = check_unpack!(node, NodeKind::ConstantUse);
        let name = children.pop().map(Name::from).expect("Name expected");
        if children.is_empty() {
            Expression::ConstUse(name)
        } else {
            #[cfg(debug_assertions)]
            {
                let _dot = children.pop();
                debug_check! { _dot, Some(Node::Leaf(Token { kind: TokenKind::Separator, lexeme: "." })) };
                let _self = children.pop();
                debug_check! { _self, Some(Node::Leaf(Token { kind: TokenKind::Keyword, lexeme: "self" })) };
            }
            let components = std::iter::once("$self")
                .chain(name.moniker.as_slice().iter().copied())
                .collect::<Vec<_>>();
            Expression::ConstUse(Name::qualified(components))
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
            init_content: body,
        }
    }

    fn chaining_method_invocation(node: Node<'a>) -> Expression<'a> {
        let mut children = check_unpack!(node, NodeKind::ChainingMethodInvocation);
        let _close_bracket = children.pop();
        debug_check! { _close_bracket, Some(Node::Leaf(Token { kind: TokenKind::Separator, lexeme: ")" })) };
        let value = match children.pop() {
            Some(node @ Node::Internal { .. }) => {
                let _open_bracket = children.pop();
                debug_check! { _open_bracket, Some(Node::Leaf(Token { kind: TokenKind::Separator, lexeme: "(" })) };
                Some(Expression::from(node))
            }
            Some(_open_bracket @ Node::Leaf(_)) => {
                debug_check! { _open_bracket, Node::Leaf(Token { kind: TokenKind::Separator, lexeme: "(" }) };
                None
            }
            _ => unreachable!("ChainingMethod has either value or bracket"),
        };
        let identifier = children
            .pop()
            .and_then(|node| node.token())
            .map(|token| token.lexeme)
            .expect("Method name is missing");
        let _dot = children.pop();
        debug_check! { _dot, Some(Node::Leaf(Token { kind: TokenKind::Separator, lexeme: "." })) };
        let receiver = children
            .pop()
            .map(Expression::from)
            .map(Box::new)
            .expect("Receiver missing");
        match *receiver {
            Expression::ChainingMethodInvocation {
                receiver,
                accessors: mut accesses,
            } => {
                accesses.push(Accessor { identifier, value });
                Expression::ChainingMethodInvocation {
                    receiver,
                    accessors: accesses,
                }
            }
            _ => Expression::ChainingMethodInvocation {
                receiver,
                accessors: vec![Accessor { identifier, value }],
            },
        }
    }

    fn eat_struct_body_init(nodes: &mut Vec<Node<'a>>) -> Option<StructInitContent<'a>> {
        if nodes.last().and_then(Node::kind) != Some(NodeKind::StructInitContent) {
            return None;
        }
        let init_content = nodes.pop().map(StructInitContent::from)?;
        match init_content.0.is_empty() {
            true => None,
            false => Some(init_content),
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
                |node| {
                    matches!(
                        node.kind(),
                        Some(NodeKind::NamedParameter | NodeKind::PositionalParameter)
                    )
                },
                |node| node.children().unwrap_or_default(),
            )
            .map(Parameter::from)
            .collect::<Vec<_>>();
            weeder::parameters::weed(&parameters);
            let _open_bracket = nodes.pop();
            debug_check! { _open_bracket, Some(Node::Leaf(Token { kind: TokenKind::Separator, lexeme: "(" })) };
            parameters
        } else {
            vec![]
        }
    }

    fn field_access(node: Node<'a>) -> Expression<'a> {
        let mut children = check_unpack!(node, NodeKind::FieldAccess);
        let field_names = children
            .pop()
            .map(Name::from)
            .expect("FieldAccess ends with Name")
            .moniker
            .as_slice()
            .to_vec();
        let _dot = children.pop();
        debug_check! { _dot, Some(Node::Leaf(Token { kind: TokenKind::Separator, lexeme: "." })) };
        let receiver = children
            .pop()
            .map(Expression::from)
            .expect("FieldAccess should have an receiver");
        Expression::FieldAccess {
            receiver: Box::new(receiver),
            field_names,
        }
    }
}
