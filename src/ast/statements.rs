use super::debug_check;
use super::Expression;
use super::{Node, NodeKind};
use crate::ast::declarations::{ConstantDeclaration, StructDeclaration};

#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub enum Statement<'a> {
    Expression(Expression<'a>),
    ConstantDeclaration(ConstantDeclaration<'a>),
    StructDeclaration(StructDeclaration<'a>),
}

impl<'a> From<Node<'a>> for Statement<'a> {
    fn from(node: Node<'a>) -> Self {
        match node {
            Node::Internal {
                kind: NodeKind::ExpressionStatement,
                children,
            } => Self::expression_statement(children),
            Node::Internal {
                kind: NodeKind::ConstantDeclarationStatement,
                children,
            } => Self::constant_declaration(children),
            Node::Internal {
                kind: NodeKind::StructDeclarationStatement,
                children,
            } => Self::struct_declaration(children),
            Node::Internal {
                kind: NodeKind::Statement,
                mut children,
            } => children
                .pop()
                .map(Self::from)
                .expect("Expect ExpressionStatement or DeclarationStatement"),
            Node::Internal {
                kind: NodeKind::DeclarationStatement,
                mut children,
            } => {
                let _end_of_line = children.pop();
                debug_check! { _end_of_line, Some(Node::Internal { kind: NodeKind::EOL, .. }) };
                children
                    .pop()
                    .map(Self::from)
                    .expect("Expect StructDeclarationStatement or ImportDeclaration")
            }
            node => unreachable!("Unexpected node reached: {:?}", node),
        }
    }
}

impl<'a> Statement<'a> {
    fn expression_statement(mut children: Vec<Node<'a>>) -> Statement<'a> {
        let _end_of_line = children.pop();
        debug_check! { _end_of_line, Some(Node::Internal { kind: NodeKind::EOL, .. }) }
        let expression = children
            .pop()
            .map(Expression::from)
            .expect("Expect Expression");
        Statement::Expression(expression)
    }

    fn constant_declaration(mut children: Vec<Node<'a>>) -> Statement<'a> {
        children
            .pop()
            .map(ConstantDeclaration::from)
            .map(Statement::ConstantDeclaration)
            .expect("Expect ConstantDeclaration")
    }

    fn struct_declaration(mut children: Vec<Node<'a>>) -> Statement<'a> {
        children
            .pop()
            .map(StructDeclaration::from)
            .map(Statement::StructDeclaration)
            .expect("Expect StructDeclaration")
    }
}
